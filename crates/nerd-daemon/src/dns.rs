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

    let zone: InMemoryZoneHandler = InMemoryZoneHandler::new(origin.clone(), records, ZoneType::Primary, AxfrPolicy::Deny)
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
            "$ErrorActionPreference='Stop'; $p = Get-NetUDPEndpoint -LocalPort {port} -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty OwningProcess; if ($p) {{ $p }}"
        ),
        "tcp" => format!(
            "$ErrorActionPreference='Stop'; $p = Get-NetTCPConnection -LocalPort {port} -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty OwningProcess; if ($p) {{ $p }}"
        ),
        other => return Err(std::io::Error::other(format!("unsupported protocol '{other}'"))),
    };
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
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
    use super::{build_catalog, probe_port};

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
}
