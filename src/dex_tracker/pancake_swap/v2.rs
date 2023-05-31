use eyre::*;
use web3::types::{H160, U256};

use crate::eth_sdk::ContractCall;

#[derive(Debug, Clone)]
pub struct SwapExactTokensForTokensParams {
    pub amount_in: U256,
    pub amount_out_min: U256,
    pub path: Vec<H160>,
    pub to: H160,
}

pub fn swap_exact_tokens_for_tokens(call: &ContractCall) -> Result<SwapExactTokensForTokensParams> {
    /*
            function swapExactTokensForTokens(
                                                    uint256 amountIn,
                                                    uint256 amountOutMin,
                                                    address[] calldata path,
                                                    address to
                    ) external payable returns (uint256 amountOut);
    */

    let path_result: Result<Vec<H160>> = call
        .get_param("path")?
        .get_value()
        .into_array()?
        .iter()
        .map(|token| token.into_address())
        .collect();
    let path = path_result?;

    Ok(SwapExactTokensForTokensParams {
        to: call.get_param("to")?.get_value().into_address()?,
        amount_in: call.get_param("amountIn")?.get_value().into_uint()?,
        amount_out_min: call.get_param("amountOutMin")?.get_value().into_uint()?,
        path: path,
    })
}

#[derive(Debug, Clone)]
pub struct SwapTokensForExactTokensParams {
    pub amount_out: U256,
    pub amount_in_max: U256,
    pub path: Vec<H160>,
    pub to: H160,
}

pub fn swap_tokens_for_exact_tokens(call: &ContractCall) -> Result<SwapTokensForExactTokensParams> {
    /*
            function swapTokensForExactTokens(
                                    uint256 amountOut,
                                    uint256 amountInMax,
                                    address[] calldata path,
                                    address to
            ) external payable override nonReentrant returns (uint256 amountIn)
    */

    let path_result: Result<Vec<H160>> = call
        .get_param("path")?
        .get_value()
        .into_array()?
        .iter()
        .map(|token| token.into_address())
        .collect();
    let path = path_result?;

    Ok(SwapTokensForExactTokensParams {
        to: call.get_param("to")?.get_value().into_address()?,
        amount_out: call.get_param("amountOut")?.get_value().into_uint()?,
        amount_in_max: call.get_param("amountInMax")?.get_value().into_uint()?,
        path: path,
    })
}
