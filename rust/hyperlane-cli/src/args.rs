use clap::{
    error::ErrorKind,
    {Arg, Command, Error},
};
use ethers::{
    core::types::{Address, Bytes, H160, H256},
    prelude::LocalWallet,
};
use std::fs;

use crate::matching_list;

#[derive(Debug)]
pub enum Args {
    Dispatch {
        rpc_url: String,
        mailbox_addr: Address,
        pkey: LocalWallet,
        dest_chain_id: u32,
        recip_addr: H256,
        msg: Bytes,
    },
    Search {
        rpc_url: String,
        mailbox_addr: Address,
        matching_list: matching_list::MatchingList,
    },
}

#[derive(Debug, Clone)]
struct ArgsError;

impl Args {
    const DISPATCH: &str = "dispatch";
    const SEARCH: &str = "search";
    const RPC_URL: &str = "RPC URL";
    const MAILBOX_ADDR: &str = "mailbox address";
    const PKEY: &str = "private key";
    const DEST_CHAIN_ID: &str = "destination chain ID";
    const RECIP_ADDR: &str = "recipient address";
    const MSG: &str = "message";
    const MATCHING_LIST: &str = "matching list";
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
            .subcommand(
                Command::new(Self::SEARCH)
                    .about("Searches for message dispatched from a target chain, matching against a provided criteria")
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
                        Arg::new(Self::MATCHING_LIST)
                            .required(true)
                            .help("Path to a JSON file containing a list of matching criteria for filtering messages"),
                    ),
            )
            .get_matches();
        match matches.subcommand() {
            Some((Self::DISPATCH, dispatch_matches)) => {
                Ok(Args::Dispatch {
                    rpc_url: dispatch_matches
                        .get_one::<String>(Self::RPC_URL)
                        .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                        .to_string(),
                    mailbox_addr: dispatch_matches
                        .get_one::<String>(Self::MAILBOX_ADDR)
                        .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                        .parse::<Address>()
                        .map_err(|_| {
                            "Mailbox address must be a 20-byte hexadecimal string without a 0x prefix"
                        })?,
                    pkey: dispatch_matches
                        .get_one::<String>(Self::PKEY)
                        .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                        .parse::<LocalWallet>()
                        .map_err(|_| {
                            "Signing key must be a 32-byte hexadecimal string without a 0x prefix"
                        })?,
                    dest_chain_id: dispatch_matches
                        .get_one::<String>(Self::DEST_CHAIN_ID)
                        .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                        .parse::<u32>()
                        .map_err(|_| "Destination chain ID must be a number")?,
                    recip_addr: H256::from(
                        dispatch_matches
                            .get_one::<String>(Self::RECIP_ADDR)
                            .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                            .parse::<H160>()
                            .map_err(|_| {
                                "Recipient address must be a 20-byte hexadecimal string without a 0x prefix"
                            })?,
                    ),
                    msg: dispatch_matches
                        .get_one::<String>(Self::MSG)
                        .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                        .parse::<Bytes>()
                        .map_err(|_| "Message body must be a hexadecimal string without a 0x prefix")?,
                })
            },
            Some((Self::SEARCH, search_matches)) => {
                let matching_list_str = fs::read_to_string(search_matches
                    .get_one::<String>(Self::MATCHING_LIST)
                    .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                )?;
                let matching_list = serde_json::from_str(matching_list_str.as_str())?;
                Ok(Args::Search {
                    rpc_url: search_matches
                        .get_one::<String>(Self::RPC_URL)
                        .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                        .to_string(),
                    mailbox_addr: search_matches
                        .get_one::<String>(Self::MAILBOX_ADDR)
                        .ok_or(Error::new(ErrorKind::MissingRequiredArgument))?
                        .parse::<Address>()
                        .map_err(|_| {
                            "Mailbox address must be a 20-byte hexadecimal string without a 0x prefix"
                        })?,
                    matching_list: matching_list,
                })
            },
            _ => { Err(Box::new(Error::new(ErrorKind::MissingSubcommand))) }
        }
    }
}
