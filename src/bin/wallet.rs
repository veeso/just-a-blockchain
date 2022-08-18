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
    #[argh(
        switch,
        short = 'b',
        description = "get balance and transactions for this wallet"
    )]
    pub balance: bool,
    #[argh(
        option,
        short = 'B',
        description = "get balance and transactions for provided wallet"
    )]
    pub get_balance_for: Option<String>,
    #[argh(switch, short = 'g', description = "generate a new wallet")]
    pub generate_wallet: bool,
    #[argh(switch, description = "send money")]
    pub send: bool,
    #[argh(switch, description = "sign genesis block")]
    pub sign_genesis_block: bool,
    #[argh(option, short = 'w', description = "provide wallet path")]
    pub wallet: PathBuf,
}

impl From<&Args> for Task {
    fn from(args: &Args) -> Self {
        if args.generate_wallet {
            Self::GenerateNewWallet
        } else if args.sign_genesis_block {
            Self::SignGenesisBlock
        } else if args.balance {
            Self::GetBalance
        } else if let Some(addr) = args.get_balance_for.as_ref() {
            Self::GetBalanceFor(addr.to_string())
        } else if args.send {
            Self::Send
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
