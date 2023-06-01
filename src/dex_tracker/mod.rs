use crate::eth_sdk::Transaction;
use crate::evm::{EnumBlockChain, EnumDex};
use axum::extract::State;
use axum::http::StatusCode;
use bytes::Bytes;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use web3::types::H160;

mod pancake_swap;
pub use pancake_swap::*;
mod parse;
use crate::evm;
use crate::evm::AppState;
pub use parse::*;

pub struct DexAddresses {
    inner: HashMap<EnumBlockChain, Vec<(EnumDex, H160)>>,
}
impl Default for DexAddresses {
    fn default() -> Self {
        let mut this = DexAddresses {
            inner: HashMap::new(),
        };

        this.inner.insert(
            EnumBlockChain::EthereumMainnet,
            vec![(
                EnumDex::PancakeSwap,
                H160::from_str("0x13f4EA83D0bd40E75C8222255bc855a974568Dd4").unwrap(),
            )],
        );
        this.inner.insert(
            EnumBlockChain::BscMainnet,
            vec![(
                EnumDex::PancakeSwap,
                H160::from_str("0x13f4EA83D0bd40E75C8222255bc855a974568Dd4").unwrap(),
            )],
        );
        this.inner.insert(
            EnumBlockChain::EthereumGoerli,
            vec![(
                EnumDex::PancakeSwap,
                H160::from_str("0x9a489505a00cE272eAa5e07Dba6491314CaE3796").unwrap(),
            )],
        );
        this.inner.insert(
            EnumBlockChain::BscTestnet,
            vec![(
                EnumDex::PancakeSwap,
                H160::from_str("0x9a489505a00cE272eAa5e07Dba6491314CaE3796").unwrap(),
            )],
        );

        this
    }
}
impl DexAddresses {
    pub fn new() -> DexAddresses {
        Default::default()
    }
    pub fn get(&self, chain: &EnumBlockChain) -> Option<&Vec<(EnumDex, H160)>> {
        self.inner.get(chain)
    }
}

use bip39::{Language, Mnemonic, Seed};
use eyre::*;
use secp256k1::SecretKey;
use tiny_hderive::bip32::ExtendedPrivKey;
use tiny_hderive::bip44::DerivationPath;
use web3::signing::{Key, SecretKeyRef};
use web3::types::{Address, U256};

use crate::pancake::PancakeSmartRouterV3Contract;

const PANCAKE_ROUTER_MAINNET_ADDRESS: &str = "0x13f4EA83D0bd40E75C8222255bc855a974568Dd4";
const PANCAKE_ROUTER_TESTNET_ADDRESS: &str = "0x9a489505a00cE272eAa5e07Dba6491314CaE3796";

pub async fn handle_eth_swap(
    state: Arc<AppState>,
    body: Bytes,
    blockchain: EnumBlockChain,
) -> Result<()> {
    let hashes =
        evm::parse_quickalert_payload(body).context("failed to parse QuickAlerts payload")?;

    let mnemonic = Mnemonic::from_phrase("", Language::English).unwrap();

    let seed = Seed::new(&mnemonic, "");
    let path = DerivationPath::from_str("m/44'/60'/0'/0/0").unwrap();
    let derived_key = ExtendedPrivKey::derive(&seed.as_bytes(), path).unwrap();
    let key = SecretKey::from_slice(&derived_key.secret()).unwrap();

    for hash in hashes {
        let conn = state
            .eth_pool
            .get_conn()
            .await
            .context("error fetching connection guard")?;
        let state = state.clone();
        tokio::spawn(async move {
            let tx = match Transaction::new_and_assume_ready(hash, &conn).await {
                Ok(tx) => tx,
                Err(err) => {
                    println!("error processing tx: {:?}", err);
                    return;
                }
            };
            // if let Err(e) = evm::cache_ethereum_transaction(&tx, &state.db, blockchain).await {
            //     println!("error caching transaction: {:?}", e);
            // };
            let trade = match parse_dex_trade(
                EnumBlockChain::BscMainnet,
                &tx,
                &state.dex_addresses,
                &state.pancake_swap,
            )
            .await
            {
                Ok(trade) => trade,
                Err(err) => {
                    println!("error parsing dex trade: {:?}", err);
                    return;
                }
            };
            // println!("trade: {:?}", trade);
            let contract = match PancakeSmartRouterV3Contract::new(
                conn.clone().into_raw().clone(),
                match H160::from_str(PANCAKE_ROUTER_MAINNET_ADDRESS) {
                    Ok(addr) => addr,
                    Err(_) => {
                        println!("error parsing pancake router address");
                        return;
                    }
                },
            ) {
                Ok(contract) => contract,
                Err(err) => {
                    println!("error creating contract: {:?}", err);
                    return;
                }
            };
            contract
                .copy_trade(&key, trade, U256::from(100), U256::from(0))
                .await
                .unwrap();
        });
    }

    Ok(())
}

pub async fn handle_eth_swap_mainnet(
    state: State<Arc<AppState>>,
    body: Bytes,
) -> Result<(), StatusCode> {
    match handle_eth_swap(state.0, body, EnumBlockChain::EthereumMainnet).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_eth_swap_goerli(
    state: State<Arc<AppState>>,
    body: Bytes,
) -> Result<(), StatusCode> {
    match handle_eth_swap(state.0, body, EnumBlockChain::EthereumGoerli).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
