//! Network setup orchestration: journal, CA lifecycle, helper UAC invocation,
//! NRPT rule management, rollback, and the DNS server handle.

use std::{
    ffi::c_void,
    fmt,
    mem::size_of,
    path::{Path, PathBuf},
    ptr::null_mut,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use nerd_core::{
    ipc::{
        NetworkRepairResponse, NetworkSetupResponse, NetworkStatusResponse,
        NetworkUninstallResponse, PortConflict,
    },
    setup::{
        HelperOperation, HelperPlan, HelperResult, JournalEntry, NRPT_DISPLAY_NAME,
        NRPT_NAMESERVER, NRPT_NAMESPACE, NrptAddParams, NrptRemoveParams, PLAN_VERSION,
        nerd_rule_comment,
    },
};
use uuid::Uuid;
use windows_sys::Win32::{
    Foundation::{CloseHandle, WAIT_OBJECT_0},
    System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject},
    UI::{
        Shell::{SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW},
        WindowsAndMessaging::SW_HIDE,
    },
};

use crate::{
    cert::{self, CertError},
    dns::{self, DnsError, DnsServerHandle, PortConflict as DnsPortConflict},
    paths::AppPaths,
    windows,
};

const JOURNAL_FILENAME: &str = "setup-journal.jsonl";
const NETWORK_STATE_FILENAME: &str = "network-state.json";
const CA_DER_FILENAME: &str = "ca.der";
const CA_KEY_FILENAME: &str = "ca-key.pem.enc";

#[derive(Debug)]
pub enum SetupError {
    PortConflict(PortConflict),
    Cert(CertError),
    Dns(DnsError),
    Helper(String),
    Io(std::io::Error),
    State(String),
    Uac(std::io::Error),
}

impl fmt::Display for SetupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortConflict(conflict) => write!(
                formatter,
                "port {} ({}) is owned by PID {}; Nerd never terminates foreign listeners",
                conflict.port, conflict.protocol, conflict.owning_process_id
            ),
            Self::Cert(error) => error.fmt(formatter),
            Self::Dns(error) => error.fmt(formatter),
            Self::Helper(message) => write!(formatter, "helper operation failed: {message}"),
            Self::Io(error) => error.fmt(formatter),
            Self::State(message) => formatter.write_str(message),
            Self::Uac(error) => write!(formatter, "elevated helper did not start: {error}"),
        }
    }
}

impl std::error::Error for SetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cert(error) => Some(error),
            Self::Dns(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Uac(error) => Some(error),
            Self::PortConflict(_) | Self::Helper(_) | Self::State(_) => None,
        }
    }
}

impl From<CertError> for SetupError {
    fn from(error: CertError) -> Self {
        Self::Cert(error)
    }
}

impl From<DnsError> for SetupError {
    fn from(error: DnsError) -> Self {
        Self::Dns(error)
    }
}

impl From<std::io::Error> for SetupError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NetworkState {
    rule_name: Option<String>,
    ca_fingerprint: Option<String>,
    ca_key_protected: bool,
}

/// Shared network runtime state; the DNS server handle lives here so the daemon
/// can keep serving `.test` queries while holding the bound sockets.
pub struct NetworkRuntime {
    pub dns: Mutex<Option<DnsServerHandle>>,
}

impl Default for NetworkRuntime {
    fn default() -> Self {
        Self {
            dns: Mutex::new(None),
        }
    }
}

pub struct NetworkSetup {
    paths: AppPaths,
    runtime: std::sync::Arc<NetworkRuntime>,
}

impl Clone for NetworkSetup {
    fn clone(&self) -> Self {
        Self {
            paths: self.paths.clone(),
            runtime: std::sync::Arc::clone(&self.runtime),
        }
    }
}

impl NetworkSetup {
    pub fn new(paths: AppPaths, runtime: std::sync::Arc<NetworkRuntime>) -> Self {
        Self { paths, runtime }
    }

