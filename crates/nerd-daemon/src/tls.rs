//! TLS termination for the loopback HTTPS proxy: per-route leaf certificates
//! issued from the Feature 02 root CA and rustls server config.

use std::io::BufReader;
use std::sync::Arc;

use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use crate::{cert, paths::AppPaths};

#[derive(Debug)]
pub enum TlsError {
    Key(cert::CertError),
    Parse(String),
    Config(String),
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(error) => error.fmt(formatter),
            Self::Parse(message) => write!(formatter, "certificate parsing failed: {message}"),
            Self::Config(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for TlsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Key(error) => Some(error),
            Self::Parse(_) | Self::Config(_) => None,
        }
    }
}

/// Load the DPAPI-protected CA key from the Nerd data directory.
pub fn load_ca_key(paths: &AppPaths) -> Result<Arc<str>, TlsError> {
    let protected = std::fs::read(paths.data_dir.join("ca-key.pem.enc")).map_err(|error| {
        TlsError::Config(format!(
            "cannot read CA key (run network setup first): {error}"
        ))
    })?;
    let key_bytes = cert::unprotect(&protected).map_err(TlsError::Key)?;
    let key = String::from_utf8(key_bytes)
        .map_err(|_| TlsError::Config("protected CA key is not valid UTF-8".to_owned()))?;
    Ok(Arc::from(key))
}

/// Issue a leaf for a hostname and build a rustls ServerConfig serving it.
pub fn server_config_for(hostname: &str, ca_key: &Arc<str>) -> Result<TlsAcceptor, TlsError> {
    let (leaf_pem, leaf_key_pem) =
        cert::issue_leaf(&[hostname.to_owned()], ca_key).map_err(TlsError::Key)?;

    let mut cert_reader = BufReader::new(leaf_pem.as_bytes());
    let cert = rustls_pemfile::certs(&mut cert_reader)
        .next()
        .transpose()
        .map_err(|error| TlsError::Parse(error.to_string()))?
        .ok_or_else(|| TlsError::Parse("no certificate in leaf PEM".to_owned()))?;

    let mut key_reader = BufReader::new(leaf_key_pem.as_bytes());
    let key = rustls_pemfile::private_key(&mut key_reader)
        .map_err(|error| TlsError::Parse(error.to_string()))?
        .ok_or_else(|| TlsError::Parse("no private key in leaf PEM".to_owned()))?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .map_err(|error| TlsError::Config(error.to_string()))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
