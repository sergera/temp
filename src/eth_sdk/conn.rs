use eyre::*;
use std::sync::Arc;
use tokio::sync::Semaphore;
use web3::signing::Key;
use web3::transports::{Either, Http, WebSocket};
use web3::types::{
    Address, Transaction, TransactionId, TransactionParameters, TransactionReceipt, H256, U256,
};
use web3::Web3;

pub type EitherTransport = Either<WebSocket, Http>;

#[derive(Clone, Debug)]
pub struct EthereumRpcConnection {
    client: Web3<EitherTransport>,
    semaphore: Arc<Semaphore>,
}

impl EthereumRpcConnection {
    pub fn new(client: Web3<EitherTransport>, max_concurrent_requests: usize) -> Self {
        Self {
            client,
            semaphore: Arc::new(Semaphore::new(max_concurrent_requests)),
        }
    }

    pub async fn get_tx(&self, tx_hash: H256) -> Result<Option<Transaction>> {
        let permit = self.semaphore.acquire().await?;
        let tx_result = self
            .client
            .eth()
            .transaction(TransactionId::Hash(tx_hash))
            .await?;
        drop(permit);
        Ok(tx_result)
    }

    pub async fn get_receipt(&self, tx_hash: H256) -> Result<Option<TransactionReceipt>> {
        let permit = self.semaphore.acquire().await?;
        let receipt_result = self.client.eth().transaction_receipt(tx_hash).await?;
        drop(permit);
        Ok(receipt_result)
    }

    pub async fn transfer(&self, by: impl Key, to: Address, amount: U256) -> Result<H256> {
        let nonce = self
            .client
            .eth()
            .transaction_count(by.address(), None)
            .await?;
        let gas_price = self.client.eth().gas_price().await?;
        let gas_limit = 21000;
        let tx = TransactionParameters {
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            gas: gas_limit.into(),
            to: Some(to),
            value: amount.into(),
            ..Default::default()
        };
        let signed = self.client.accounts().sign_transaction(tx, by).await?;
        let tx_hash = self
            .client
            .eth()
            .send_raw_transaction(signed.raw_transaction)
            .await?;
        Ok(tx_hash)
    }

    pub async fn ping(&self) -> Result<()> {
        let _ = self.client.eth().block_number().await?;
        Ok(())
    }

    pub fn get_raw(&self) -> &Web3<EitherTransport> {
        &self.client
    }
    pub fn into_raw(self) -> Web3<EitherTransport> {
        self.client
    }
}
