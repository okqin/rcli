mod cli;
mod process;
mod utils;

pub use cli::{Base64Command, Cli, Commands, HttpCommand, SignFormat, TextCommand};
pub use process::*;
pub use utils::*;

#[allow(async_fn_in_trait)]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}
