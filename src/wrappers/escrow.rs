use web3::types::{
    Address, BlockNumber, Bytes, CallRequest, SignedTransaction, TransactionParameters, H256, U256,
};

use web3::contract::Contract;
use web3::Transport;

#[derive(Debug, Clone)]
pub struct EscrowContract<T: Transport> {
    inner: Contract<T>,
    address: Address,
}
