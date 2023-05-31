use crate::dex_tracker::pancake::build_pancake_swap;
use crate::dex_tracker::{DexAddresses, PancakeSwap};
use crate::eth_sdk::erc20::build_erc_20;
use crate::eth_sdk::{ContractCall, EthereumRpcConnectionPool, TransactionReady};
use bytes::Bytes;
use eyre::*;
use serde::{Deserialize, Serialize};

// use tracing::error;
use web3::ethabi::Contract;
use web3::types::{H160, H256, U256};

pub struct AppState {
    pub dex_addresses: DexAddresses,
    pub eth_pool: EthereumRpcConnectionPool,
    pub pancake_swap: PancakeSwap,
    // pub db: DbClient,
    // pub stablecoin_addresses: StableCoinAddresses,
    pub erc_20: Contract,
}
impl AppState {
    pub fn new(eth_pool: EthereumRpcConnectionPool) -> Result<Self> {
        Ok(Self {
            dex_addresses: DexAddresses::new(),
            eth_pool,
            pancake_swap: build_pancake_swap()?,
            // db,
            // stablecoin_addresses: StableCoinAddresses::default(),
            erc_20: build_erc_20()?,
        })
    }
}
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum StableCoin {
    Usdc,
    Usdt,
    Busd,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum DexPath {
    /* every path for every token_in token_out pair in every dex in every chain must be recorded in the database */
    /* so that we can trigger our own trades in the futures */
    /* note that reciprocals are different pairs with different paths */
    /* i.e. the path for token_in x and token_out y is different from token_in y and token_out x */
    PancakeV2(Vec<H160>),
    PancakeV3SingleHop(PancakeV3SingleHopPath),
    PancakeV3MultiHop(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PancakeV3SingleHopPath {
    pub token_in: H160,
    pub token_out: H160,
    pub fee: U256,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum EnumBlockChain {
    EthereumMainnet,
    EthereumGoerli,
    BscMainnet,
    BscTestnet,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum EnumDexVersion {
    V1,
    V2,
    V3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum EnumDex {
    UniSwap,
    PancakeSwap,
    SushiSwap,
}

#[derive(Clone, Debug)]
pub struct Trade {
    pub chain: EnumBlockChain,
    pub contract: H160,
    pub dex: EnumDex,
    pub token_in: H160,
    pub token_out: H160,
    pub caller: H160,
    pub amount_in: U256,
    pub amount_out: U256,
    /* some trades go through multiple swap calls because of pool availability */
    /* this means that for some pairs, we must keep track of all swap calls made in order and their paths */
    pub swap_calls: Vec<ContractCall>,
    pub paths: Vec<DexPath>,
    pub dex_versions: Vec<EnumDexVersion>,
}

pub fn parse_quickalert_payload(payload: Bytes) -> Result<Vec<H256>> {
    let result: Result<Vec<H256>, _> = serde_json::from_slice(&payload);

    match result {
        Ok(hashes) => Ok(hashes),
        Err(e) => Err(e.into()),
    }
}
