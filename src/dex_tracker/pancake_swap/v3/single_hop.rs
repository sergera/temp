use eyre::*;
use web3::types::{H160, U256};

use crate::eth_sdk::ContractCall;

#[derive(Debug, Clone)]
pub struct ExactInputSingleParams {
    pub token_in: H160,
    pub token_out: H160,
    pub fee: U256,
    pub recipient: H160,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
    pub sqrt_price_limit_x96: U256,
}

pub fn exact_input_single(call: &ContractCall) -> Result<ExactInputSingleParams> {
    /*
                    function exactInputSingle(
                                    ExactInputSingleParams memory params
                    ) external payable override nonReentrant returns (uint256 amountOut)

                                                    struct ExactInputSingleParams {
                                                                    address tokenIn;
                                                                    address tokenOut;
                                                                    uint24 fee;
                                                                    address recipient;
                                                                    uint256 amountIn;
                                                                    uint256 amountOutMinimum;
                                                                    uint160 sqrtPriceLimitX96;
                                                    }
    */

    let params = call.get_param("params")?.get_value().into_tuple()?;

    Ok(ExactInputSingleParams {
        token_in: params[0].into_address()?,
        token_out: params[1].into_address()?,
        fee: params[2].into_uint()?,
        recipient: params[3].into_address()?,
        amount_in: params[4].into_uint()?,
        amount_out_minimum: params[5].into_uint()?,
        sqrt_price_limit_x96: params[6].into_uint()?,
    })
}

#[derive(Debug, Clone)]
pub struct ExactOutputSingleParams {
    pub token_in: H160,
    pub token_out: H160,
    pub fee: U256,
    pub recipient: H160,
    pub amount_out: U256,
    pub amount_in_maximum: U256,
    pub sqrt_price_limit_x96: U256,
}

pub fn exact_output_single(call: &ContractCall) -> Result<ExactOutputSingleParams> {
    /*
                    function exactOutputSingle(
                                    ExactOutputSingleParams calldata params
                    ) external payable override nonReentrant returns (uint256 amountIn)

                                                    struct ExactOutputSingleParams {
                                                                    address tokenIn;
                                                                    address tokenOut;
                                                                    uint24 fee;
                                                                    address recipient;
                                                                    uint256 amountOut;
                                                                    uint256 amountInMaximum;
                                                                    uint160 sqrtPriceLimitX96;
                                                    }
    */

    let params = call.get_param("params")?.get_value().into_tuple()?;

    Ok(ExactOutputSingleParams {
        token_in: params[0].into_address()?,
        token_out: params[1].into_address()?,
        fee: params[2].into_uint()?,
        recipient: params[3].into_address()?,
        amount_out: params[4].into_uint()?,
        amount_in_maximum: params[5].into_uint()?,
        sqrt_price_limit_x96: params[6].into_uint()?,
    })
}
