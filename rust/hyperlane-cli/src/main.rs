use std::sync::Arc;

use ethers::{
    core::types::{Address, Bytes, H256},
    prelude::{LocalWallet, SignerMiddleware},
    providers::{Http, Middleware, Provider, StreamExt},
    signers::Signer,
};

use hyperlane_core::HyperlaneMessage;

mod args;
mod mailbox;
mod matching_list;

async fn dispatch(
    rpc_url: String,
    mailbox_addr: Address,
    pkey: LocalWallet,
    dest_chain_id: u32,
    recip_addr: H256,
    msg: Bytes,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the origin chain and determine it's chain ID
    let provider = Arc::new(Provider::<Http>::try_from(rpc_url.clone())?);
    let origin_chain_id = provider.get_chainid().await?.as_u32();
    println!(
        "Connected to network via {}, chain ID is {}",
        rpc_url, origin_chain_id
    );

    // Build an interface to the origin chain's mailbox,
    // include signer middleware in order to send transactions
    let wallet = pkey.with_chain_id(origin_chain_id);
    let provider = Arc::new(SignerMiddleware::new(provider, wallet));
    let contract = mailbox::Mailbox::new(mailbox_addr, provider.clone());

    // Receive a quote from mailbox for the dispatched message's gas cost
    let quote_dispatch =
        contract.quote_dispatch(dest_chain_id, recip_addr.to_fixed_bytes(), msg.clone());
    let quote = quote_dispatch.call().await?;
    println!("Mailbox's gas quote for this message is: {quote}");

    // Assemble dispatch transaction
    let dispatch = contract.dispatch_0(dest_chain_id, recip_addr.to_fixed_bytes(), msg.clone());
    let gas_price = provider.get_gas_price().await?;
    println!("Average gas price: {gas_price}");
    let gas = dispatch.estimate_gas().await?;
    println!("Transaction gas estimate: {gas}\nAverage gas price: {gas_price}");
    let dispatch = dispatch.gas(gas).gas_price(gas_price);

    // Send dispatch transaction
    let tx_hash = dispatch.send().await?.tx_hash();
    println!("Dispatch transaction is pending: https://etherscan.io/tx/{tx_hash:?}");
    Ok(())
}

async fn search(
    rpc_url: String,
    mailbox_addr: Address,
    matching_list: matching_list::MatchingList,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(Provider::<Http>::try_from(rpc_url.clone())?);
    let origin_chain_id = provider.get_chainid().await?.as_u32();
    println!(
        "Connected to network via {}, chain ID is {}",
        rpc_url, origin_chain_id
    );

    // Build an interface to the origin chain's mailbox,
    let contract = mailbox::Mailbox::new(mailbox_addr, provider.clone());

    let event = contract.event::<mailbox::DispatchFilter>();
    let mut event_stream = event.stream().await?;

    while let Some(Ok(dispatch_filter)) = event_stream.next().await {
        let hyperlane_msg: HyperlaneMessage = dispatch_filter.message.to_vec().into();
        if matching_list.msg_matches(&hyperlane_msg, false) {
            println!("{:?}", hyperlane_msg);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match args::Args::new()? {
        args::Args::Dispatch {
            rpc_url,
            mailbox_addr,
            pkey,
            dest_chain_id,
            recip_addr,
            msg,
        } => dispatch(rpc_url, mailbox_addr, pkey, dest_chain_id, recip_addr, msg).await,
        args::Args::Search {
            rpc_url,
            mailbox_addr,
            matching_list,
        } => search(rpc_url, mailbox_addr, matching_list).await,
    }
}
