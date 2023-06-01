use super::v2::{swap_exact_tokens_for_tokens, swap_tokens_for_exact_tokens};
use super::v3::{
    multi_hop::{exact_input, exact_output},
    single_hop::{exact_input_single, exact_output_single},
};

use crate::eth_sdk::{erc20::build_erc_20, ContractCall, SerializableToken, TransactionReady};
use crate::evm::{DexPath, PancakeV3SingleHopPath, Trade};
use crate::evm::{EnumBlockChain, EnumDex, EnumDexVersion};
use crate::v3::multi_hop::MultiHopPath;
use eyre::*;
use std::str::FromStr;
use web3::ethabi::Contract;
use web3::types::{H160, H256, U256};

pub struct Swap {
    pub recipient: H160,
    pub token_in: H160,
    pub token_out: H160,
    pub amount_in: Option<U256>,
    pub amount_out: Option<U256>,
    pub amount_out_minimum: Option<U256>,
    pub amount_in_maximum: Option<U256>,
    pub path: DexPath,
}

#[derive(Clone, Debug)]
pub struct PancakeSwap {
    smart_router: Contract,
    transfer_event_signature: H256,
    paid_in_native_flag: H160,
}

impl PancakeSwap {
    /* Parses Calls to the PancakeSwap V3 Smart Router into a Trade */
    /* https://etherscan.io/address/0x13f4EA83D0bd40E75C8222255bc855a974568Dd4#code */

    pub fn new(smart_router: Contract, transfer_event_signature: H256) -> Self {
        Self {
            smart_router,
            transfer_event_signature,
            paid_in_native_flag: H160::from_str("0x0000000000000000000000000000000000000002")
                .unwrap(),
        }
    }

