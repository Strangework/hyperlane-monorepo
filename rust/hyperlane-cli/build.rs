use ethers::contract::Abigen;
use std::env;
use std::path::Path;

fn main() {
    // Generate a Rust model for interfacing against the Mailbox ABI
    let out_dir = env::current_dir().unwrap();
    let out_file = Path::new(&out_dir).join("src/mailbox.rs");
    if out_file.exists() {
        std::fs::remove_file(&out_file).unwrap();
    }
    Abigen::new(
        "Mailbox",
        "../chains/hyperlane-ethereum/abis/IMailbox.abi.json",
    )
    .unwrap()
    .generate()
    .unwrap()
    .write_to_file(out_file)
    .unwrap();
}
