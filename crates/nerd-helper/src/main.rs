use nerd_core::setup::{
    HELPER_EXIT_INVALID_PLAN, HELPER_EXIT_OK, HELPER_EXIT_OPERATION_FAILED, HelperOperation,
    HelperPlan, HelperResult, HelperStepResult, JournalEntry, NRPT_DISPLAY_NAME, NRPT_NAMESERVER,
    NRPT_NAMESPACE, NrptAddParams, NrptRemoveParams, PLAN_VERSION,
};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn main() {
    let mut arguments = std::env::args_os();
    let program = arguments.next();
    let exit = run(&mut arguments, &program.unwrap_or_default());
    std::process::exit(exit);
}

fn run(arguments: &mut dyn Iterator<Item = std::ffi::OsString>, program: &std::ffi::OsStr) -> i32 {
    let mut positional = Vec::new();
    for argument in arguments {
        let text = argument.to_string_lossy();
        if text == "--version" {
            println!("nerd-helper {}", nerd_core::APPLICATION_VERSION);
            return 0;
        }
        if text.starts_with('-') {
            eprintln!("nerd-helper: unexpected option '{text}'");
            eprintln!("usage: nerd-helper <plan.json> | --version");
            return HELPER_EXIT_INVALID_PLAN;
        }
        positional.push(PathBuf::from(&*text));
    }

    let _ = program;
    if positional.len() != 1 {
        eprintln!("nerd-helper: exactly one plan file argument is required");
        return HELPER_EXIT_INVALID_PLAN;
    }

    let plan_path = &positional[0];
    let plan = match read_plan(plan_path) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("nerd-helper: invalid plan: {error}");
            return HELPER_EXIT_INVALID_PLAN;
        }
    };

    let result_path = result_path_for(plan_path);
    match execute(&plan, &result_path) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("nerd-helper: {error}");
            HELPER_EXIT_OPERATION_FAILED
        }
    }
}

fn read_plan(path: &Path) -> Result<HelperPlan, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read plan file '{}': {error}", path.display()))?;
    if text.len() > 64 * 1024 {
        return Err("plan file exceeds 64 KiB".to_owned());
    }
    let plan: HelperPlan =
        serde_json::from_str(&text).map_err(|error| format!("plan JSON is invalid: {error}"))?;
    if plan.plan_version != PLAN_VERSION {
        return Err(format!("unsupported plan version {}", plan.plan_version));
    }
    if plan.operations.is_empty() {
        return Err("plan contains no operations".to_owned());
    }
    validate_journal_path(&plan.journal_path)?;
    validate_operations(&plan.operations)?;
    Ok(plan)
}

fn validate_journal_path(journal_path: &str) -> Result<(), String> {
    let local_app_data =
        std::env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA is not set".to_owned())?;
    let allowed_root = std::path::Path::new(&local_app_data)
        .join("Nerd")
        .canonicalize()
        .map_err(|_| "cannot resolve Nerd data directory".to_owned())?;
    let journal = std::path::Path::new(journal_path)
        .canonicalize()
        .map_err(|_| "journal path is not resolvable".to_owned())?;
    if !journal.starts_with(&allowed_root) {
        return Err("journal path is outside the Nerd data directory".to_owned());
    }
    Ok(())
}

fn validate_operations(operations: &[HelperOperation]) -> Result<(), String> {
    for operation in operations {
        match operation {
            HelperOperation::NrptAdd(params) => {
                if params.namespace != NRPT_NAMESPACE {
                    return Err(format!("namespace must be '{NRPT_NAMESPACE}'"));
                }
                if params.nameserver != NRPT_NAMESERVER {
                    return Err(format!("nameserver must be '{NRPT_NAMESERVER}'"));
                }
                if params.display_name != NRPT_DISPLAY_NAME {
                    return Err("display name is not a Nerd-owned value".to_owned());
                }
                if !is_safe_comment(&params.comment) {
                    return Err("comment contains invalid characters".to_owned());
                }
            }
            HelperOperation::NrptRemove(params) => {
                if !is_guid(&params.rule_name) {
                    return Err("rule name is not a valid rule identifier".to_owned());
                }
            }
        }
    }
    Ok(())
}

