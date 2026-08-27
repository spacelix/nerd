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
    supervisor::{LogBuffer, RunConfig, SupervisedRun, stop_run, wait_child_exit},
};

#[derive(Clone, Debug)]
pub struct RunSnapshot {
    pub project_id: Uuid,
    pub state: LifecycleState,
    pub port: Option<u16>,
    pub failure: Option<String>,
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
}

impl ControlManager {
    pub fn new(preflight: PreflightService, node: NodeManager) -> Self {
        Self {
            runs: Mutex::new(BTreeMap::new()),
            logs: Mutex::new(BTreeMap::new()),
            ports: PortAllocator::default(),
            preflight,
            node,
        }
    }

    /// Start a project. When the preflight needs approval, the caller must
    /// have provided an explicit approval token.
    pub async fn start(
        &self,
        name: &str,
        approved: bool,
    ) -> Result<(Uuid, u16, bool), PreflightError> {
        let runtime = self.preflight.build(name, 0).await?;

        if !approved && self.preflight.needs_approval(&runtime) {
            return Err(PreflightError::RuntimeUnavailable(
                "project requires explicit Trust and Start approval".to_owned(),
            ));
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
            command: runtime.command.clone(),
            args: runtime.args.clone(),
            port,
            port_is_env: matches!(runtime.port_kind, crate::framework::PortKind::Env),
        };
        let run = SupervisedRun::spawn(&config).map_err(PreflightError::RuntimeUnavailable)?;
        let group_id = run.child.id();
        let logs = Arc::clone(&run.logs);
        self.logs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(runtime.project_id, logs);
        self.runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(runtime.project_id, LiveRun { run, group_id });

        Ok((runtime.project_id, port, false))
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
