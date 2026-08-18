pub mod codec;
pub mod ipc;
pub mod setup;

pub const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const IPC_PROTOCOL_VERSION: u32 = 1;
pub const PIPE_NAME: &str = r"\\.\pipe\Nerd.Control.6f843fb5-1bc8-4d47-a038-751c7c218fe8";
