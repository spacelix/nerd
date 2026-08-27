//! `nerd.json` schema v1 parser with path-specific validation errors.
//!
//! Unknown keys are rejected with the exact JSON path that was rejected, and
//! prohibited content (secrets, private keys, generated ports) fails parsing
//! before any project can start.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manifest {
    pub schema_version: u32,
    pub name: Option<String>,
    pub node: Option<String>,
    pub https: bool,
    pub framework: Option<String>,
    pub dev_script: Option<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestError {
    Parse(String),
    NotObject,
    MissingSchemaVersion,
    UnsupportedSchemaVersion(u32),
    UnknownKey(String),
    InvalidValue { path: String, reason: String },
    ProhibitedEnvKey(String),
    ProhibitedContent(String),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(formatter, "nerd.json is not valid JSON: {error}"),
            Self::NotObject => formatter.write_str("nerd.json must be a JSON object"),
            Self::MissingSchemaVersion => formatter.write_str("nerd.json requires schemaVersion"),
            Self::UnsupportedSchemaVersion(version) => write!(
                formatter,
                "nerd.json schemaVersion {version} is not supported"
            ),
            Self::UnknownKey(path) => write!(formatter, "{path}: unknown key"),
            Self::InvalidValue { path, reason } => write!(formatter, "{path}: {reason}"),
            Self::ProhibitedEnvKey(path) => write!(
                formatter,
                "{path}: secrets, private keys, credentials, and generated ports are prohibited in nerd.json"
            ),
            Self::ProhibitedContent(path) => write!(
                formatter,
                "{path}: private key material is prohibited in nerd.json"
            ),
        }
    }
}

impl std::error::Error for ManifestError {}

const RESERVED_ENV_KEY_WORDS: &[&str] = &[
    "secret", "token", "password", "passwd", "apikey", "api_key", "private", "port",
];

/// True when any underscore-separated word of the key is a reserved word.
fn has_reserved_env_word(key: &str) -> bool {
    key.to_ascii_lowercase()
        .split('_')
        .any(|word| RESERVED_ENV_KEY_WORDS.contains(&word))
}

/// Parse `nerd.json` text into a validated manifest.
pub fn parse(text: &str) -> Result<Manifest, ManifestError> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|error| ManifestError::Parse(error.to_string()))?;
    let object = value.as_object().ok_or(ManifestError::NotObject)?;

    // Reject unknown top-level keys with their path.
    const ALLOWED_TOP_LEVEL: &[&str] = &[
        "schemaVersion",
        "name",
        "node",
        "https",
        "framework",
        "scripts",
        "env",
    ];
    for key in object.keys() {
        if !ALLOWED_TOP_LEVEL.contains(&key.as_str()) {
            return Err(ManifestError::UnknownKey(key.clone()));
        }
    }

    let schema_version = object
        .get("schemaVersion")
        .ok_or(ManifestError::MissingSchemaVersion)?;
    let schema_version = schema_version
        .as_u64()
        .ok_or_else(|| ManifestError::InvalidValue {
            path: "schemaVersion".to_owned(),
            reason: "must be an integer".to_owned(),
        })?;
    if schema_version > 1 {
        return Err(ManifestError::UnsupportedSchemaVersion(
            u32::try_from(schema_version).unwrap_or(u32::MAX),
        ));
    }

    let read_optional_string = |key: &str| -> Result<Option<String>, ManifestError> {
        match object.get(key) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(value @ serde_json::Value::String(_)) => {
                let text = value.as_str().expect("checked string").trim();
                if text.is_empty() || text.chars().count() > 128 || text.contains('\0') {
                    return Err(ManifestError::InvalidValue {
                        path: key.to_owned(),
                        reason: "length must be between 1 and 128 characters".to_owned(),
                    });
                }
                Ok(Some(text.to_owned()))
            }
            Some(_) => Err(ManifestError::InvalidValue {
                path: key.to_owned(),
                reason: "must be a string".to_owned(),
            }),
        }
    };

    let name = read_optional_string("name")?;
    if let Some(name) = &name
        && !is_dns_label(name)
    {
        return Err(ManifestError::InvalidValue {
            path: "name".to_owned(),
            reason: "must be a lowercase DNS label (letters, digits, hyphen)".to_owned(),
        });
    }

    let node = read_optional_string("node")?;
    let framework = read_optional_string("framework")?;

    let https = match object.get("https") {
        None | Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::Bool(flag)) => *flag,
        Some(_) => {
            return Err(ManifestError::InvalidValue {
                path: "https".to_owned(),
                reason: "must be a boolean".to_owned(),
            });
        }
    };

    // scripts allows only "dev".
    let mut dev_script = None;
    match object.get("scripts") {
        None | Some(serde_json::Value::Null) => {}
        Some(scripts) => {
            let scripts = scripts
                .as_object()
                .ok_or_else(|| ManifestError::InvalidValue {
                    path: "scripts".to_owned(),
                    reason: "must be an object".to_owned(),
                })?;
            for key in scripts.keys() {
                if key != "dev" {
                    return Err(ManifestError::UnknownKey(format!("scripts.{key}")));
                }
            }
            if let Some(dev) = scripts.get("dev") {
                let dev = dev.as_str().ok_or_else(|| ManifestError::InvalidValue {
                    path: "scripts.dev".to_owned(),
                    reason: "must be a string".to_owned(),
                })?;
                if dev.trim().is_empty() || dev.chars().count() > 256 {
                    return Err(ManifestError::InvalidValue {
                        path: "scripts.dev".to_owned(),
                        reason: "length must be between 1 and 256 characters".to_owned(),
                    });
                }
                dev_script = Some(dev.to_owned());
            }
        }
    }

    // env keys must be safe identifiers; values may reference ${NERD_*}
    // placeholders but must never contain secret-like content.
    let mut env = BTreeMap::new();
    match object.get("env") {
        None | Some(serde_json::Value::Null) => {}
        Some(envs) => {
            let envs = envs
                .as_object()
                .ok_or_else(|| ManifestError::InvalidValue {
                    path: "env".to_owned(),
                    reason: "must be an object of string values".to_owned(),
                })?;
            for (key, value) in envs {
                if !is_env_key(key) {
                    return Err(ManifestError::InvalidValue {
                        path: format!("env.{key}"),
                        reason: "must be an uppercase identifier (A-Z, digits, underscore)"
                            .to_owned(),
                    });
                }
                if has_reserved_env_word(key) {
                    return Err(ManifestError::ProhibitedEnvKey(format!("env.{key}")));
                }
                let value = value.as_str().ok_or_else(|| ManifestError::InvalidValue {
                    path: format!("env.{key}"),
                    reason: "must be a string".to_owned(),
                })?;
                if value.contains("-----BEGIN") {
                    return Err(ManifestError::ProhibitedContent(format!("env.{key}")));
                }
                env.insert(key.clone(), value.to_owned());
            }
        }
    }

    Ok(Manifest {
        schema_version: 1,
        name,
        node,
        https,
        framework,
        dev_script,
        env,
    })
}

