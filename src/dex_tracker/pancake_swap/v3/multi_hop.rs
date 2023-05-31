use eyre::*;
use web3::types::{H160, U256};

use web3::ethabi::Token;

use crate::eth_sdk::ContractCall;

#[derive(Debug, Clone)]
pub struct MultiHopPath {
    pub first_token: H160,
    pub fee: U256,
    pub second_token: H160,
}

impl MultiHopPath {
    pub fn from_bytes(path: &[u8]) -> Result<Vec<Self>> {
        if path.len() < 43 {
            /* 20 bytes for address, 3 bytes for uint24, 20 bytes for address */
            bail!("path is too short");
        }

        let mut full_path: Vec<MultiHopPath> = Vec::new();
        let mut first_token: H160 = H160::from_slice(&path[0..20]);
        for i in 0..((path.len() - 20) / 23) {
            let start = 20 + i * 23;
            if start + 23 > path.len() {
                bail!("path does not have enough bytes for reading next path entry");
            }

            let fee_bytes: [u8; 3] = match path[start..start + 3].try_into() {
                Ok(bytes) => bytes,
                Err(e) => {
                    bail!(
                        "error parsing 'path' from PancakeSwap exactInput call: {}",
                        e
                    );
                }
            };
            let fee = U256::from(u32::from_be_bytes([
                0,
                fee_bytes[0],
                fee_bytes[1],
                fee_bytes[2],
            ]));
            let second_token: H160 = H160::from_slice(&path[start + 3..start + 23]);
            full_path.push(MultiHopPath {
                first_token,
                fee,
                second_token,
            });
            first_token = second_token;
        }
        Ok(full_path)
    }

    pub fn to_bytes(paths: &Vec<Self>) -> Result<Vec<u8>> {
        if paths.is_empty() {
            bail!("paths is empty");
        }
        let max_fee = U256::from(2).pow(U256::from(24)) - U256::from(1);
        let mut res: Vec<u8> = Vec::new();
        for (i, path) in paths.iter().enumerate() {
            if path.fee > max_fee {
                // fee can't be larger than max value for uint24
                bail!("Fee is larger than the maximum value for uint24");
            }
            if i == 0 {
                res.extend_from_slice(&path.first_token.0);
            }
            let mut buffer = [0u8; 32];
            path.fee.to_big_endian(&mut buffer);
            let fee_bytes = &buffer[29..32];
            res.extend_from_slice(fee_bytes);
            res.extend_from_slice(&path.second_token.0);
        }
        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub struct ExactInputParams {
    pub path: Vec<u8>,
    pub recipient: H160,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
}

pub fn exact_input(call: &ContractCall) -> Result<ExactInputParams> {
    /*
                    function exactInput(
                                    ExactInputParams memory params
                    ) external payable nonReentrant override returns (uint256 amountOut)

                                                    struct ExactInputParams {
                                                                    bytes path;
                                                                    address recipient;
                                                                    uint256 amountIn;
                                                                    uint256 amountOutMinimum;
                                                    }
    */

    let params = call.get_param("params")?.get_value().into_tuple()?;

    Ok(ExactInputParams {
        path: params[0].into_bytes()?,
        recipient: params[1].into_address()?,
        amount_in: params[2].into_uint()?,
        amount_out_minimum: params[3].into_uint()?,
    })
}

#[derive(Debug, Clone)]
pub struct ExactOutputParams {
    pub path: Vec<u8>,
    pub recipient: H160,
    pub amount_out: U256,
    pub amount_in_maximum: U256,
}

pub fn exact_output(call: &ContractCall) -> Result<ExactOutputParams> {
    /*
                    function exactOutput(
                                    ExactOutputParams calldata params
                    ) external payable override nonReentrant returns (uint256 amountIn)

                                                    struct ExactOutputParams {
                                                                    bytes path;
                                                                    address recipient;
                                                                    uint256 amountOut;
                                                                    uint256 amountInMaximum;
                                                    }
    */

    let params = call.get_param("params")?.get_value().into_tuple()?;

    Ok(ExactOutputParams {
        path: params[0].into_bytes()?,
        recipient: params[1].into_address()?,
        amount_out: params[2].into_uint()?,
        amount_in_maximum: params[3].into_uint()?,
    })
}
