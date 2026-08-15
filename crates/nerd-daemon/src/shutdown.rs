use std::{fmt, future::Future, io};

use tokio::signal::windows::{ctrl_break, ctrl_c, ctrl_close, ctrl_logoff, ctrl_shutdown};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShutdownReason {
    CtrlC,
    CtrlBreak,
    ConsoleClose,
    Logoff,
    SystemShutdown,
}

impl fmt::Display for ShutdownReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::CtrlC => "ctrl_c",
            Self::CtrlBreak => "ctrl_break",
            Self::ConsoleClose => "console_close",
            Self::Logoff => "logoff",
            Self::SystemShutdown => "system_shutdown",
        };
        formatter.write_str(value)
    }
}

pub fn wait_for_shutdown_signal() -> io::Result<impl Future<Output = ShutdownReason>> {
    let mut ctrl_c = ctrl_c()?;
    let mut ctrl_break = ctrl_break()?;
    let mut ctrl_close = ctrl_close()?;
    let mut ctrl_logoff = ctrl_logoff()?;
    let mut ctrl_shutdown = ctrl_shutdown()?;

    Ok(async move {
        tokio::select! {
            _ = ctrl_c.recv() => ShutdownReason::CtrlC,
            _ = ctrl_break.recv() => ShutdownReason::CtrlBreak,
            _ = ctrl_close.recv() => ShutdownReason::ConsoleClose,
            _ = ctrl_logoff.recv() => ShutdownReason::Logoff,
            _ = ctrl_shutdown.recv() => ShutdownReason::SystemShutdown,
        }
    })
}
