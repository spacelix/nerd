//! Per-user root CA: generation, DPAPI-protected key, CurrentUser trust store,
//! and on-demand leaf certificates.

use std::{ffi::c_void, fmt, ptr::null_mut};

use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use time::{Duration, OffsetDateTime};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::Cryptography::{
        CERT_CLOSE_STORE_FORCE_FLAG, CERT_CONTEXT, CERT_FIND_SHA1_HASH, CERT_SHA1_HASH_PROP_ID,
        CERT_STORE_ADD_REPLACE_EXISTING, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
        CertAddEncodedCertificateToStore, CertCloseStore, CertCreateCertificateContext,
        CertDeleteCertificateFromStore, CertFindCertificateInStore,
        CertGetCertificateContextProperty, CertOpenSystemStoreW, CryptProtectData,
        CryptUnprotectData,
    },
};

#[repr(C)]
struct HashBlob {
    cb_data: u32,
    pb_data: *mut u8,
}

pub const CA_SUBJECT: &str = "Nerd Root CA";
pub const CA_LIFETIME_DAYS: i64 = 3650;
pub const LEAF_LIFETIME_DAYS: i64 = 90;

const ENCODING: u32 = 0x1 | 0x10000; // X509_ASN_ENCODING | PKCS_7_ASN_ENCODING

#[derive(Debug)]
pub enum CertError {
    Rcgen(rcgen::Error),
    Windows(std::io::Error),
    NotInStore,
    StoreCorrupt,
}

impl fmt::Display for CertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rcgen(_) => formatter.write_str("certificate generation failed"),
            Self::Windows(_) => {
                formatter.write_str("Windows certificate or DPAPI operation failed")
            }
            Self::NotInStore => formatter.write_str("CA certificate is not in the trust store"),
            Self::StoreCorrupt => formatter.write_str("trust store returned an unexpected shape"),
        }
    }
}

impl std::error::Error for CertError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rcgen(error) => Some(error),
            Self::Windows(error) => Some(error),
            Self::NotInStore | Self::StoreCorrupt => None,
        }
    }
}

impl From<rcgen::Error> for CertError {
    fn from(error: rcgen::Error) -> Self {
        Self::Rcgen(error)
    }
}

#[derive(Clone, Debug)]
pub struct CaMaterial {
    pub ca_der: Vec<u8>,
    pub key_pem: String,
    pub fingerprint_hex: String,
}

pub fn generate_ca() -> Result<CaMaterial, CertError> {
    let key = KeyPair::generate()?;
    let mut params = CertificateParams::new(Vec::<String>::new())?;
    params
        .distinguished_name
        .push(DnType::CommonName, CA_SUBJECT);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    params.not_before = OffsetDateTime::now_utc() - Duration::days(1);
    params.not_after = OffsetDateTime::now_utc() + Duration::days(CA_LIFETIME_DAYS);
    let cert = params.self_signed(&key)?;
    let ca_der = cert.der().to_vec();
    let fingerprint_hex = fingerprint_hex(&ca_der)?;
    Ok(CaMaterial {
        ca_der,
        key_pem: key.serialize_pem(),
        fingerprint_hex,
    })
}

pub fn issue_leaf(sans: &[String], ca_key_pem: &str) -> Result<(String, String), CertError> {
    let ca_key = KeyPair::from_pem(ca_key_pem)?;
    let mut issuer_params = CertificateParams::new(Vec::<String>::new())?;
    issuer_params
        .distinguished_name
        .push(DnType::CommonName, CA_SUBJECT);
    issuer_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    issuer_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    let issuer = Issuer::new(issuer_params, ca_key);

    let leaf_key = KeyPair::generate()?;
    let mut params = CertificateParams::new(sans.to_vec())?;
    params.is_ca = IsCa::NoCa;
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.not_before = OffsetDateTime::now_utc() - Duration::days(1);
    params.not_after = OffsetDateTime::now_utc() + Duration::days(LEAF_LIFETIME_DAYS);
    let cert = params.signed_by(&leaf_key, &issuer)?;
    Ok((cert.pem(), leaf_key.serialize_pem()))
}

