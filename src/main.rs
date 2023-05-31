use eyre::*;

use crate::dex_tracker::pancake::build_pancake_swap;
use crate::dex_tracker::*;
use crate::evm::AppState;
use axum::{body::Body, routing::post, Router};
use eth_sdk::erc20::build_erc_20;
use eth_sdk::EthereumRpcConnectionPool;
use eyre::*;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

mod crypto;
mod dex_tracker;
mod eth_sdk;
mod evm;
mod token;
mod pancake;

#[tokio::main]
async fn main() -> Result<()> {
    // let transport = web3::transports::Http::new(
    //     "https://solemn-dark-tree.quiknode.pro/57a79c40189d56356ac9433efd358f6c2cc05ca7/",
    // )
    // .unwrap();
    // let web3 = web3::Web3::new(transport);

    // let contract_address = Address::from_str("0xeD1DB453C3156Ff3155a97AD217b3087D5Dc5f6E").unwrap();

    // let bytecode = include_str!("./test_token.bin");
    // let abi_file = File::open("src/test_token/test_token_abi.json")?;
    // let reader = BufReader::new(abi_file);
    // let abi_json: serde_json::Value = serde_json::from_reader(reader)?;
    // let deployer = ContractDeployer::new(web3.eth(), abi_json)?.code(bytecode.to_owned());
    // let contract = deployer.sign_with_key_and_execute((), &key).await?;

    // COPIED FROM ORIGINAL
    let eth_pool = EthereumRpcConnectionPool::new(
        "https://solemn-dark-tree.quiknode.pro/57a79c40189d56356ac9433efd358f6c2cc05ca7/"
            .to_string(),
        10,
    )?;
    let app: Router<(), Body> = Router::new()
        .route("/eth-mainnet-swaps", post(handle_eth_swap_mainnet))
        .route("/eth-goerli-swaps", post(handle_eth_swap_goerli))
        .with_state(Arc::new(AppState {
            dex_addresses: DexAddresses::new(),
            eth_pool,
            erc_20: build_erc_20()?,
            pancake_swap: build_pancake_swap()?,
        }));

    let addr = tokio::net::lookup_host(("localhost", 9000))
        .await?
        .next()
        .context("failed to resolve address")?;
    println!("Trade watcher listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
