use crate::utils::{wait_for_confirmations_simple, wei_to_eth};
use crypto::Signer;
use eyre::*;
use signer::EthereumSigner;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use token::CryptoToken;
use web3::transports::Http;
use web3::types::{Address, TransactionParameters, TransactionRequest, H256, U256};
use web3::Web3;

pub mod contract;
pub mod erc20;
pub mod signer;
pub mod utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EthereumNet {
    Mainnet,
    Ropsten,
    Rinkeby,
    Goerli,
    Kovan,
    Local,
}

#[derive(Clone)]
pub struct EthereumToken {
    pub client: Web3<Http>,
    pub net: EthereumNet,
}
impl EthereumToken {
    pub fn new(net: EthereumNet) -> Result<Self> {
        let url = match net {
            EthereumNet::Mainnet => "https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Ropsten => "https://ropsten.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Rinkeby => "https://rinkeby.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Goerli => "https://goerli.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Kovan => "https://kovan.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
            EthereumNet::Local => "http://localhost:8545",
        };
        let transport = Http::new(url).unwrap();
        let client = Web3::new(transport);
        Ok(EthereumToken { client, net })
    }
    pub fn try_from_str(s: &str) -> Result<Option<Self>> {
        let x = match s {
            "ETH@mainnet" => Some(EthereumToken::new(EthereumNet::Mainnet)?),
            "ETH@ropsten" => Some(EthereumToken::new(EthereumNet::Ropsten)?),
            "ETH@rinkeby" => Some(EthereumToken::new(EthereumNet::Rinkeby)?),
            "ETH@goerli" => Some(EthereumToken::new(EthereumNet::Goerli)?),
            "ETH@kovan" => Some(EthereumToken::new(EthereumNet::Kovan)?),
            "ETH@local" => Some(EthereumToken::new(EthereumNet::Local)?),
            _ => None,
        };
        Ok(x)
    }
    pub async fn get_accounts(&self) -> Result<Vec<Address>> {
        let accounts = self.client.eth().accounts().await?;

        Ok(accounts)
    }
    pub async fn transfer_debug(&self, from: Address, to: Address, amount: f64) -> Result<String> {
        let amount = (amount * 1e18) as u64;
        let nonce = self.client.eth().transaction_count(from, None).await?;
        let gas_price = self.client.eth().gas_price().await?;
        let tx = TransactionRequest {
            from: from,
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            to: Some(to),
            value: Some(amount.into()),
            ..Default::default()
        };
        let tx_hash = self.client.eth().send_transaction(tx).await?;
        Ok(format!("{:?}", tx_hash))
    }
}
impl Debug for EthereumToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthereumToken")
            .field("net", &self.net)
            .finish()
    }
}

#[async_trait::async_trait]
impl CryptoToken for EthereumToken {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_network_type(&self) -> String {
        match self.net {
            EthereumNet::Mainnet => "ETH@mainnet",
            EthereumNet::Ropsten => "ETH@ropsten",
            EthereumNet::Rinkeby => "ETH@rinkeby",
            EthereumNet::Goerli => "ETH@goerli",
            EthereumNet::Kovan => "ETH@kovan",
            EthereumNet::Local => "ETH@local",
        }
        .to_string()
    }
    fn convert_display_unit_to_internal_unit(&self, amount: &str) -> Result<String> {
        let amount = amount.parse::<f64>()?;
        let amount = (amount * 1e18).to_string();
        Ok(amount)
    }
    fn convert_internal_unit_to_display_unit(&self, amount: &str) -> Result<String> {
        let amount = U256::from_str_radix(amount, 10)?;
        Ok(wei_to_eth(amount).to_string())
    }
    fn public_exponent_to_address(
        &self,
        public_exponent: &crypto::PublicExpontent,
    ) -> Result<String> {
        utils::eth_public_exponent_to_address(public_exponent).map(|x| format!("{:?}", x))
    }

    fn address_to_public_exponent(&self, _address: &str) -> Result<crypto::PublicExpontent> {
        bail!("not available for ethereum")
    }

    fn get_address_explorer_url(&self, address: &str) -> String {
        match self.net {
            EthereumNet::Mainnet => format!("https://etherscan.io/address/{}", address),
            EthereumNet::Ropsten => format!("https://ropsten.etherscan.io/address/{}", address),
            EthereumNet::Rinkeby => format!("https://rinkeby.etherscan.io/address/{}", address),
            EthereumNet::Goerli => format!("https://goerli.etherscan.io/address/{}", address),
            EthereumNet::Kovan => format!("https://kovan.etherscan.io/address/{}", address),
            EthereumNet::Local => format!("http://localhost:3000/address/{}", address),
        }
    }

