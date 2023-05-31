use super::dex_tracker::v3::multi_hop::MultiHopPath;
use super::evm::Trade;
use crate::dex_tracker::{
    v2::swap_exact_tokens_for_tokens, v2::swap_tokens_for_exact_tokens, v3::multi_hop::exact_input,
    v3::multi_hop::exact_output, v3::single_hop::exact_input_single,
    v3::single_hop::exact_output_single,
};
use crate::eth_sdk::utils::wait_for_confirmations_simple;
use crate::eth_sdk::ContractCall;
use crate::evm::DexPath;
use eyre::*;
use std::str::FromStr;
use std::time::Duration;
use web3::api::{Eth, Namespace};
use web3::contract::{Contract, Options};
use web3::ethabi::Token;

use web3::signing::{Key, SecretKeyRef};
use web3::types::{Address, BlockNumber, FilterBuilder, H256, U256};
use web3::{Transport, Web3};

const SMART_ROUTER_ABI_JSON: &str = include_str!("../abi/pancake_swap/smart_router_v3.json");

#[derive(Debug, Clone)]
pub struct PancakeSmartRouterV3Contract<T: Transport> {
    contract: Contract<T>,
    w3: Web3<T>,
}

impl<T: Transport> PancakeSmartRouterV3Contract<T> {
    pub fn new(w3: Web3<T>, address: Address) -> Result<Self> {
        let contract = Contract::from_json(w3.eth(), address, SMART_ROUTER_ABI_JSON.as_bytes())?;
        Ok(Self { contract, w3 })
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn copy_trade(
        &self,
        signer: impl Key + Clone,
        trade: Trade,
        amount_in: Option<U256>,
        amount_out: Option<U256>,
    ) -> Result<Trade> {
        let recipient = signer.address();
        let mut call: ContractCall;
        let mut path: DexPath;
        for i in 0..trade.swap_calls.len() {
            call = trade.swap_calls[i].clone();
            path = trade.paths[i].clone();
            println!("");
            println!("function called: {:?}", call.get_name());
            match PancakeSmartRouterV3Functions::from_str(&call.get_name())? {
                PancakeSmartRouterV3Functions::SwapExactTokensForTokens => {
                    let params = swap_exact_tokens_for_tokens(&call)?;
                    let amount_in = params.amount_in;
                    let amount_out_min = params.amount_out_min;
                    let path = params.path;
                    let to = params.to;

                    self.swap_exact_tokens_for_tokens(
                        signer.clone(),
                        to,
                        amount_in,
                        amount_out_min,
                        path,
                    )
                    .await?;
                }
                PancakeSmartRouterV3Functions::SwapTokensForExactTokens => {
                    let params = swap_tokens_for_exact_tokens(&call)?;
                    let amount_out = params.amount_out;
                    let amount_in_max = params.amount_in_max;
                    let path = params.path;
                    let to = params.to;

                    self.swap_tokens_for_exact_tokens(
                        signer.clone(),
                        to,
                        amount_out,
                        amount_in_max,
                        path,
                    )
                    .await?;
                }
                PancakeSmartRouterV3Functions::ExactInputSingle => {
                    let params = exact_input_single(&call)?;
                    let token_in = params.token_in;
                    let token_out = params.token_out;
                    let fee = params.fee;
                    let recipient = params.recipient;
                    let amount_in = params.amount_in;
                    let amount_out_minimum = params.amount_out_minimum;

                    self.exact_input_single(
                        signer.clone(),
                        token_in,
                        token_out,
                        fee,
                        recipient,
                        amount_in,
                        amount_out_minimum,
                    )
                    .await?;
                }
                PancakeSmartRouterV3Functions::ExactOutputSingle => {
                    let params = exact_output_single(&call)?;
                    let token_in = params.token_in;
                    let token_out = params.token_out;
                    let fee = params.fee;
                    let recipient = params.recipient;
                    let amount_out = params.amount_out;
                    let amount_in_maximum = params.amount_in_maximum;

                    self.exact_output_single(
                        signer.clone(),
                        token_in,
                        token_out,
                        fee,
                        recipient,
                        amount_out,
                        amount_in_maximum,
                    )
                    .await?;
                }
                PancakeSmartRouterV3Functions::ExactInput => {
                    let params = exact_input(&call)?;
                    let path = params.path;
                    let recipient = params.recipient;
                    let amount_in = params.amount_in;
                    let amount_out_minimum = params.amount_out_minimum;

                    println!("before decoding multihoppath");
                    self.exact_input(
                        signer.clone(),
                        MultiHopPath::from_bytes(&path)?,
                        recipient,
                        amount_in,
                        amount_out_minimum,
                    )
                    .await?;
                }
                PancakeSmartRouterV3Functions::ExactOutput => {
                    let params = exact_output(&call)?;
                    let path = params.path;
                    let recipient = params.recipient;
                    let amount_out = params.amount_out;
                    let amount_in_maximum = params.amount_in_maximum;

                    self.exact_output(
                        signer.clone(),
                        MultiHopPath::from_bytes(&path)?,
                        recipient,
                        amount_out,
                        amount_in_maximum,
                    )
                    .await?;
                }
            }
        }

        Ok(trade)
    }

    pub async fn swap_exact_tokens_for_tokens(
        &self,
        signer: impl Key,
        recipient: Address,
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
    ) -> Result<H256> {
        let params = (amount_in, amount_out_min, path, recipient);
        let estimated_gas = self
            .contract
            .estimate_gas(
                PancakeSmartRouterV3Functions::SwapExactTokensForTokens.as_str(),
                params.clone(),
                signer.address(),
                Options::default(),
            )
            .await?;
        // let tx = self
        //     .contract
        //     .signed_call(
        //         PancakeSmartRouterV3Functions::SwapExactTokensForTokens.as_str(),
        //         params,
        //         Options::with(|options| options.gas = Some(estimated_gas)),
        //         signer,
        //     )
        //     .await?;
        // wait_for_confirmations_simple(&self.w3.eth(), tx, Duration::from_secs(3), 5).await?;
        // Ok(tx)
        Ok(H256::zero())
    }

    pub async fn swap_tokens_for_exact_tokens(
        &self,
        signer: impl Key,
        recipient: Address,
        amount_out: U256,
        amount_in_max: U256,
        path: Vec<Address>,
    ) -> Result<H256> {
        let params = (amount_out, amount_in_max, path, recipient);
        let estimated_gas = self
            .contract
            .estimate_gas(
                PancakeSmartRouterV3Functions::SwapTokensForExactTokens.as_str(),
                params.clone(),
                signer.address(),
                Options::default(),
            )
            .await?;
        // let tx = self
        //     .contract
        //     .signed_call(
        //         PancakeSmartRouterV3Functions::SwapTokensForExactTokens.as_str(),
        //         params,
        //         Options::with(|options| options.gas = Some(estimated_gas)),
        //         signer,
        //     )
        //     .await?;
        // wait_for_confirmations_simple(&self.w3.eth(), tx, Duration::from_secs(3), 5).await?;
        // Ok(tx)
        Ok(H256::zero())
    }

    pub async fn exact_input_single(
        &self,
        signer: impl Key,
        token_in: Address,
        token_out: Address,
        fee: U256,
        recipient: Address,
        amount_in: U256,
        amount_out_minimum: U256,
    ) -> Result<H256> {
        // fee is a Solidity uint24
        let max_uint24: U256 = U256::from(2).pow(24.into()) - U256::from(1);
        if fee > max_uint24 {
            return Err(eyre::eyre!(
                "The fee exceeds the maximum value for a uint24"
            ));
        }

        let params = Token::Tuple(vec![
            Token::Address(token_in),
            Token::Address(token_out),
            Token::Uint(fee),
            Token::Address(recipient),
            Token::Uint(amount_in),
            Token::Uint(amount_out_minimum),
            Token::Uint(U256::from(0)),
        ]);
        let estimated_gas = self
            .contract
            .estimate_gas(
                PancakeSmartRouterV3Functions::ExactInputSingle.as_str(),
                params.clone(),
                signer.address(),
                Options::default(),
            )
            .await?;
        // let tx = self
        //     .contract
        //     .signed_call(
        //         PancakeSmartRouterV3Functions::ExactInputSingle.as_str(),
        //         params,
        //         Options::with(|options| options.gas = Some(estimated_gas)),
        //         signer,
        //     )
        //     .await?;
        // wait_for_confirmations_simple(&self.w3.eth(), tx, Duration::from_secs(3), 5).await?;
        // Ok(tx)
        Ok(H256::zero())
    }

    pub async fn exact_output_single(
        &self,
        signer: impl Key,
        token_in: Address,
        token_out: Address,
        fee: U256,
        recipient: Address,
        amount_out: U256,
        amount_in_maximum: U256,
    ) -> Result<H256> {
        let max_uint24: U256 = U256::from(2).pow(24.into()) - U256::from(1);
        if fee > max_uint24 {
            return Err(eyre::eyre!(
                "The fee exceeds the maximum value for a uint24"
            ));
        }

        let params = Token::Tuple(vec![
            Token::Address(token_in),
            Token::Address(token_out),
            Token::Uint(fee),
            Token::Address(recipient),
            Token::Uint(amount_out),
            Token::Uint(amount_in_maximum),
            Token::Uint(U256::from(0)),
        ]);
        let estimated_gas = self
            .contract
            .estimate_gas(
                PancakeSmartRouterV3Functions::ExactOutputSingle.as_str(),
                params.clone(),
                signer.address(),
                Options::default(),
            )
            .await?;
        // let tx = self
        //     .contract
        //     .signed_call(
        //         PancakeSmartRouterV3Functions::ExactOutputSingle.as_str(),
        //         params,
        //         Options::with(|options| options.gas = Some(estimated_gas)),
        //         signer,
        //     )
        //     .await?;
        // wait_for_confirmations_simple(&self.w3.eth(), tx, Duration::from_secs(3), 5).await?;
        // Ok(tx)
        Ok(H256::zero())
    }

    pub async fn exact_input(
        &self,
        signer: impl Key,
        path: Vec<MultiHopPath>,
        recipient: Address,
        amount_in: U256,
        amount_out_minimum: U256,
    ) -> Result<H256> {
        println!("before encoding params");
        let params = Token::Tuple(vec![
            Token::Bytes(MultiHopPath::to_bytes(&path)?),
            Token::Address(recipient),
            Token::Uint(amount_in),
            Token::Uint(amount_out_minimum),
        ]);
        println!("before estimating gas");
        let estimated_gas = self
            .contract
            .estimate_gas(
                PancakeSmartRouterV3Functions::ExactInput.as_str(),
                params.clone(),
                signer.address(),
                Options::default(),
            )
            .await?;
        // let tx = self
        //     .contract
        //     .signed_call(
        //         PancakeSmartRouterV3Functions::ExactInput.as_str(),
        //         params,
        //         Options::with(|options| options.gas = Some(estimated_gas)),
        //         signer,
        //     )
        //     .await?;
        // wait_for_confirmations_simple(&self.w3.eth(), tx, Duration::from_secs(3), 5).await?;
        // Ok(tx)
        Ok(H256::zero())
    }

    pub async fn exact_output(
        &self,
        signer: impl Key,
        path: Vec<MultiHopPath>,
        recipient: Address,
        amount_out: U256,
        amount_in_maximum: U256,
    ) -> Result<H256> {
        let params = Token::Tuple(vec![
            Token::Bytes(MultiHopPath::to_bytes(&path)?),
            Token::Address(recipient),
            Token::Uint(amount_out),
            Token::Uint(amount_in_maximum),
        ]);
        let estimated_gas = self
            .contract
            .estimate_gas(
                PancakeSmartRouterV3Functions::ExactOutput.as_str(),
                params.clone(),
                signer.address(),
                Options::default(),
            )
            .await?;
        // let tx = self
        //     .contract
        //     .signed_call(
        //         PancakeSmartRouterV3Functions::ExactOutput.as_str(),
        //         params,
        //         Options::with(|options| options.gas = Some(estimated_gas)),
        //         signer,
        //     )
        //     .await?;
        // wait_for_confirmations_simple(&self.w3.eth(), tx, Duration::from_secs(3), 5).await?;
        // Ok(tx)
        Ok(H256::zero())
    }
}

enum PancakeSmartRouterV3Functions {
    SwapExactTokensForTokens,
    SwapTokensForExactTokens,
    ExactInputSingle,
    ExactInput,
    ExactOutputSingle,
    ExactOutput,
}

impl PancakeSmartRouterV3Functions {
    fn as_str(&self) -> &'static str {
        match self {
            Self::SwapExactTokensForTokens => "swapExactTokensForTokens",
            Self::SwapTokensForExactTokens => "swapTokensForExactTokens",
            Self::ExactInputSingle => "exactInputSingle",
            Self::ExactInput => "exactInput",
            Self::ExactOutputSingle => "exactOutputSingle",
            Self::ExactOutput => "exactOutput",
        }
    }

    fn from_str(function: &str) -> Result<Self> {
        match function {
            "swapExactTokensForTokens" => Ok(Self::SwapExactTokensForTokens),
            "swapTokensForExactTokens" => Ok(Self::SwapTokensForExactTokens),
            "exactInputSingle" => Ok(Self::ExactInputSingle),
            "exactInput" => Ok(Self::ExactInput),
            "exactOutputSingle" => Ok(Self::ExactOutputSingle),
            "exactOutput" => Ok(Self::ExactOutput),
            _ => bail!("invalid function name"),
        }
    }
}