/// Install a DER-encoded certificate into the CurrentUser Root store.
pub fn install_ca_to_store(ca_der: &[u8]) -> Result<(), CertError> {
    let store = open_root_store()?;
    // SAFETY: `store` is an owned handle closed exactly once below.
    let added = unsafe {
        CertAddEncodedCertificateToStore(
            store,
            ENCODING,
            ca_der.as_ptr(),
            ca_der.len() as u32,
            CERT_STORE_ADD_REPLACE_EXISTING,
            null_mut(),
        )
    };
    // SAFETY: `store` was opened above and must be closed exactly once.
    unsafe {
        CertCloseStore(store, CERT_CLOSE_STORE_FORCE_FLAG);
    }
    if added == 0 {
        Err(CertError::Windows(std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}

pub fn ca_is_installed(ca_der: &[u8]) -> Result<bool, CertError> {
    let fingerprint = fingerprint_hex(ca_der)?;
    let store = open_root_store()?;
    let context = match find_by_thumbprint(store, &fingerprint) {
        Ok(context) => Some(context),
        Err(CertError::NotInStore) => None,
        Err(error) => {
            // SAFETY: the store must be closed exactly once before returning.
            unsafe {
                CertCloseStore(store, CERT_CLOSE_STORE_FORCE_FLAG);
            }
            return Err(error);
        }
    };
    // SAFETY: `store` owns any `context` and closes it exactly once when freed.
    unsafe {
        CertCloseStore(store, CERT_CLOSE_STORE_FORCE_FLAG);
    }
    let _ = context;
    Ok(true)
}

pub fn remove_ca_from_store(ca_der: &[u8]) -> Result<bool, CertError> {
    let fingerprint = fingerprint_hex(ca_der)?;
    let store = open_root_store()?;
    let context = find_by_thumbprint(store, &fingerprint)?;
    // SAFETY: `context` points to a certificate owned by `store`; deletion consumes it.
    let deleted = unsafe { CertDeleteCertificateFromStore(context) };
    // SAFETY: `store` must be closed exactly once; the deleted context is already consumed.
    unsafe {
        CertCloseStore(store, CERT_CLOSE_STORE_FORCE_FLAG);
    }
    Ok(deleted != 0)
}

fn open_root_store() -> Result<windows_sys::Win32::Security::Cryptography::HCERTSTORE, CertError> {
    let name = crate::windows::to_wide("Root");
    // SAFETY: `name` is NUL-terminated and remains alive through the call.
    let store = unsafe { CertOpenSystemStoreW(0, name.as_ptr()) };
    if store.is_null() {
        Err(CertError::Windows(std::io::Error::last_os_error()))
    } else {
        Ok(store)
    }
}

fn find_by_thumbprint(
    store: windows_sys::Win32::Security::Cryptography::HCERTSTORE,
    fingerprint_hex: &str,
) -> Result<*mut CERT_CONTEXT, CertError> {
    let thumbprint = hex_to_bytes(fingerprint_hex).ok_or(CertError::StoreCorrupt)?;
    let blob = HashBlob {
        cb_data: thumbprint.len() as u32,
        pb_data: thumbprint.as_ptr() as *mut u8,
    };
    // SAFETY: `blob` and the store handle remain valid through the lookup.
    let context = unsafe {
        CertFindCertificateInStore(
            store,
            ENCODING,
            0,
            CERT_FIND_SHA1_HASH,
            (&raw const blob).cast::<c_void>(),
            null_mut(),
        )
    };
    if context.is_null() {
        Err(CertError::NotInStore)
    } else {
        Ok(context)
    }
}

/// Compute the SHA-1 thumbprint hex string of a DER certificate using the Windows
/// certificate API, avoiding a separate hash dependency.
fn fingerprint_hex(cert_der: &[u8]) -> Result<String, CertError> {
    // SAFETY: `cert_der` is valid for the length passed and the encoding flags are constants.
    let context =
        unsafe { CertCreateCertificateContext(ENCODING, cert_der.as_ptr(), cert_der.len() as u32) };
    if context.is_null() {
        return Err(CertError::Windows(std::io::Error::last_os_error()));
    }
    let mut required = 0u32;
    // SAFETY: null data with zero length is the documented size-query call.
    let size_query = unsafe {
        CertGetCertificateContextProperty(
            context,
            CERT_SHA1_HASH_PROP_ID,
            null_mut(),
            &mut required,
        )
    };
    if size_query == 0 || required == 0 {
        // SAFETY: the context was created above and must be freed exactly once.
        unsafe {
            windows_sys::Win32::Security::Cryptography::CertFreeCertificateContext(context);
        }
        return Err(CertError::StoreCorrupt);
    }
    let mut buffer = vec![0u8; required as usize];
    // SAFETY: the buffer has the queried size and the context is still valid.
    let loaded = unsafe {
        CertGetCertificateContextProperty(
            context,
            CERT_SHA1_HASH_PROP_ID,
            buffer.as_mut_ptr().cast::<c_void>(),
            &mut required,
        )
    };
    // SAFETY: the context must be freed exactly once.
    unsafe {
        windows_sys::Win32::Security::Cryptography::CertFreeCertificateContext(context);
    }
    if loaded == 0 {
        return Err(CertError::Windows(std::io::Error::last_os_error()));
    }
    Ok(hex_from_bytes(&buffer[..(required as usize)]))
}

/// Protect bytes with DPAPI under the current user scope.
pub fn protect(data: &[u8]) -> Result<Vec<u8>, CertError> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };
    // SAFETY: both blobs are valid and `output` is a writable output; UI is forbidden.
    let protected = unsafe {
        CryptProtectData(
            &input,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if protected == 0 {
        return Err(CertError::Windows(std::io::Error::last_os_error()));
    }
    // SAFETY: `output.pbData` is a LocalAlloc buffer of `output.cbData` bytes.
    let result = unsafe {
        let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
        slice.to_vec()
    };
    // SAFETY: CryptProtectData allocates the output with LocalAlloc.
    unsafe {
        LocalFree(output.pbData.cast::<c_void>());
    }
    Ok(result)
}

/// Unprotect bytes with DPAPI under the current user scope.
pub fn unprotect(data: &[u8]) -> Result<Vec<u8>, CertError> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };
    // SAFETY: both blobs are valid and `output` is a writable output; UI is forbidden.
    let unprotected = unsafe {
        CryptUnprotectData(
            &input,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if unprotected == 0 {
        return Err(CertError::Windows(std::io::Error::last_os_error()));
    }
    // SAFETY: `output.pbData` is a LocalAlloc buffer of `output.cbData` bytes.
    let result = unsafe {
        let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
        slice.to_vec()
    };
    // SAFETY: CryptUnprotectData allocates the output with LocalAlloc.
    unsafe {
        LocalFree(output.pbData.cast::<c_void>());
    }
    Ok(result)
}

fn hex_from_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{generate_ca, hex_from_bytes, hex_to_bytes, issue_leaf, protect, unprotect};

    #[test]
    fn hex_round_trip_is_correct() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        let hex = hex_from_bytes(&bytes);
        assert_eq!(hex, "deadbeef");
        assert_eq!(hex_to_bytes(&hex).expect("decode"), bytes);
        assert!(hex_to_bytes("abc").is_none());
    }

    #[test]
    fn ca_and_leaf_generate_with_dpapi_round_trip() {
        let ca = generate_ca().expect("generate CA");
        assert!(!ca.ca_der.is_empty());
        assert!(!ca.key_pem.is_empty());
        assert_eq!(ca.fingerprint_hex.len(), 40);

        let protected = protect(ca.key_pem.as_bytes()).expect("protect key");
        assert_ne!(protected, ca.key_pem.as_bytes());
        let restored = unprotect(&protected).expect("unprotect key");
        assert_eq!(restored, ca.key_pem.as_bytes());

        let (leaf, leaf_key) =
            issue_leaf(&["foo.test".to_owned()], &ca.key_pem).expect("issue leaf");
        assert!(leaf.contains("BEGIN CERTIFICATE"));
        assert!(leaf_key.contains("BEGIN PRIVATE KEY"));
    }
}