    fn get_transaction_explorer_url(&self, address: &str) -> String {
        match self.net {
            EthereumNet::Mainnet => format!("https://etherscan.io/tx/{}", address),
            EthereumNet::Ropsten => format!("https://ropsten.etherscan.io/tx/{}", address),
            EthereumNet::Rinkeby => format!("https://rinkeby.etherscan.io/tx/{}", address),
            EthereumNet::Goerli => format!("https://goerli.etherscan.io/tx/{}", address),
            EthereumNet::Kovan => format!("https://kovan.etherscan.io/tx/{}", address),
            EthereumNet::Local => format!("http://localhost:3000/tx/{}", address),
        }
    }
    async fn get_balance(&self, addr: &str) -> Result<String> {
        let addr = Address::from_str(addr)?;
        let balance = self.client.eth().balance(addr, None).await?;
        Ok(balance.to_string())
    }
    async fn request_airdrop(&self, addr: &str, amount: &str) -> Result<String> {
        let addresses = self.get_accounts().await?;
        if addresses.is_empty() {
            bail!("no account found. Cannot request airdrop")
        }
        let from = addresses[0];
        self.client
            .personal()
            .unlock_account(from, "", None)
            .await?;
        let to = Address::from_str(addr)?;
        let amount = U256::from_str_radix(amount, 10)?;
        let nonce = self.client.eth().transaction_count(from, None).await?;
        let gas_price = self.client.eth().gas_price().await?;
        let tx = TransactionRequest {
            from,
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            to: Some(to),
            value: Some(amount),
            ..Default::default()
        };
        let tx_hash = self.client.eth().send_transaction(tx).await?;
        Ok(format!("{:?}", tx_hash))
    }
    async fn transfer(
        &self,
        _fee_payer: Arc<dyn Signer>,
        by: Arc<dyn Signer>,
        from: &str,
        to: &str,
        amount: &str,
    ) -> Result<String> {
        let amount = U256::from_str_radix(amount, 10)?;
        let by = EthereumSigner::new(by)?;
        if by.address != Address::from_str(from)? {
            bail!("from address is not match")
        }
        let to = Address::from_str(to)?;
        let nonce = self
            .client
            .eth()
            .transaction_count(by.address, None)
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
        Ok(format!("{:?}", tx_hash))
    }