    pub fn status(&self) -> Result<NetworkStatusResponse, SetupError> {
        let udp53 = dns::probe_port(53, "udp")?;
        let tcp80 = dns::probe_port(80, "tcp")?;
        let tcp443 = dns::probe_port(443, "tcp")?;
        let dns_listener_active = self
            .runtime
            .dns
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false);
        let nrpt_rule_present = self.nrpt_rule_present()?;
        let ca_present = self.ca_present()?;
        Ok(NetworkStatusResponse {
            dns_listener_active,
            nrpt_rule_present,
            ca_present,
            port_53_conflict: udp53.map(convert_conflict),
            port_80_conflict: tcp80.map(convert_conflict),
            port_443_conflict: tcp443.map(convert_conflict),
        })
    }

    pub async fn setup(&self) -> Result<NetworkSetupResponse, SetupError> {
        let this = self.clone();
        let setup_result = tokio::task::spawn_blocking(move || this.setup_blocking())
            .await
            .map_err(|error| SetupError::State(format!("setup worker panicked: {error}")))?;
        if setup_result.is_ok() {
            self.start_dns_if_free().await;
        }
        setup_result
    }

    fn setup_blocking(&self) -> Result<NetworkSetupResponse, SetupError> {
        let operation_id = Uuid::new_v4();
        self.append_journal(&operation_id, "daemon", "setup", "started", "begin")?;
        let mut created_ca = false;
        let mut created_rule = false;
        let mut rule_name = None;

        let setup_result = (|| {
            if let Some(conflict) = dns::probe_port(53, "udp")? {
                return Err(SetupError::PortConflict(convert_conflict(conflict)));
            }

            let state = self.load_state()?;
            let ca = match self.ca_material(&state)? {
                Some(ca) => ca,
                None => {
                    let ca = cert::generate_ca()?;
                    let protected = cert::protect(ca.key_pem.as_bytes())?;
                    std::fs::write(self.paths.data_dir.join(CA_DER_FILENAME), &ca.ca_der)?;
                    std::fs::write(self.paths.data_dir.join(CA_KEY_FILENAME), &protected)?;
                    created_ca = true;
                    cert::install_ca_to_store(&ca.ca_der)?;
                    self.append_journal(
                        &operation_id,
                        "daemon",
                        "ca_install",
                        "ok",
                        &ca.fingerprint_hex,
                    )?;
                    ca
                }
            };

            if !self.nrpt_rule_present()? {
                let plan = HelperPlan {
                    plan_version: PLAN_VERSION,
                    operation_id,
                    journal_path: self.journal_path().to_string_lossy().into_owned(),
                    operations: vec![HelperOperation::NrptAdd(NrptAddParams {
                        namespace: NRPT_NAMESPACE.to_owned(),
                        nameserver: NRPT_NAMESERVER.to_owned(),
                        display_name: NRPT_DISPLAY_NAME.to_owned(),
                        comment: nerd_rule_comment(&operation_id),
                    })],
                };
                let result = self.invoke_helper(plan)?;
                rule_name = result.steps.first().and_then(|step| step.rule_name.clone());
                created_rule = true;
            }

            let next_state = NetworkState {
                rule_name: rule_name.clone().or_else(|| state.rule_name.clone()),
                ca_fingerprint: Some(ca.fingerprint_hex.clone()),
                ca_key_protected: true,
            };
            self.save_state(&next_state)?;
            Ok(NetworkSetupResponse {
                success: true,
                rolled_back: false,
                nrpt_rule_name: rule_name.clone().or_else(|| state.rule_name.clone()),
                ca_fingerprint: Some(ca.fingerprint_hex),
            })
        })();

        match setup_result {
            Ok(response) => {
                self.append_journal(&operation_id, "daemon", "setup", "ok", "complete")?;
                Ok(response)
            }
            Err(error) => {
                let mut rolled_back = false;
                if created_ca {
                    let _ = self.rollback_ca(&operation_id);
                    rolled_back = true;
                }
                if created_rule && let Some(name) = &rule_name {
                    let _ = self.remove_rule_elevated(&operation_id, name);
                    rolled_back = true;
                }
                self.append_journal(
                    &operation_id,
                    "daemon",
                    "setup",
                    "failed",
                    &error.to_string(),
                )?;
                if rolled_back {
                    self.append_journal(&operation_id, "daemon", "rollback", "ok", "complete")?;
                }
                Err(error)
            }
        }
    }

    pub async fn uninstall(&self) -> Result<NetworkUninstallResponse, SetupError> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.uninstall_blocking())
            .await
            .map_err(|error| SetupError::State(format!("uninstall worker panicked: {error}")))?
    }

    fn uninstall_blocking(&self) -> Result<NetworkUninstallResponse, SetupError> {
        let operation_id = Uuid::new_v4();
        self.append_journal(&operation_id, "daemon", "uninstall", "started", "begin")?;
        let state = self.load_state()?;
        let mut removed_rule = false;
        if let Some(rule_name) = &state.rule_name {
            removed_rule = self.remove_rule_elevated(&operation_id, rule_name)?;
        }
        let removed_ca = self.rollback_ca(&operation_id)?;
        if let Ok(mut guard) = self.runtime.dns.lock()
            && let Some(handle) = guard.take()
        {
            handle.stop();
        }
        let preserved_unrelated_rules = self.count_unrelated_rules()?;
        let _ = std::fs::remove_file(self.network_state_path());
        self.append_journal(&operation_id, "daemon", "uninstall", "ok", "complete")?;
        Ok(NetworkUninstallResponse {
            success: true,
            removed_nrpt_rule: removed_rule,
            removed_ca,
            preserved_unrelated_rules,
        })
    }

    pub async fn repair(&self) -> Result<NetworkRepairResponse, SetupError> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.repair_blocking())
            .await
            .map_err(|error| SetupError::State(format!("repair worker panicked: {error}")))?
    }

    fn repair_blocking(&self) -> Result<NetworkRepairResponse, SetupError> {
        let operation_id = Uuid::new_v4();
        self.append_journal(&operation_id, "daemon", "repair", "started", "begin")?;
        let mut action = String::new();
        let state = self.load_state()?;

        if !self.nrpt_rule_present()? {
            let plan = HelperPlan {
                plan_version: PLAN_VERSION,
                operation_id,
                journal_path: self.journal_path().to_string_lossy().into_owned(),
                operations: vec![HelperOperation::NrptAdd(NrptAddParams {
                    namespace: NRPT_NAMESPACE.to_owned(),
                    nameserver: NRPT_NAMESERVER.to_owned(),
                    display_name: NRPT_DISPLAY_NAME.to_owned(),
                    comment: nerd_rule_comment(&operation_id),
                })],
            };
            let result = self.invoke_helper(plan)?;
            let rule_name = result.steps.first().and_then(|step| step.rule_name.clone());
            action.push_str("re-added NRPT rule");
            self.save_state(&NetworkState {
                rule_name: rule_name.or_else(|| state.rule_name.clone()),
                ..state.clone()
            })?;
        }

        if !self.ca_present()? {
            let state = self.load_state()?;
            if let Some(expected) = &state.ca_fingerprint {
                let der = std::fs::read(self.paths.data_dir.join(CA_DER_FILENAME))?;
                let actual = cert_fingerprint(&der)?;
                if *expected != actual {
                    return Err(SetupError::State(
                        "CA fingerprint does not match Nerd ownership record; refusing to install"
                            .to_owned(),
                    ));
                }
                cert::install_ca_to_store(&der)?;
                action.push_str("; re-installed CA");
            }
        }

        self.append_journal(&operation_id, "daemon", "repair", "ok", &action)?;
        Ok(NetworkRepairResponse {
            success: true,
            action: if action.is_empty() {
                "no repair needed".to_owned()
            } else {
                action
            },
        })
    }

    pub async fn start_dns_if_free(&self) {
        let conflict = match dns::probe_port(53, "udp") {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => return,
        };
        if conflict {
            return;
        }
        let Ok(addr) = "127.0.0.1:53".parse() else {
            return;
        };
        let Ok(handle) = dns::start(addr).await else {
            return;
        };
        if let Ok(mut guard) = self.runtime.dns.lock() {
            *guard = Some(handle);
        }
    }

    fn ca_material(&self, state: &NetworkState) -> Result<Option<cert::CaMaterial>, SetupError> {
        let der_path = self.paths.data_dir.join(CA_DER_FILENAME);
        if !der_path.exists() {
            return Ok(None);
        }
        let ca_der = std::fs::read(&der_path)?;
        let fingerprint_hex = cert_fingerprint(&ca_der)?;
        let key_pem = {
            let protected = std::fs::read(self.paths.data_dir.join(CA_KEY_FILENAME))?;
            let key_bytes = cert::unprotect(&protected)?;
            String::from_utf8(key_bytes)
                .map_err(|_| SetupError::State("protected CA key is not valid UTF-8".to_owned()))?
        };
        if let Some(expected) = &state.ca_fingerprint
            && *expected != fingerprint_hex
        {
            return Err(SetupError::State(
                "CA fingerprint does not match Nerd ownership record".to_owned(),
            ));
        }
        Ok(Some(cert::CaMaterial {
            ca_der,
            key_pem,
            fingerprint_hex,
        }))
    }

    fn ca_present(&self) -> Result<bool, SetupError> {
        let state = self.load_state()?;
        if state.ca_fingerprint.is_none() {
            return Ok(false);
        }
        let der_path = self.paths.data_dir.join(CA_DER_FILENAME);
        if !der_path.exists() {
            return Ok(false);
        }
        let ca_der = std::fs::read(&der_path)?;
        Ok(cert::ca_is_installed(&ca_der).unwrap_or(false))
    }

    fn rollback_ca(&self, operation_id: &Uuid) -> Result<bool, SetupError> {
        let state = self.load_state()?;
        let der_path = self.paths.data_dir.join(CA_DER_FILENAME);
        let removed = if der_path.exists() {
            let ca_der = std::fs::read(&der_path)?;
            let actual = cert_fingerprint(&ca_der)?;
            match &state.ca_fingerprint {
                Some(expected) if *expected == actual => cert::remove_ca_from_store(&ca_der)?,
                _ => {
                    self.append_journal(
                        operation_id,
                        "daemon",
                        "ca_remove",
                        "skipped",
                        "fingerprint mismatch; refusing to remove foreign trust anchor",
                    )?;
                    false
                }
            }
        } else {
            false
        };
        if let Some(fingerprint) = &state.ca_fingerprint {
            self.append_journal(operation_id, "daemon", "ca_remove", "ok", fingerprint)?;
        }
        let _ = std::fs::remove_file(&der_path);
        let _ = std::fs::remove_file(self.paths.data_dir.join(CA_KEY_FILENAME));
        Ok(removed)
    }

    fn nrpt_rule_present(&self) -> Result<bool, SetupError> {
        let output = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "$r = Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Where-Object { $_.Namespace -contains '.test' -and $_.Comment -like 'nerd-*' }; if ($r) { 'present' } else { 'missing' }",
            ])
            .output()?;
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text.contains("present"))
    }

    fn count_unrelated_rules(&self) -> Result<u32, SetupError> {
        let output = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "(Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Where-Object { $_.Comment -notlike 'nerd-*' }).Count",
            ])
            .output()?;
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text.trim().parse::<u32>().unwrap_or(0))
    }

    fn remove_rule_elevated(
        &self,
        operation_id: &Uuid,
        rule_name: &str,
    ) -> Result<bool, SetupError> {
        let plan = HelperPlan {
            plan_version: PLAN_VERSION,
            operation_id: *operation_id,
            journal_path: self.journal_path().to_string_lossy().into_owned(),
            operations: vec![HelperOperation::NrptRemove(NrptRemoveParams {
                rule_name: rule_name.to_owned(),
            })],
        };
        let result = self.invoke_helper(plan)?;
        let removed = result
            .steps
            .first()
            .map(|step| step.status == "ok")
            .unwrap_or(false);
        if removed {
            let mut state = self.load_state()?;
            state.rule_name = None;
            self.save_state(&state)?;
        }
        Ok(removed)
    }

    fn invoke_helper(&self, plan: HelperPlan) -> Result<HelperResult, SetupError> {
        let plan_path = self.plan_path(&plan.operation_id);
        let text = serde_json::to_string(&plan)
            .map_err(|error| SetupError::State(format!("plan serialization failed: {error}")))?;
        std::fs::write(&plan_path, text)?;

        let helper_path = helper_executable()?;
        let verb = windows::to_wide("runas");
        let file = windows::to_wide(&helper_path);
        let parameters = windows::to_wide(&plan_path.to_string_lossy());
        let directory = windows::to_wide(&self.paths.data_dir.to_string_lossy());
        let mut info = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
            hwnd: null_mut(),
            lpVerb: verb.as_ptr(),
            lpFile: file.as_ptr(),
            lpParameters: parameters.as_ptr(),
            lpDirectory: directory.as_ptr(),
            nShow: SW_HIDE,
            hInstApp: null_mut(),
            lpIDList: null_mut(),
            lpClass: null_mut(),
            hkeyClass: null_mut(),
            dwHotKey: 0,
            Anonymous: unsafe { std::mem::zeroed() },
            hProcess: null_mut(),
        };
        // SAFETY: every pointer references a live NUL-terminated buffer for the call's duration.
        let launched = unsafe { ShellExecuteExW(&mut info) };
        if launched == 0 {
            return Err(SetupError::Uac(std::io::Error::last_os_error()));
        }
        // SAFETY: ShellExecuteExW with NOCLOSEPROCESS returned a valid handle.
        let wait = unsafe { WaitForSingleObject(info.hProcess, INFINITE) };
        if wait != WAIT_OBJECT_0 {
            // SAFETY: the process handle must be closed exactly once.
            unsafe {
                CloseHandle(info.hProcess);
            }
            return Err(SetupError::Uac(std::io::Error::last_os_error()));
        }
        let mut exit_code = 0u32;
        // SAFETY: the process has exited and `exit_code` is a writable output.
        let queried = unsafe { GetExitCodeProcess(info.hProcess, &mut exit_code) };
        // SAFETY: the process handle must be closed exactly once.
        unsafe {
            CloseHandle(info.hProcess);
        }
        if queried == 0 {
            let _ = std::fs::remove_file(&plan_path);
            let _ = std::fs::remove_file(result_path_for(&plan_path));
            return Err(SetupError::Uac(std::io::Error::last_os_error()));
        }
        if exit_code != 0 {
            let result_path = result_path_for(&plan_path);
            let detail = std::fs::read_to_string(&result_path)
                .ok()
                .and_then(|text| serde_json::from_str::<HelperResult>(&text).ok())
                .and_then(|result| result.steps.first().map(|step| step.detail.clone()))
                .unwrap_or_else(|| format!("helper exited with code {exit_code}"));
            let _ = std::fs::remove_file(&plan_path);
            let _ = std::fs::remove_file(&result_path);
            return Err(SetupError::Helper(detail));
        }

        let result_path = result_path_for(&plan_path);
        let result_text = std::fs::read_to_string(&result_path)?;
        let result: HelperResult = serde_json::from_str(&result_text)
            .map_err(|error| SetupError::State(format!("helper result is invalid: {error}")))?;
        let _ = std::fs::remove_file(&plan_path);
        let _ = std::fs::remove_file(&result_path);
        if !result.success {
            let detail = result
                .steps
                .first()
                .map(|step| step.detail.clone())
                .unwrap_or_else(|| "no step detail".to_owned());
            return Err(SetupError::Helper(detail));
        }
        Ok(result)
    }

    fn plan_path(&self, operation_id: &Uuid) -> PathBuf {
        self.paths
            .data_dir
            .join(format!("plan-{operation_id}.json"))
    }

    fn journal_path(&self) -> PathBuf {
        self.paths.data_dir.join(JOURNAL_FILENAME)
    }

    fn network_state_path(&self) -> PathBuf {
        self.paths.data_dir.join(NETWORK_STATE_FILENAME)
    }

    fn load_state(&self) -> Result<NetworkState, SetupError> {
        let path = self.network_state_path();
        if !path.exists() {
            return Ok(NetworkState::default());
        }
        let text = std::fs::read_to_string(&path)?;
        serde_json::from_str(&text)
            .map_err(|error| SetupError::State(format!("network state is invalid: {error}")))
    }

    fn save_state(&self, state: &NetworkState) -> Result<(), SetupError> {
        let text = serde_json::to_string_pretty(state)
            .map_err(|error| SetupError::State(format!("state serialization failed: {error}")))?;
        std::fs::write(self.network_state_path(), text)?;
        Ok(())
    }

    fn append_journal(
        &self,
        operation_id: &Uuid,
        actor: &str,
        step: &str,
        status: &str,
        detail: &str,
    ) -> Result<(), SetupError> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.journal_path())?;
        let sequence = journal_line_count(&self.journal_path()) + 1;
        let entry = JournalEntry {
            sequence,
            timestamp_ms: now_ms(),
            operation: nerd_core::setup::journal_operation(operation_id),
            actor: actor.to_owned(),
            step: step.to_owned(),
            status: status.to_owned(),
            detail: detail.to_owned(),
        };
        let line = serde_json::to_string(&entry)
            .map_err(|error| SetupError::State(format!("journal serialization failed: {error}")))?;
        use std::io::Write;
        writeln!(file, "{line}")?;
        Ok(())
    }
}