fn is_safe_comment(comment: &str) -> bool {
    comment
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn is_guid(value: &str) -> bool {
    let stripped = strip_guid_braces(value);
    let bytes = stripped.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

fn strip_guid_braces(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('{') && trimmed.ends_with('}') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

fn execute(plan: &HelperPlan, result_path: &Path) -> Result<i32, String> {
    let mut steps = Vec::with_capacity(plan.operations.len());
    let mut success = true;

    for operation in &plan.operations {
        let (name, step) = match operation {
            HelperOperation::NrptAdd(params) => ("nrpt_add", execute_add(plan, params)),
            HelperOperation::NrptRemove(params) => ("nrpt_remove", execute_remove(plan, params)),
        };
        match step {
            Ok(step) => steps.push(step),
            Err(error) => {
                success = false;
                steps.push(HelperStepResult {
                    operation: name.to_owned(),
                    status: "failed".to_owned(),
                    detail: error,
                    rule_name: None,
                });
                break;
            }
        }
    }

    let result = HelperResult {
        operation_id: plan.operation_id,
        success,
        steps,
    };
    write_json_file(result_path, &result)?;
    if success {
        Ok(HELPER_EXIT_OK)
    } else {
        Ok(HELPER_EXIT_OPERATION_FAILED)
    }
}

fn execute_add(plan: &HelperPlan, params: &NrptAddParams) -> Result<HelperStepResult, String> {
    append_journal(plan, "helper", "nrpt_add", "started", &params.namespace)?;

    let script = format!(
        "$r = Add-DnsClientNrptRule -Namespace '{ns}' -NameServers '{server}' -DisplayName '{display}' -Comment '{comment}' -PassThru; if ($r) {{ $r.Name }}",
        ns = params.namespace,
        server = params.nameserver,
        display = params.display_name,
        comment = params.comment,
    );
    let output = run_powershell(&script)?;

    let rule_name = output.trim().to_owned();
    if !is_guid(&rule_name) {
        append_journal(
            plan,
            "helper",
            "nrpt_add",
            "failed",
            "rule name not returned",
        )?;
        return Err("Add-DnsClientNrptRule did not return a rule name".to_owned());
    }

    let verify = run_powershell(&format!(
        "$r = Get-DnsClientNrptRule -Name '{rule_name}' -ErrorAction SilentlyContinue; if ($r) {{ 'present' }} else {{ 'missing' }}"
    ))?;
    if !verify.contains("present") {
        append_journal(
            plan,
            "helper",
            "nrpt_add",
            "failed",
            "postcondition missing",
        )?;
        return Err("added NRPT rule failed postcondition verification".to_owned());
    }

    append_journal(plan, "helper", "nrpt_add", "ok", &rule_name)?;
    Ok(HelperStepResult {
        operation: "nrpt_add".to_owned(),
        status: "ok".to_owned(),
        detail: format!("rule {rule_name} added"),
        rule_name: Some(rule_name),
    })
}

fn execute_remove(
    plan: &HelperPlan,
    params: &NrptRemoveParams,
) -> Result<HelperStepResult, String> {
    let rule_name = strip_guid_braces(&params.rule_name).to_owned();
    append_journal(plan, "helper", "nrpt_remove", "started", &rule_name)?;

    let owned = run_powershell(&format!(
        "$r = Get-DnsClientNrptRule -Name '{name}' -ErrorAction SilentlyContinue; if ($r -and ($r.Comment -like 'nerd-*') -and ($r.Namespace -contains '.test')) {{ 'owned' }} else {{ 'not-owned' }}",
        name = rule_name,
    ))?;
    if !owned.contains("owned") {
        append_journal(
            plan,
            "helper",
            "nrpt_remove",
            "failed",
            "rule is not Nerd-owned",
        )?;
        return Err("refusing to remove a rule that is not Nerd-owned".to_owned());
    }

    let script = format!(
        "Remove-DnsClientNrptRule -Name '{rule_name}' -Confirm:$false",
        rule_name = rule_name,
    );
    let output = run_powershell(&script)?;
    if !output.is_empty() && !output.trim().is_empty() {
        append_journal(
            plan,
            "helper",
            "nrpt_remove",
            "failed",
            "remove cmdlet errored",
        )?;
        return Err(format!("Remove-DnsClientNrptRule reported: {output}"));
    }

    let verify = run_powershell(&format!(
        "$r = Get-DnsClientNrptRule -Name '{name}' -ErrorAction SilentlyContinue; if ($r) {{ 'present' }} else {{ 'missing' }}",
        name = params.rule_name,
    ))?;
    if !verify.contains("missing") {
        append_journal(
            plan,
            "helper",
            "nrpt_remove",
            "failed",
            "postcondition still present",
        )?;
        return Err("removed NRPT rule failed postcondition verification".to_owned());
    }

    append_journal(plan, "helper", "nrpt_remove", "ok", &params.rule_name)?;
    Ok(HelperStepResult {
        operation: "nrpt_remove".to_owned(),
        status: "ok".to_owned(),
        detail: format!("rule {} removed", params.rule_name),
        rule_name: None,
    })
}

fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|error| format!("failed to launch powershell.exe: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "powershell exited with {}: {stderr}",
            output.status.code().unwrap_or(-1)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn append_journal(
    plan: &HelperPlan,
    actor: &str,
    step: &str,
    status: &str,
    detail: &str,
) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&plan.journal_path)
        .map_err(|error| format!("cannot open journal '{}': {error}", plan.journal_path))?;

    let sequence = journal_line_count(&plan.journal_path) + 1;
    let entry = JournalEntry {
        sequence,
        timestamp_ms: now_ms(),
        operation: operation_kind(&plan.operation_id),
        actor: actor.to_owned(),
        step: step.to_owned(),
        status: status.to_owned(),
        detail: detail.to_owned(),
    };
    let line = serde_json::to_string(&entry)
        .map_err(|error| format!("journal entry serialization failed: {error}"))?;
    writeln!(file, "{line}").map_err(|error| format!("journal write failed: {error}"))?;
    Ok(())
}

fn journal_line_count(path: &str) -> u64 {
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return 0,
    };
    let mut text = String::new();
    if file.read_to_string(&mut text).is_err() {
        return 0;
    }
    text.lines().count() as u64
}

fn operation_kind(operation_id: &Uuid) -> String {
    format!("setup-{operation_id}")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn result_path_for(plan_path: &Path) -> PathBuf {
    let mut path = plan_path.as_os_str().to_owned();
    path.push(".result.json");
    PathBuf::from(path)
}

fn write_json_file(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| format!("result serialization failed: {error}"))?;
    std::fs::write(path, text)
        .map_err(|error| format!("cannot write '{}': {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{is_guid, is_safe_comment};

    #[test]
    fn guid_validation_accepts_only_well_formed_guids() {
        assert!(is_guid("{D0EE39CF-C5E0-4D21-A09F-7188ECC36253}"));
        assert!(is_guid("d0ee39cf-c5e0-4d21-a09f-7188ecc36253"));
        assert!(!is_guid("not-a-guid"));
        assert!(!is_guid("{D0EE39CFC5E04D21A09F7188ECC36253}"));
        assert!(!is_guid(""));
    }

    #[test]
    fn comment_validation_rejects_injection() {
        assert!(is_safe_comment("nerd-abc123"));
        assert!(is_safe_comment("Nerd-ABC-123"));
        assert!(!is_safe_comment("nerd-abc'; Remove-Item -Recurse"));
        assert!(!is_safe_comment("nerd abc"));
        assert!(!is_safe_comment("nerd/abc"));
    }
}