fn is_dns_label(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 63
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

fn is_env_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && !key.chars().next().is_some_and(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::{ManifestError, parse};

    #[test]
    fn parses_valid_minimal_manifest() {
        let manifest = parse(r#"{"schemaVersion":1}"#).expect("parse minimal");
        assert_eq!(manifest.schema_version, 1);
        assert!(manifest.env.is_empty());
    }

    #[test]
    fn parses_valid_full_manifest() {
        let manifest = parse(
            r#"{
                "schemaVersion": 1,
                "name": "docs",
                "node": "^20",
                "https": true,
                "framework": "vite",
                "scripts": {"dev": "npm run dev"},
                "env": {"NODE_OPTIONS": "--max-old-space-size=2048"}
            }"#,
        )
        .expect("parse full");
        assert_eq!(manifest.name.as_deref(), Some("docs"));
        assert!(manifest.https);
        assert_eq!(manifest.dev_script.as_deref(), Some("npm run dev"));
    }

    #[test]
    fn unknown_keys_are_rejected_with_path() {
        assert_eq!(
            parse(r#"{"schemaVersion":1,"autostart":true}"#),
            Err(ManifestError::UnknownKey("autostart".to_owned()))
        );
        assert_eq!(
            parse(r#"{"schemaVersion":1,"scripts":{"build":"x"}}"#),
            Err(ManifestError::UnknownKey("scripts.build".to_owned()))
        );
    }

    #[test]
    fn schema_version_is_required() {
        assert_eq!(parse("{}"), Err(ManifestError::MissingSchemaVersion));
        assert_eq!(
            parse(r#"{"schemaVersion":2}"#),
            Err(ManifestError::UnsupportedSchemaVersion(2))
        );
    }

    #[test]
    fn prohibited_env_keys_are_rejected() {
        assert_eq!(
            parse(r#"{"schemaVersion":1,"env":{"DATABASE_PASSWORD":"x"}}"#),
            Err(ManifestError::ProhibitedEnvKey(
                "env.DATABASE_PASSWORD".to_owned()
            ))
        );
        assert_eq!(
            parse(r#"{"schemaVersion":1,"env":{"TOKEN":"abc"}}"#),
            Err(ManifestError::ProhibitedEnvKey("env.TOKEN".to_owned()))
        );
    }

    #[test]
    fn private_key_material_is_rejected() {
        let manifest = r#"{"schemaVersion":1,"env":{"KEY":"-----BEGIN RSA PRIVATE KEY-----"}}"#;
        assert_eq!(
            parse(manifest),
            Err(ManifestError::ProhibitedContent("env.KEY".to_owned()))
        );
    }

    #[test]
    fn invalid_name_is_rejected() {
        assert!(matches!(
            parse(r#"{"schemaVersion":1,"name":"Not_Dns!"}"#),
            Err(ManifestError::InvalidValue { path, .. }) if path == "name"
        ));
    }
}
