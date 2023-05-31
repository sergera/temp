use crate::crypto;
use crate::crypto::Signer;
use crate::token::CryptoToken;
use eyre::*;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use utils::{wait_for_confirmations_simple, wei_to_eth};
use web3::signing::Key;
use web3::types::{Address, TransactionParameters, TransactionRequest, H256, U256};
use web3::Web3;

mod calldata;
mod conn;
pub mod contract;
pub mod erc20;
mod pool;
mod tx;
pub mod utils;

pub use calldata::*;
pub use conn::*;
pub use pool::*;
pub use tx::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EthereumNet {
    Mainnet,
    Ropsten,
    Rinkeby,
    Goerli,
    Kovan,
    Local,
}

#[derive(Clone)]
pub struct EthereumToken {
    pub client: Web3<EitherTransport>,
    pub net: EthereumNet,
}
impl EthereumToken {
    pub async fn new(net: EthereumNet) -> Result<Self> {
        let url = match net {
            EthereumNet::Mainnet => "https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Ropsten => "https://ropsten.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Rinkeby => "https://rinkeby.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Goerli => "https://goerli.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Kovan => "https://kovan.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Local => "http://localhost:8545",
        };
        let transport = new_transport(url).await?;
        let client = Web3::new(transport);
        Ok(EthereumToken { client, net })
    }

    pub async fn get_accounts(&self) -> Result<Vec<Address>> {
        let accounts = self.client.eth().accounts().await?;

        Ok(accounts)
    }
    pub async fn transfer_debug(&self, from: Address, to: Address, amount: f64) -> Result<String> {
        let amount = (amount * 1e18) as u64;
        let nonce = self.client.eth().transaction_count(from, None).await?;
        let gas_price = self.client.eth().gas_price().await?;
        let tx = TransactionRequest {
            from: from,
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            to: Some(to),
            value: Some(amount.into()),
            ..Default::default()
        };
        let tx_hash = self.client.eth().send_transaction(tx).await?;
        Ok(format!("{:?}", tx_hash))
    }
}
impl Debug for EthereumToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthereumToken")
            .field("net", &self.net)
            .finish()
    }
}
