//! Project run control: in-memory per-project supervisor state and the
//! start/stop/status/logs surface used by IPC.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::{
    lifecycle::LifecycleState,
    node::NodeManager,
    port_allocator::PortAllocator,
    preflight::{PreflightError, PreflightService},
    state::StateClient,
    supervisor::{LogBuffer, RunConfig, SupervisedRun, stop_run, wait_child_exit},
};

#[derive(Clone, Debug)]
pub struct RunSnapshot {
    pub project_id: Uuid,
    pub state: LifecycleState,
    pub port: Option<u16>,
    pub failure: Option<String>,
}

/// Registered project route mapping used by the proxy for host resolution.
#[derive(Clone, Debug)]
pub struct RegisteredProject {
    pub project_id: Uuid,
    pub route: Option<String>,
}

struct LiveRun {
    run: SupervisedRun,
    group_id: u32,
}

pub struct ControlManager {
    runs: Mutex<BTreeMap<Uuid, LiveRun>>,
    logs: Mutex<BTreeMap<Uuid, Arc<LogBuffer>>>,
    ports: PortAllocator,
    preflight: PreflightService,
    node: NodeManager,
    state: StateClient,
}

#[derive(Clone, Debug)]
pub struct StartOutcome {
    pub project_id: Uuid,
    pub port: Option<u16>,
    pub requires_approval: bool,
}

impl ControlManager {
    pub fn new(state: StateClient, preflight: PreflightService, node: NodeManager) -> Self {
        Self {
            runs: Mutex::new(BTreeMap::new()),
            logs: Mutex::new(BTreeMap::new()),
            ports: PortAllocator::default(),
            preflight,
            node,
            state,
        }
    }

    /// List registered projects with their active route names (for proxy host
    /// resolution). A project without an explicit route uses its derived name.
    pub async fn list_registered(&self) -> Vec<RegisteredProject> {
        let Ok(routes) = self.state.list_routes().await else {
            return Vec::new();
        };
        let Ok(projects) = self.state.list_projects().await else {
            return Vec::new();
        };
        let mut by_id: std::collections::HashMap<Uuid, String> = routes
            .into_iter()
            .map(|route| (route.project_id, route.route_name))
            .collect();
        projects
            .into_iter()
            .map(|project| RegisteredProject {
                project_id: project.project_id,
                route: by_id.remove(&project.project_id).or(Some(project.name)),
            })
            .collect()
    }

    /// Start a project. When the preflight needs approval and none was
    /// provided, the outcome signals it without spawning.
    pub async fn start(&self, name: &str, approved: bool) -> Result<StartOutcome, PreflightError> {
        let runtime = self.preflight.build(name, 0).await?;

        if !approved && self.preflight.needs_approval(&runtime) {
            return Ok(StartOutcome {
                project_id: runtime.project_id,
                port: None,
                requires_approval: true,
            });
        }
        let _ = self.node;

        // Allocate a concrete internal port now.
        let port = self.ports.allocate(None).ok_or_else(|| {
            PreflightError::RuntimeUnavailable("no free internal port".to_owned())
        })?;

        let node_dir = runtime
            .node_exe
            .as_ref()
            .and_then(|path| path.parent())
            .map(|dir| dir.to_path_buf())
            .unwrap_or_default();
        let config = RunConfig {
            project_id: runtime.project_id,
            project_dir: runtime.working_dir.clone(),
            node_exe: node_dir,
            script: runtime.command.clone(),
            args: runtime.args.clone(),
            port,
            port_is_env: matches!(runtime.port_kind, crate::framework::PortKind::Env),
            readiness_path: None,
        };
        let mut run = SupervisedRun::spawn(&config).map_err(PreflightError::RuntimeUnavailable)?;
        let group_id = run.child.id();
        // Drive the lifecycle through readiness before recording the run.
        let state =
            crate::supervisor::wait_ready(&mut run, crate::supervisor::STARTUP_TIMEOUT, group_id);
        if state == crate::lifecycle::LifecycleState::Failed {
            return Err(PreflightError::RuntimeUnavailable(
                "project failed to reach ready state; check the logs".to_owned(),
            ));
        }
        let logs = Arc::clone(&run.logs);
        self.logs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(runtime.project_id, logs);
        self.runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(runtime.project_id, LiveRun { run, group_id });

        Ok(StartOutcome {
            project_id: runtime.project_id,
            port: Some(port),
            requires_approval: false,
        })
    }

    pub fn stop(&self, project_id: Uuid) -> bool {
        let removed = self
            .runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&project_id);
        let Some(live) = removed else {
            return false;
        };
        let _ = stop_run(&live.run, live.group_id);
        let _ = wait_child_exit(&live.run.child, std::time::Duration::from_secs(3));
        self.ports.release(live.run.port);
        true
    }

    pub fn snapshot(&self, project_id: Uuid) -> Option<RunSnapshot> {
        let runs = self
            .runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let live = runs.get(&project_id)?;
        Some(RunSnapshot {
            project_id,
            state: live.run.state,
            port: Some(live.run.port),
            failure: None,
        })
    }

    pub fn logs(&self, project_id: Uuid) -> Option<String> {
        self.logs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&project_id)
            .map(|logs| logs.snapshot())
    }
}
