use crate::crypto::{PublicExponent, Signer};
use eyre::*;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait CryptoToken: Any + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn get_network_type(&self) -> String;
    fn convert_internal_unit_to_display_unit(&self, amount: &str) -> Result<String>;
    fn convert_display_unit_to_internal_unit(&self, amount: &str) -> Result<String>;
    fn public_exponent_to_address(&self, public_exponent: &PublicExponent) -> Result<String>;
    fn address_to_public_exponent(&self, address: &str) -> Result<PublicExponent>;
    fn get_address_explorer_url(&self, address: &str) -> String;
    fn get_transaction_explorer_url(&self, address: &str) -> String;
    async fn transfer(
        &self,
        fee_payer: Arc<dyn Signer>,
        by: Arc<dyn Signer>,
        from: &str,
        to: &str,
        amount: &str,
    ) -> Result<String> {
        bail!("Not supported")
    }
    async fn request_airdrop(&self, addr: &str, amount: &str) -> Result<String> {
        bail!("Not supported")
    }
    async fn get_balance(&self, addr: &str) -> Result<String> {
        bail!("Not supported")
    }
    async fn create_account(
        &self,
        fee_payer: Arc<dyn Signer>,
        owner: &str,
        account: Arc<dyn Signer>,
    ) -> Result<String> {
        bail!("Not supported")
    }
    async fn mint_to(
        &self,
        fee_payer: Arc<dyn Signer>,
        minter: Arc<dyn Signer>,
        account: &str,
        amount: &str,
    ) -> Result<String> {
        bail!("Not supported")
    }
    async fn get_latest_blockhash(&self) -> Result<String> {
        bail!("Not supported")
    }
    async fn confirm_transaction(&self, hash: &str) -> Result<()> {
        bail!("Not supported")
    }
}
