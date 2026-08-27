//! Framework adapters: detection from `package.json` and command building for
//! dev servers with internal-port injection and strict-port behavior.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framework {
    Next,
    Vite,
    Nuxt,
    Astro,
    Nest,
    Express,
}

impl Framework {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Vite => "vite",
            Self::Nuxt => "nuxt",
            Self::Astro => "astro",
            Self::Nest => "nest",
            Self::Express => "express",
        }
    }
}

/// Parse `package.json` dependencies/devDependencies keys.
fn dependency_names(package_json: &str) -> BTreeSet<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(package_json) else {
        return BTreeSet::new();
    };
    let mut names = BTreeSet::new();
    for section in ["dependencies", "devDependencies"] {
        if let Some(object) = value.get(section).and_then(|v| v.as_object()) {
            for key in object.keys() {
                names.insert(key.clone());
            }
        }
    }
    names
}

/// Detect a framework from `package.json` content (never executing scripts).
pub fn detect(package_json: &str, scripts: &str) -> Option<Framework> {
    let deps = dependency_names(package_json);
    if deps.contains("next") {
        return Some(Framework::Next);
    }
    if deps.contains("nuxt") || deps.contains("nuxt3") {
        return Some(Framework::Nuxt);
    }
    if deps.contains("astro") {
        return Some(Framework::Astro);
    }
    if deps.contains("@nestjs/core") {
        return Some(Framework::Nest);
    }
    if deps.contains("vite") || deps.contains("@vitejs/plugin-react") {
        return Some(Framework::Vite);
    }
    // Express/custom: any dev/start script (or any script at all) is the
    // fallback adapter.
    if scripts.contains("dev") || scripts.contains("start") || !scripts.trim().is_empty() {
        return Some(Framework::Express);
    }
    None
}

/// Build the dev command for a framework. Returns (npm-script-name, args-after,
/// port-injection-kind). Port is injected either as a CLI flag (strict) or via
/// the PORT environment variable.
pub fn dev_command(
    framework: Framework,
    dev_script: &str,
    port: u16,
) -> (String, Vec<String>, PortKind) {
    let port_arg = port.to_string();
    match framework {
        Framework::Next => (
            dev_script.to_owned(),
            vec!["--".into(), "-p".into(), port_arg],
            PortKind::Cli,
        ),
        Framework::Vite | Framework::Astro => (
            dev_script.to_owned(),
            vec![
                "--".into(),
                "--port".into(),
                port_arg,
                "--strictPort".into(),
            ],
            PortKind::Cli,
        ),
        Framework::Nuxt => (
            dev_script.to_owned(),
            vec!["--".into(), "--port".into(), port_arg],
            PortKind::Cli,
        ),
        Framework::Nest => (dev_script.to_owned(), Vec::new(), PortKind::Env),
        Framework::Express => (dev_script.to_owned(), Vec::new(), PortKind::Env),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortKind {
    Cli,
    Env,
}

#[cfg(test)]
mod tests {
    use super::{Framework, PortKind, detect, dev_command};

    const NEXT: &str = r#"{"dependencies":{"next":"^14"},"scripts":{"dev":"next dev"}}"#;
    const VITE: &str = r#"{"devDependencies":{"vite":"^5"},"scripts":{"dev":"vite"}}"#;
    const EXPRESS: &str = r#"{"scripts":{"dev":"node server.js"}}"#;

    #[test]
    fn detects_frameworks_without_executing() {
        assert_eq!(detect(NEXT, "next dev"), Some(Framework::Next));
        assert_eq!(detect(VITE, "vite"), Some(Framework::Vite));
        assert_eq!(detect(EXPRESS, "node server.js"), Some(Framework::Express));
        assert_eq!(detect("{}", ""), None);
    }

    #[test]
    fn builds_commands_with_port_injection() {
        let (script, args, kind) = dev_command(Framework::Next, "dev", 3001);
        assert_eq!(script, "dev");
        assert_eq!(args, vec!["--", "-p", "3001"]);
        assert_eq!(kind, PortKind::Cli);
        let (_, _, kind) = dev_command(Framework::Express, "dev", 3001);
        assert_eq!(kind, PortKind::Env);
    }
}
