use clap::{Arg, Command};
use ethers::{
    core::types::{Address, Bytes, H160, H256},
    prelude::LocalWallet,
};

#[derive(Debug)]
pub struct Args {
    pub rpc_url: String,
    pub mailbox_addr: Address,
    pub pkey: LocalWallet,
    pub dest_chain_id: u32,
    pub recip_addr: H256,
    pub msg: Bytes,
}

impl Args {
    const DISPATCH: &str = "dispatch";
    const RPC_URL: &str = "RPC URL";
    const MAILBOX_ADDR: &str = "mailbox address";
    const PKEY: &str = "private key";
    const DEST_CHAIN_ID: &str = "destination chain ID";
    const RECIP_ADDR: &str = "recipient address";
    const MSG: &str = "message";
    pub fn new() -> Result<Args, Box<dyn std::error::Error>> {
        let matches = Command::new("hyperlane-cli")
            .about("CLI tool for dispatching Hyperlane messages and watching for Hyperlane events")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .allow_external_subcommands(true)
            .subcommand(
                Command::new(Self::DISPATCH)
                    .about("Dispatch a Hyperlane message")
                    .arg(
                        Arg::new(Self::RPC_URL)
                            .long("rpc")
                            .required(true)
                            .help("Origin chain RPC URL"),
                    )
                    .arg(
                        Arg::new(Self::MAILBOX_ADDR)
                            .long("mailbox")
                            .required(true)
                            .help("Mailbox address on the origin chain"),
                    )
                    .arg(
                        Arg::new(Self::PKEY)
                            .long("pkey")
                            .required(true)
                            .help("Private key for signing dispatches"),
                    )
                    .arg(
                        Arg::new(Self::DEST_CHAIN_ID)
                            .short('d')
                            .required(true)
                            .help("Destination chain ID"),
                    )
                    .arg(
                        Arg::new(Self::RECIP_ADDR)
                            .short('r')
                            .required(true)
                            .help("Recipient's address on the destination chain"),
                    )
                    .arg(
                        Arg::new(Self::MSG)
                            .required(true)
                            .help("Message to be dispatched"),
                    ),
            )
            .get_matches();
        let dispatch_matches = matches.subcommand_matches(Self::DISPATCH).unwrap();
        Ok(Args {
            rpc_url: dispatch_matches
                .get_one::<String>(Self::RPC_URL)
                .unwrap()
                .to_string(),
            mailbox_addr: dispatch_matches
                .get_one::<String>(Self::MAILBOX_ADDR)
                .unwrap()
                .parse::<Address>()
                .map_err(|_| {
                    "Mailbox address must be a 20-byte hexadecimal string without a 0x prefix"
                })?,
            pkey: dispatch_matches
                .get_one::<String>(Self::PKEY)
                .unwrap()
                .parse::<LocalWallet>()
                .map_err(|_| {
                    "Signing key must be a 32-byte hexadecimal string without a 0x prefix"
                })?,
            dest_chain_id: dispatch_matches
                .get_one::<String>(Self::DEST_CHAIN_ID)
                .unwrap()
                .parse::<u32>()
                .map_err(|_| "Destination chain ID must be a number")?,
            recip_addr: H256::from(
                dispatch_matches
                    .get_one::<String>(Self::RECIP_ADDR)
                    .unwrap()
                    .parse::<H160>()
                    .map_err(|_| {
                        "Recipient address must be a 20-byte hexadecimal string without a 0x prefix"
                    })?,
            ),
            msg: dispatch_matches
                .get_one::<String>(Self::MSG)
                .unwrap()
                .parse::<Bytes>()
                .map_err(|_| "Message body must be a hexadecimal string without a 0x prefix")?,
        })
    }
}
