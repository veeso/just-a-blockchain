mod client;
use client::{App, Task};

use argh::FromArgs;
use std::path::PathBuf;

#[derive(FromArgs)]
#[argh(
    description = "Please, report issues to <https://github.com/veeso/just-a-blockchain>
Please, consider supporting the author <https://ko-fi.com/veeso>"
)]
pub struct Args {
    #[argh(switch, short = 'g', description = "generate a new wallet")]
    pub generate_wallet: bool,
    #[argh(switch, description = "sign genesis block")]
    pub sign_genesis_block: bool,
    #[argh(switch, short = 'D', description = "enable TRACE log level")]
    pub debug: bool,
    #[argh(option, short = 'w', description = "provide wallet path")]
    pub wallet: PathBuf,
}

impl From<&Args> for Task {
    fn from(args: &Args) -> Self {
        if args.generate_wallet {
            Self::GenerateNewWallet
        } else if args.sign_genesis_block {
            Self::SignGenesisBlock
        } else {
            Self::None
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();
    let task = Task::from(&args);
    App::run(task, args)
}
