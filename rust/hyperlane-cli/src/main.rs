use std::sync::Arc;

use ethers::{
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::Signer,
};

mod args;
mod mailbox;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::new()?;

    // Connect to the origin chain and determine it's chain ID
    let provider = Arc::new(Provider::<Http>::try_from(args.rpc_url.clone())?);
    let origin_chain_id = provider.get_chainid().await?.as_u32();
    println!(
        "Connected to network via {}, chain ID is {}",
        args.rpc_url, origin_chain_id
    );

    // Build an interface to the origin chain's mailbox,
    // include signer middleware in order to send transactions
    let wallet = args.pkey.with_chain_id(origin_chain_id);
    let provider = Arc::new(SignerMiddleware::new(provider, wallet));
    let contract = mailbox::Mailbox::new(args.mailbox_addr, provider.clone());

    /*
     * TODO: Determine the proper formula for gas calculation.
     * Currently, the gas used in a transaction is the sum of
     * ether-rs' gas estimation and the mailbox's quote.
     */
    // Receive a quote from mailbox for the dispatched message's gas cost
    let quote_dispatch = contract.quote_dispatch(
        args.dest_chain_id,
        args.recip_addr.to_fixed_bytes(),
        args.msg.clone(),
    );
    let quote = quote_dispatch.call().await?;
    println!("Mailbox's gas quote for this message is: {quote}");

    // Assemble dispatch transaction
    let dispatch = contract.dispatch_0(
        args.dest_chain_id,
        args.recip_addr.to_fixed_bytes(),
        args.msg.clone(),
    );
    let gas_price = provider.get_gas_price().await?;
    let gas = dispatch.estimate_gas().await? + quote;
    println!("Transaction gas estimate: {gas}\nAverage gas price: {gas_price}");
    let dispatch = dispatch.gas(gas).gas_price(gas_price);

    // Send dispatch transaction
    let tx_hash = dispatch.send().await?.tx_hash();
    println!("Dispatch transaction is pending: https://etherscan.io/tx/{tx_hash:?}");

    Ok(())
}
