//! TLS termination for the loopback HTTPS proxy: per-route leaf certificates
//! issued from the Feature 02 root CA and rustls server config.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rustls::{
    ServerConfig,
    crypto::ring::sign::any_supported_type,
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    server::{ClientHello, ResolvesServerCert},
    sign::CertifiedKey,
};
use rustls_pki_types::pem::PemObject;
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

/// Resolves a per-SNI leaf from the Feature 02 CA, cached per hostname.
/// Every `name.test` route gets a certificate whose SAN matches, so trusted
/// HTTPS works for each framework fixture.
pub struct RouteCertResolver {
    cache: Mutex<HashMap<String, Arc<CertifiedKey>>>,
    ca_key: Arc<str>,
}

impl std::fmt::Debug for RouteCertResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RouteCertResolver")
            .field("cached", &self.cache.lock().map(|c| c.len()).unwrap_or(0))
            .finish()
    }
}

impl RouteCertResolver {
    pub fn new(ca_key: Arc<str>) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ca_key,
        }
    }

    pub fn build_acceptor(&self) -> Result<TlsAcceptor, TlsError> {
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(self.clone()));
        Ok(TlsAcceptor::from(Arc::new(config)))
    }

    fn certified_key_for(&self, hostname: &str) -> Result<Arc<CertifiedKey>, TlsError> {
        if let Some(cached) = self
            .cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(hostname).cloned())
        {
            return Ok(cached);
        }
        let (leaf_pem, leaf_key_pem) =
            cert::issue_leaf(&[hostname.to_owned()], &self.ca_key).map_err(TlsError::Key)?;

        let cert = CertificateDer::from_pem_slice(leaf_pem.as_bytes())
            .map_err(|error| TlsError::Parse(error.to_string()))?;
        let pkcs8 = PrivatePkcs8KeyDer::from_pem_slice(leaf_key_pem.as_bytes())
            .map_err(|error| TlsError::Parse(error.to_string()))?;
        let key = PrivateKeyDer::from(pkcs8);
        let signing_key =
            any_supported_type(&key).map_err(|error| TlsError::Config(error.to_string()))?;

        let certified = Arc::new(CertifiedKey::new(vec![cert], signing_key));
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(hostname.to_owned(), Arc::clone(&certified));
        }
        Ok(certified)
    }
}

impl Clone for RouteCertResolver {
    fn clone(&self) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ca_key: Arc::clone(&self.ca_key),
        }
    }
}

impl ResolvesServerCert for RouteCertResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        let hostname = client_hello
            .server_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "localhost".to_owned());
        self.certified_key_for(&hostname).ok()
    }
}