fn convert_conflict(conflict: DnsPortConflict) -> PortConflict {
    PortConflict {
        port: conflict.port,
        protocol: conflict.protocol,
        owning_process_id: conflict.owning_process_id,
    }
}

fn cert_fingerprint(cert_der: &[u8]) -> Result<String, CertError> {
    // Recompute the fingerprint with the same Windows API used at generation.
    let context = unsafe {
        windows_sys::Win32::Security::Cryptography::CertCreateCertificateContext(
            0x1 | 0x10000,
            cert_der.as_ptr(),
            cert_der.len() as u32,
        )
    };
    if context.is_null() {
        return Err(CertError::Windows(std::io::Error::last_os_error()));
    }
    let mut required = 0u32;
    unsafe {
        windows_sys::Win32::Security::Cryptography::CertGetCertificateContextProperty(
            context,
            3,
            null_mut(),
            &mut required,
        )
    };
    if required == 0 {
        unsafe {
            windows_sys::Win32::Security::Cryptography::CertFreeCertificateContext(context);
        }
        return Err(CertError::StoreCorrupt);
    }
    let mut buffer = vec![0u8; required as usize];
    unsafe {
        windows_sys::Win32::Security::Cryptography::CertGetCertificateContextProperty(
            context,
            3,
            buffer.as_mut_ptr().cast::<c_void>(),
            &mut required,
        )
    };
    unsafe {
        windows_sys::Win32::Security::Cryptography::CertFreeCertificateContext(context);
    }
    Ok(buffer[..(required as usize)]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn helper_executable() -> Result<String, SetupError> {
    let current = std::env::current_exe()?;
    let directory = current
        .parent()
        .ok_or_else(|| SetupError::State("cannot resolve helper directory".to_owned()))?;
    Ok(directory
        .join("nerd-helper.exe")
        .to_string_lossy()
        .into_owned())
}

fn result_path_for(plan_path: &Path) -> PathBuf {
    let mut path = plan_path.as_os_str().to_owned();
    path.push(".result.json");
    PathBuf::from(path)
}

fn journal_line_count(path: &Path) -> u64 {
    std::fs::read_to_string(path)
        .map(|text| text.lines().count() as u64)
        .unwrap_or(0)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