    pub fn parse_trade(&self, tx: &TransactionReady, chain: EnumBlockChain) -> Result<Trade> {
        /* if tx is successful, all of the following should be Some */
        let value = tx.get_value();

        let caller = match tx.get_from() {
            Some(caller) => caller,
            None => {
                return Err(eyre!("failed to get caller"));
            }
        };

        let contract = match tx.get_to() {
            Some(contract) => contract,
            None => {
                return Err(eyre!("failed to get contract"));
            }
        };

        /* all swaps go through the "multicall" smart router function */
        let function_calls = self.get_multicall_funcs_and_params(tx)?;
        println!("");
        println!("tx: {:?}", tx.get_hash());
        println!("number of functions: {:?}", function_calls.len());
        let mut swap_infos: Vec<(Swap, EnumDexVersion, ContractCall)> = Vec::new();
        for call in function_calls {
            let method_name = call.get_name();
            println!("method name: {:?}", method_name);
            // println!("call: {:?}", call);
            if let Some(method) = self.get_method_by_name(&method_name) {
                swap_infos.push(match method {
                    /* V2 */
                    PancakeSwapMethod::SwapExactTokensForTokens => {
                        let swap_exact_tokens_for_tokens_params =
                            swap_exact_tokens_for_tokens(&call)?;
                        println!("params: {:?}", swap_exact_tokens_for_tokens_params);
                        let swap = Swap {
                            recipient: swap_exact_tokens_for_tokens_params.to,
                            token_in: swap_exact_tokens_for_tokens_params.path[0],
                            token_out: swap_exact_tokens_for_tokens_params.path
                                [swap_exact_tokens_for_tokens_params.path.len() - 1],
                            amount_in: Some(swap_exact_tokens_for_tokens_params.amount_in),
                            amount_out: None,
                            amount_out_minimum: Some(
                                swap_exact_tokens_for_tokens_params.amount_out_min,
                            ),
                            amount_in_maximum: None,
                            path: DexPath::PancakeV2(swap_exact_tokens_for_tokens_params.path),
                        };
                        (swap, EnumDexVersion::V2, call)
                    }
                    PancakeSwapMethod::SwapTokensForExactTokens => {
                        let swap_tokens_for_exact_tokens_params =
                            swap_tokens_for_exact_tokens(&call)?;
                        println!("params: {:?}", swap_tokens_for_exact_tokens_params);
                        let swap = Swap {
                            recipient: swap_tokens_for_exact_tokens_params.to,
                            token_in: swap_tokens_for_exact_tokens_params.path[0],
                            token_out: swap_tokens_for_exact_tokens_params.path
                                [swap_tokens_for_exact_tokens_params.path.len() - 1],
                            amount_in: None,
                            amount_out: Some(swap_tokens_for_exact_tokens_params.amount_out),
                            amount_out_minimum: None,
                            amount_in_maximum: Some(
                                swap_tokens_for_exact_tokens_params.amount_in_max,
                            ),
                            path: DexPath::PancakeV2(swap_tokens_for_exact_tokens_params.path),
                        };

                        (swap, EnumDexVersion::V2, call)
                    }
                    /* V3 */
                    PancakeSwapMethod::ExactInputSingle => {
                        let exact_input_single_params = exact_input_single(&call)?;
                        println!("params: {:?}", exact_input_single_params);
                        let swap = Swap {
                            recipient: exact_input_single_params.recipient,
                            token_in: exact_input_single_params.token_in,
                            token_out: exact_input_single_params.token_out,
                            amount_in: Some(exact_input_single_params.amount_in),
                            amount_out: None,
                            amount_out_minimum: Some(exact_input_single_params.amount_out_minimum),
                            amount_in_maximum: None,
                            path: DexPath::PancakeV3SingleHop(PancakeV3SingleHopPath {
                                token_in: exact_input_single_params.token_in,
                                token_out: exact_input_single_params.token_out,
                                fee: exact_input_single_params.fee,
                            }),
                        };
                        (swap, EnumDexVersion::V3, call)
                    }
                    PancakeSwapMethod::ExactOutputSingle => {
                        let exact_output_single_params = exact_output_single(&call)?;
                        println!("params: {:?}", exact_output_single_params);
                        let swap = Swap {
                            recipient: exact_output_single_params.recipient,
                            token_in: exact_output_single_params.token_in,
                            token_out: exact_output_single_params.token_out,
                            amount_in: None,
                            amount_out: Some(exact_output_single_params.amount_out),
                            amount_out_minimum: None,
                            amount_in_maximum: Some(exact_output_single_params.amount_in_maximum),
                            path: DexPath::PancakeV3SingleHop(PancakeV3SingleHopPath {
                                token_in: exact_output_single_params.token_in,
                                token_out: exact_output_single_params.token_out,
                                fee: exact_output_single_params.fee,
                            }),
                        };
                        (swap, EnumDexVersion::V3, call)
                    }
                    PancakeSwapMethod::ExactInput => {
                        let exact_input_params = exact_input(&call)?;
                        let full_path = MultiHopPath::from_bytes(&exact_input_params.path)?;
                        println!("params: {:?}", exact_input_params);
                        println!("full path: {:?}", full_path);
                        let swap = Swap {
                            recipient: exact_input_params.recipient,
                            token_in: full_path[0].first_token,
                            token_out: full_path[full_path.len() - 1].second_token,
                            amount_in: Some(exact_input_params.amount_in),
                            amount_out: None,
                            amount_out_minimum: Some(exact_input_params.amount_out_minimum),
                            amount_in_maximum: None,
                            path: DexPath::PancakeV3MultiHop(exact_input_params.path.to_vec()),
                        };
                        (swap, EnumDexVersion::V3, call)
                    }
                    PancakeSwapMethod::ExactOutput => {
                        let exact_output_params = exact_output(&call)?;
                        let full_path = MultiHopPath::from_bytes(&exact_output_params.path)?;
                        println!("params: {:?}", exact_output_params);
                        println!("full path: {:?}", full_path);
                        let swap = Swap {
                            recipient: exact_output_params.recipient,
                            token_in: full_path[full_path.len() - 1].second_token,
                            token_out: full_path[0].first_token,
                            amount_in: None,
                            amount_out: Some(exact_output_params.amount_out),
                            amount_out_minimum: None,
                            amount_in_maximum: Some(exact_output_params.amount_in_maximum),
                            path: DexPath::PancakeV3MultiHop(exact_output_params.path.to_vec()),
                        };
                        (swap, EnumDexVersion::V3, call)
                    }
                });
            }
        }

        if swap_infos.is_empty() {
            return Err(eyre!("no suitable method found"));
        }

        let mut paths: Vec<DexPath> = Vec::new();
        let mut versions: Vec<EnumDexVersion> = Vec::new();
        let mut calls: Vec<ContractCall> = Vec::new();
        for (swap, version, call) in &mut swap_infos {
            paths.push(swap.path.clone());
            versions.push(*version);
            calls.push(call.clone());
            if swap.amount_out.is_none() {
                /* amount out missing */
                if swap.recipient == self.paid_in_native_flag {
                    /* user got paid in native tokens, transfer is from token_out to router */
                    let amount_out = tx
                        .amount_of_token_received(
                            swap.token_out,
                            contract,
                            self.transfer_event_signature,
                        )
                        .map_err(|err| eyre!("failed to get amount_out: {}", err))?;
                    swap.amount_out = Some(amount_out);
                } else {
                    /* user got paid in token_out, transfer is from token_out to recipient */
                    let amount_out = tx
                        .amount_of_token_received(
                            swap.token_out,
                            swap.recipient,
                            self.transfer_event_signature,
                        )
                        .map_err(|err| eyre!("failed to get amount_out: {}", err))?;
                    swap.amount_out = Some(amount_out);
                }
            } else {
                /* amount in missing */
                if value != 0.into() {
                    /* user paid in native tokens, transfer is from router to pool */
                    /* because the router first wrapped the token, in order to use pool */
                    let amount_in = tx
                        .amount_of_token_sent(
                            swap.token_in,
                            contract,
                            self.transfer_event_signature,
                        )
                        .map_err(|err| eyre!("failed to get amount_in: {}", err))?;
                    swap.amount_in = Some(amount_in);
                } else {
                    /* user paid in token_in, transfer is from user to pool */
                    let amount_in = tx
                        .amount_of_token_sent(swap.token_in, caller, self.transfer_event_signature)
                        .map_err(|err| eyre!("failed to get amount_in: {}", err))?;
                    swap.amount_in = Some(amount_in);
                }
            }
        }

        Ok(Trade {
            chain,
            contract,
            dex: EnumDex::PancakeSwap,
            token_in: swap_infos[0].0.token_in,
            token_out: swap_infos[swap_infos.len() - 1].0.token_out,
            caller,
            amount_in: swap_infos[0].0.amount_in.unwrap(),
            amount_out: swap_infos[swap_infos.len() - 1].0.amount_out.unwrap(),
            swap_calls: calls,
            paths,
            dex_versions: versions,
        })
    }

