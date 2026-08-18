//! Loopback authoritative DNS server for the `.test` zone and port probes.

use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use hickory_server::{
    proto::rr::{
        Name, RData, RecordSet, RecordType, RrKey,
        rdata::{A, NS, SOA},
    },
    server::Server,
    store::in_memory::InMemoryZoneHandler,
    zone_handler::{AxfrPolicy, Catalog, ZoneType},
};
use tokio::{
    net::{TcpListener, UdpSocket},
    task::JoinHandle,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortConflict {
    pub port: u16,
    pub protocol: String,
    pub owning_process_id: u32,
}

#[derive(Debug)]
pub enum DnsError {
    Bind(std::io::Error),
    Catalog(String),
}

impl std::fmt::Display for DnsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bind(_) => formatter.write_str("failed to bind the loopback DNS sockets"),
            Self::Catalog(_) => formatter.write_str("failed to build the .test DNS zone"),
        }
    }
}

impl std::error::Error for DnsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bind(error) => Some(error),
            Self::Catalog(_) => None,
        }
    }
}

pub struct DnsServerHandle {
    task: JoinHandle<()>,
    pub addr: SocketAddr,
}

impl DnsServerHandle {
    pub fn stop(&self) {
        self.task.abort();
    }
}

pub async fn start(addr: SocketAddr) -> Result<DnsServerHandle, DnsError> {
    let udp = UdpSocket::bind(addr).await.map_err(DnsError::Bind)?;
    let tcp = TcpListener::bind(addr).await.map_err(DnsError::Bind)?;
    let catalog = build_catalog().map_err(DnsError::Catalog)?;
    let task = tokio::spawn(async move {
        let mut server = Server::new(catalog);
        server.register_socket(udp);
        server.register_listener(tcp, Duration::from_secs(5), 4096);
        let _ = server.block_until_done().await;
    });
    Ok(DnsServerHandle { task, addr })
}

fn build_catalog() -> Result<Catalog, String> {
    let origin = Name::from_str("test.").map_err(|error| error.to_string())?;
    let name_server = Name::from_str("ns.test.").map_err(|error| error.to_string())?;
    let wildcard = Name::from_str("*.test.").map_err(|error| error.to_string())?;
    let hostmaster = Name::from_str("hostmaster.test.").map_err(|error| error.to_string())?;

    let mut records = BTreeMap::new();

    let soa = RData::SOA(SOA::new(
        name_server.clone(),
        hostmaster,
        1,
        3600,
        3600,
        3600,
        3600,
    ));
    let mut soa_set = RecordSet::new(origin.clone(), RecordType::SOA, 3600);
    soa_set.add_rdata(soa);
    records.insert(RrKey::new(origin.clone().into(), RecordType::SOA), soa_set);

    let mut ns_set = RecordSet::new(origin.clone(), RecordType::NS, 3600);
    ns_set.add_rdata(RData::NS(NS(name_server)));
    records.insert(RrKey::new(origin.clone().into(), RecordType::NS), ns_set);

    let mut wildcard_a = RecordSet::new(wildcard.clone(), RecordType::A, 60);
    wildcard_a.add_rdata(RData::A(A(Ipv4Addr::new(127, 0, 0, 1))));
    records.insert(RrKey::new(wildcard.into(), RecordType::A), wildcard_a);

    let zone: InMemoryZoneHandler =
        InMemoryZoneHandler::new(origin.clone(), records, ZoneType::Primary, AxfrPolicy::Deny)
            .map_err(|error| error.to_string())?;
    let mut catalog = Catalog::new();
    catalog.upsert(origin.into(), vec![Arc::new(zone)]);
    Ok(catalog)
}

/// Probe whether a loopback port is free. When a foreign listener owns it, report
/// its PID. Distinguishes "no listener" (Ok(None)) from "probe failed" (Err).
pub fn probe_port(port: u16, protocol: &str) -> Result<Option<PortConflict>, std::io::Error> {
    let script = match protocol {
        "udp" => format!(
            "$p = Get-NetUDPEndpoint -LocalPort {port} -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty OwningProcess; if ($p) {{ $p }}; exit 0"
        ),
        "tcp" => format!(
            "$p = Get-NetTCPConnection -LocalPort {port} -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty OwningProcess; if ($p) {{ $p }}; exit 0"
        ),
        other => {
            return Err(std::io::Error::other(format!(
                "unsupported protocol '{other}'"
            )));
        }
    };
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!(
            "port {port} {protocol} probe failed: {stderr}"
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let pid = text.trim().parse::<u32>().ok();
    Ok(pid.map(|owning_process_id| PortConflict {
        port,
        protocol: protocol.to_owned(),
        owning_process_id,
    }))
}

#[cfg(test)]
mod tests {
    use std::net::{SocketAddr, ToSocketAddrs};

    use super::{build_catalog, probe_port, start};
    use hickory_server::proto::{
        op::{Message, MessageType, OpCode, Query},
        rr::{Name, RecordType},
    };

    #[test]
    fn unsupported_protocol_is_rejected() {
        assert!(probe_port(53, "sctp").is_err());
    }

    #[test]
    fn test_zone_builds_with_wildcard_a_record() {
        use hickory_server::proto::rr::{LowerName, Name};
        use std::str::FromStr;
        let catalog = build_catalog().expect("build catalog");
        let name = LowerName::from(Name::from_str("test.").unwrap());
        assert!(catalog.contains(&name));
    }

    #[test]
    fn dns_server_resolves_test_names_over_udp() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        runtime.block_on(async {
            let addr = free_loopback_port();
            let handle = start(addr).await.expect("start DNS server");

            let mut query = Message::new(0, MessageType::Query, OpCode::Query);
            query.add_query(Query::query(
                Name::from_ascii("foo.test.").expect("valid name"),
                RecordType::A,
            ));
            let bytes = query.to_vec().expect("encode query");

            let socket = tokio::net::UdpSocket::bind("127.0.0.1:0")
                .await
                .expect("bind client");
            socket.send_to(&bytes, addr).await.expect("send query");
            let mut buffer = [0u8; 512];
            let (len, _) = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                socket.recv_from(&mut buffer),
            )
            .await
            .expect("response timeout")
            .expect("recv response");

            let response = Message::from_vec(&buffer[..len]).expect("parse response");
            use hickory_server::proto::rr::{RData, rdata::A};
            assert!(
                response.answers.iter().any(|record| {
                    record.record_type() == RecordType::A
                        && matches!(record.data, RData::A(A(ip)) if ip.to_string() == "127.0.0.1")
                }),
                "expected A record 127.0.0.1, got {:?}",
                response.answers
            );
            handle.stop();
        });
    }

    fn free_loopback_port() -> SocketAddr {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind for free port");
        let port = listener.local_addr().expect("local addr").port();
        drop(listener);
        ("127.0.0.1", port)
            .to_socket_addrs()
            .expect("resolve addr")
            .next()
            .expect("socket addr")
    }
}
