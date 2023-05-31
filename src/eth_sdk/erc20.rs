// use super::signer::EthereumSigner;
// use web3::signing::Key;
use super::utils::{eth_public_exponent_to_address, wait_for_confirmations_simple, wei_to_eth};
use super::{EitherTransport, EthereumNet};
use crate::crypto;
use crate::crypto::Signer;
use crate::token::CryptoToken;
use eyre::*;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use web3::api::Web3;
use web3::contract::{Contract, Options};
use web3::signing::Key;
use web3::types::{Address, H256, U256};

pub const ERC20_ABI: &'static str = include_str!("erc20.abi.json");

pub struct Erc20Token {
    client: Web3<EitherTransport>,
    net: EthereumNet,
    pub address: Address,
    pub contract: Contract<EitherTransport>,
}

impl Erc20Token {
    pub fn new(client: Web3<EitherTransport>, contract: Contract<EitherTransport>) -> Result<Self> {
        Ok(Self {
            client,
            net: EthereumNet::Mainnet,
            address: contract.address(),
            contract,
        })
    }

    pub async fn mint(&self, secret: impl Key, to: Address, amount: U256) -> Result<H256> {
        Ok(self
            .contract
            .signed_call("mint", (to, amount), Options::default(), secret)
            .await?)
    }

    pub async fn burn(&self, secret: impl Key, from: Address, amount: U256) -> Result<H256> {
        Ok(self
            .contract
            .signed_call("burn", (from, amount), Options::default(), secret)
            .await?)
    }

    pub async fn transfer(&self, secret: impl Key, to: Address, amount: U256) -> Result<H256> {
        Ok(self
            .contract
            .signed_call("transfer", (to, amount), Options::default(), secret)
            .await?)
    }

    pub async fn transfer_from(
        &self,
        secret: impl Key,
        from: Address,
        to: Address,
        amount: U256,
    ) -> Result<H256> {
        Ok(self
            .contract
            .signed_call(
                "transferFrom",
                (from, to, amount),
                Options::default(),
                secret,
            )
            .await?)
    }

    pub async fn approve(&self, secret: impl Key, spender: Address, amount: U256) -> Result<H256> {
        Ok(self
            .contract
            .signed_call("approve", (spender, amount), Options::default(), secret)
            .await?)
    }

    pub async fn balance_of(&self, owner: Address) -> Result<U256> {
        Ok(self
            .contract
            .query("balanceOf", owner, None, Options::default(), None)
            .await?)
    }

    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<U256> {
        Ok(self
            .contract
            .query(
                "allowance",
                (owner, spender),
                None,
                Options::default(),
                None,
            )
            .await?)
    }

    pub async fn total_supply(&self) -> Result<U256> {
        Ok(self
            .contract
            .query("totalSupply", (), None, Options::default(), None)
            .await?)
    }
}

impl Debug for Erc20Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ERC20Token")
            .field("net", &self.net)
            .field("address", &self.address)
            .finish()
    }
}

pub fn build_erc_20() -> Result<web3::ethabi::Contract> {
    Ok(web3::ethabi::Contract::load(ERC20_ABI.as_bytes())
        .context("failed to parse contract ABI")?)
}
