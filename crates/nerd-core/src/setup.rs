//! Feature 02 network setup contracts.
//!
//! Shared wire shapes for the daemon-to-helper file contract (plan and result)
//! and the setup journal. The helper is invoked with the plan file path as its
//! single argument and reports through the result file and its exit code.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PLAN_VERSION: u32 = 1;

pub const NRPT_NAMESPACE: &str = ".test";
pub const NRPT_NAMESERVER: &str = "127.0.0.1";
pub const NRPT_DISPLAY_NAME: &str = "Nerd";
pub const NRPT_COMMENT_PREFIX: &str = "nerd-";

pub const HELPER_EXIT_OK: i32 = 0;
pub const HELPER_EXIT_INVALID_PLAN: i32 = 1;
pub const HELPER_EXIT_OPERATION_FAILED: i32 = 2;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HelperPlan {
    #[serde(deserialize_with = "positive_u32")]
    pub plan_version: u32,
    pub operation_id: Uuid,
    #[serde(deserialize_with = "nonempty_string")]
    pub journal_path: String,
    pub operations: Vec<HelperOperation>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum HelperOperation {
    NrptAdd(NrptAddParams),
    NrptRemove(NrptRemoveParams),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NrptAddParams {
    #[serde(deserialize_with = "nonempty_string")]
    pub namespace: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub nameserver: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub display_name: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub comment: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NrptRemoveParams {
    #[serde(deserialize_with = "nonempty_string")]
    pub rule_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HelperResult {
    pub operation_id: Uuid,
    pub success: bool,
    pub steps: Vec<HelperStepResult>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HelperStepResult {
    pub operation: String,
    pub status: String,
    pub detail: String,
    #[serde(default, deserialize_with = "optional_nonempty_string")]
    pub rule_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JournalEntry {
    #[serde(deserialize_with = "positive_u64")]
    pub sequence: u64,
    #[serde(deserialize_with = "positive_u64")]
    pub timestamp_ms: u64,
    #[serde(deserialize_with = "nonempty_string")]
    pub operation: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub actor: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub step: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub status: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub detail: String,
}

pub fn nerd_rule_comment(operation_id: &Uuid) -> String {
    format!("{NRPT_COMMENT_PREFIX}{operation_id}")
}

pub fn journal_operation(operation_id: &Uuid) -> String {
    format!("setup-{operation_id}")
}

fn positive_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;
    if value == 0 {
        Err(serde::de::Error::custom("value must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn positive_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    if value == 0 {
        Err(serde::de::Error::custom("value must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn nonempty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() || value.contains('\0') {
        Err(serde::de::Error::custom("string must not be empty"))
    } else {
        Ok(value)
    }
}

fn optional_nonempty_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = &value
        && (value.is_empty() || value.contains('\0'))
    {
        return Err(serde::de::Error::custom("string must not be empty"));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{HelperOperation, HelperPlan, NRPT_NAMESERVER, NRPT_NAMESPACE, NrptAddParams};

    #[test]
    fn helper_plan_round_trips_through_json() {
        let plan = HelperPlan {
            plan_version: super::PLAN_VERSION,
            operation_id: Uuid::new_v4(),
            journal_path: "C:\\tmp\\journal.jsonl".to_owned(),
            operations: vec![HelperOperation::NrptAdd(NrptAddParams {
                namespace: NRPT_NAMESPACE.to_owned(),
                nameserver: NRPT_NAMESERVER.to_owned(),
                display_name: "Nerd".to_owned(),
                comment: "nerd-test".to_owned(),
            })],
        };
        let json = serde_json::to_string(&plan).expect("serialize");
        let decoded: HelperPlan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, plan);
    }
}