    fn get_multicall_funcs_and_params(&self, tx: &TransactionReady) -> Result<Vec<ContractCall>> {
        /*
                        function multicall(
                                bytes[] calldata data
                        ) public payable override returns (bytes[] memory results);
        */
        let multicall_input_data = tx.get_input_data();

        let multicall = ContractCall::from_inputs(&self.smart_router, &multicall_input_data)?;

        let mut actual_function_calls: Vec<ContractCall> = Vec::new();
        /* the single parameter from "multicall" is ambiguously called "data" */
        if let Ok(param) = multicall.get_param("data") {
            /* data is an unsized array of byte arrays */
            let value_array = match param.get_value() {
                SerializableToken::Array(value) => value,
                _ => {
                    return Err(eyre!("data is not an array"));
                }
            };

            for token in value_array {
                /* each byte array is a nested function call */
                let input_data = match token.into_bytes() {
                    Ok(input_data) => input_data,
                    Err(_) => {
                        return Err(eyre!("failed to get input data"));
                    }
                };
                let function_call = ContractCall::from_inputs(&self.smart_router, &input_data)?;
                actual_function_calls.push(function_call);
            }
        }

        Ok(actual_function_calls)
    }

    fn get_method_by_name(&self, name: &str) -> Option<PancakeSwapMethod> {
        match name {
            "swapExactTokensForTokens" => Some(PancakeSwapMethod::SwapExactTokensForTokens),
            "swapTokensForExactTokens" => Some(PancakeSwapMethod::SwapTokensForExactTokens),
            "exactInputSingle" => Some(PancakeSwapMethod::ExactInputSingle),
            "exactInput" => Some(PancakeSwapMethod::ExactInput),
            "exactOutputSingle" => Some(PancakeSwapMethod::ExactOutputSingle),
            "exactOutput" => Some(PancakeSwapMethod::ExactOutput),
            _ => None,
        }
    }
}

enum PancakeSwapMethod {
    SwapExactTokensForTokens,
    SwapTokensForExactTokens,
    ExactInputSingle,
    ExactInput,
    ExactOutputSingle,
    ExactOutput,
}

const PANCAKE_SMART_ROUTER_PATH: &str = "abi/pancake_swap/smart_router_v3.json";

pub fn build_pancake_swap() -> Result<PancakeSwap> {
    let pancake_smart_router = Contract::load(
        std::fs::File::open(PANCAKE_SMART_ROUTER_PATH).context("failed to read contract ABI")?,
    )
    .context("failed to parse contract ABI")?;
    let erc20 = build_erc_20()?;
    let transfer_event_signature = erc20
        .event("Transfer")
        .context("Failed to get Transfer event signature")?
        .signature();
    let pancake = PancakeSwap::new(pancake_smart_router, transfer_event_signature);
    Ok(pancake)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::eth_sdk::{EthereumRpcConnectionPool, Transaction};
    use crate::evm::EnumBlockChain;

    #[tokio::test]
    async fn test_pancakeswap() -> Result<()> {
        let pancake = build_pancake_swap()?;
        let conn_pool = EthereumRpcConnectionPool::mainnet();
        let conn = conn_pool.get_conn().await?;
        let tx = Transaction::new_and_assume_ready(
            "0x750d90bf90ad0fe7d035fbbab41334f6bb10bf7e71246d430cb23ed35d1df7c2".parse()?,
            &conn,
        )
        .await?;

        let trade = pancake.parse_trade(&tx, EnumBlockChain::EthereumMainnet)?;
        println!("trade: {:?}", trade);
        Ok(())
    }
}
