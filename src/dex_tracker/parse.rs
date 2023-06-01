use crate::eth_sdk::TransactionReady;
use eyre::*;

use super::pancake_swap::PancakeSwap;
use crate::evm::Trade;
use crate::evm::{EnumBlockChain, EnumDex};
use crate::DexAddresses;

pub async fn parse_dex_trade(
    chain: EnumBlockChain,
    tx: &TransactionReady,
    dex_addresses: &DexAddresses,
    pancake_swap: &PancakeSwap,
) -> Result<Trade> {
    let called_contract = tx.get_to().context("no called contract")?;
    let eth_mainnet_dexes = dex_addresses.get(&chain).unwrap();
    for (dex, address) in eth_mainnet_dexes {
        if *address == called_contract {
            let trade = match dex {
                EnumDex::PancakeSwap => pancake_swap.parse_trade(tx, chain.clone())?,
                EnumDex::UniSwap => {
                    bail!("does not support dex: UniSwap");
                }
                EnumDex::SushiSwap => {
                    bail!("does not support dex: SushiSwap");
                }
            };
            // println!("");
            // println!("tx: {:?}", tx.get_hash());
            println!("trade: {:?}", trade);
            return Ok(trade);
        }
    }
    bail!("no dex found for this tx");
}