    async fn confirm_transaction(&self, hash: &str) -> Result<()> {
        if hash.is_empty() {
            return Ok(());
        }
        let hash = H256::from_str(hash)?;
        let eth = self.client.eth();
        wait_for_confirmations_simple(&eth, hash, Duration::from_secs_f64(3.0), 10).await?;
        Ok(())
    }
    async fn create_account(
        &self,
        _fee_payer: Arc<dyn Signer>,
        _owner: &str,
        _account: Arc<dyn Signer>,
    ) -> Result<String> {
        Ok("".to_owned())
    }
    async fn get_latest_blockhash(&self) -> Result<String> {
        let block = self.client.eth().block_number().await?;
        Ok(block.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::contract::{ContractDeployer, ForgeJsonOutput};
    use crate::signer::SecretKeyOwned;
    use crate::utils::setup_logs;
    use crate::{EthereumNet, EthereumToken};
    use crypto::openssl::OpensslPrivateKey;
    use crypto::securosys::{
        get_securosys_token, make_single_approver_policy, spawn_auto_approver, SecurosysSdk,
    };
    use crypto::PrivateKey;
    use crypto::PublicKey;
    use serde_json::Value;
    use token::CryptoToken;
    use tracing::info;
    use web3::types::U256;

    #[tokio::test]
    async fn test_get_eth_balance() -> Result<()> {
        let token = EthereumToken::new(EthereumNet::Mainnet)?;
        // any address
        let balance = token
            .get_balance("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D")
            .await?;
        info!("balance: {}", balance);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_eth_transfer() -> Result<()> {
        setup_logs()?;
        let token = EthereumToken::new(EthereumNet::Local)?;

        // raw openssl private key is not working with ethereum: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2.md
        let signer = OpensslPrivateKey::new_secp256k1_none("local_eth_key")?;

        let signer = SecretKeyOwned::new_from_private_exponent(&signer.private_exponent()?)?;
        let signer = Arc::new(signer);
        let addr = format!("{:?}", signer.address);

        let tx = token
            .request_airdrop(&addr, &token.convert_display_unit_to_internal_unit("10.0")?)
            .await?;
        token.confirm_transaction(&tx).await?;
        let to_address = "0x111013b7862Ebc1B9726420aa0E8728De310Ee63";
        let balance = U256::from_str_radix(&token.get_balance(&addr).await?, 10)?;
        let tx = token
            .transfer(
                signer.clone() as _,
                signer.clone() as _,
                format!("{:?}", signer.address).as_str(),
                to_address,
                &(balance / 2).to_string(),
            )
            .await?;
        token.confirm_transaction(&tx).await?;
        Ok(())
    }
    // This test requires prior deposit to the address
    // gas fee is required
    #[tokio::test(flavor = "multi_thread")]
    async fn test_eth_transfer_on_goerli_eth() -> Result<()> {
        setup_logs()?;
        let token = EthereumToken::new(EthereumNet::Goerli)?;

        // raw openssl private key is not working with ethereum: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2.md
        let signer: OpensslPrivateKey = OpensslPrivateKey::new_secp256k1_none("local_eth_key")?;
        // let private_key = hex::encode(&signer.private_exponent()?.content);
        // info!("private key: {}", private_key);
        // return Ok(());
        let signer = SecretKeyOwned::new_from_private_exponent(&signer.private_exponent()?)?;
        let signer = Arc::new(signer);

        let addr = format!("{:?}", signer.address);
        info!("The address is {}", addr);
        let to_address = "0x111013b7862Ebc1B9726420aa0E8728De310Ee63";
        let balance = U256::from_str_radix(&token.get_balance(&addr).await?, 10)?;
        let tx = token
            .transfer(
                signer.clone() as _,
                signer.clone() as _,
                format!("{:?}", signer.address).as_str(),
                to_address,
                &(balance / 2).to_string(),
            )
            .await?;
        token.confirm_transaction(&tx).await?;
        Ok(())
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn test_eth_transfer_with_securosys() -> Result<()> {
        setup_logs()?;
        let token = EthereumToken::new(EthereumNet::Local)?;

        // raw openssl private key is not working with ethereum: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2.md
        let signer = OpensslPrivateKey::new_secp256k1_none("local_eth_key")?;

        let signer = SecretKeyOwned::new_from_private_exponent(&signer.private_exponent()?)?;
        let signer = Arc::new(signer);
        let addr = format!("{:?}", signer.address);

        let tx = token.request_airdrop(&addr, "10.0").await?;
        token.confirm_transaction(&tx).await?;

        let to_address = "0x111013b7862Ebc1B9726420aa0E8728De310Ee63";
        let balance = token.get_balance(&addr).await?;
        println!("balance: {}", balance);
        let tx = token
            .transfer(
                signer.clone() as _,
                signer.clone() as _,
                &addr,
                to_address,
                &token.convert_display_unit_to_internal_unit("8.0")?,
            )
            .await?;
        token.confirm_transaction(&tx).await?;
        Ok(())
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn test_transfer_with_securosys() -> Result<()> {
        setup_logs()?;
        let hsm = Arc::new(SecurosysSdk::new(get_securosys_token()?)?);
        let approver = "approver";
        let keyname = "test_ethereum_key";
        let local_key = OpensslPrivateKey::new_secp256k1_none(keyname)?;
        let approver_key = OpensslPrivateKey::new_secp256k1_sha256(approver)?;

        let approver_key1 = OpensslPrivateKey::new_secp256k1_sha256(approver)?;
        let hsm_signer = Arc::new(SecretKeyOwned::new_from_private_exponent(
            &local_key.private_exponent()?,
        )?);

        let private_key = local_key.private_key()?;
        let public_key = local_key.public_key()?;
        hsm.delete_key(keyname).await?;
        let policy = make_single_approver_policy(approver.to_owned(), approver_key1.public_key()?);
        hsm.import_key_secp256k1(&keyname, policy, private_key, public_key)
            .await?;
        let terminate_tx = spawn_auto_approver(hsm.clone(), approver_key);
        let token = EthereumToken::new(EthereumNet::Local)?;
        let addresses = token.get_accounts().await?;
        info!("addresses: {:?}", addresses);
        let address1_addr = addresses[0];
        let address1_str = format!("{:?}", address1_addr);
        let balance = token.get_balance(&address1_str).await?;
        info!("balance: {}", balance);
        let tx = token
            .transfer_debug(address1_addr, hsm_signer.address, 10.0)
            .await?;
        token.confirm_transaction(&tx).await?;
        let balance2 = token.get_balance(&address1_str).await?;
        info!("balance: {}", balance2);
        let balance_signer = token
            .get_balance(&format!("{:?}", hsm_signer.address))
            .await?;
        info!("balance: {}", balance_signer);
        let tx = token
            .transfer(
                hsm_signer.clone() as _,
                hsm_signer.clone() as _,
                &format!("{:?}", hsm_signer.address),
                &address1_str,
                token.convert_display_unit_to_internal_unit("8.0")?.as_str(),
            )
            .await?;
        token.confirm_transaction(&tx).await?;
        let balance3 = token.get_balance(&address1_str).await?;
        info!("balance: {}", balance3);
        drop(terminate_tx);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_deploy_erc20() -> Result<()> {
        setup_logs()?;
        let token = EthereumToken::new(EthereumNet::Local)?;
        let signer = OpensslPrivateKey::new_secp256k1_none("local_eth_key")?;

        let signer = SecretKeyOwned::new_from_private_exponent(&signer.private_exponent()?)?;
        let signer = Arc::new(signer);
        let addr = format!("{:?}", signer.address);

        let tx = token
            .request_airdrop(&addr, &token.convert_display_unit_to_internal_unit("10.0")?)
            .await?;
        token.confirm_transaction(&tx).await?;

        let output = include_str!("../../erc20/out/MyERC20.sol/MyERC20.json");
        let output: ForgeJsonOutput = serde_json::from_str(output)?;

        let tx = ContractDeployer::new(token.client.eth(), Value::Array(output.abi))?
            .code(output.bytecode.object)
            .sign_with_key_and_execute((), EthereumSigner::new(signer.clone() as _)?)
            .await?;
        info!("deployed tx: {:?}", tx.address());
        Ok(())
    }
}
