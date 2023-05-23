use crate::signer::EthereumSigner;
use crate::utils::{eth_public_exponent_to_address, wait_for_confirmations_simple, wei_to_eth};
use crate::EthereumNet;
use crypto::Signer;
use eyre::*;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use token::CryptoToken;
use web3::api::Web3;
use web3::contract::{Contract, Options};
use web3::transports::http::Http;
use web3::types::{Address, H256, U256};

const ERC20_ABI: &'static str = include_str!("erc20.abi.json");

pub struct Erc20Token {
    client: Web3<Http>,
    net: EthereumNet,
    address: Address,
    contract: Contract<Http>,
}

impl Erc20Token {
    pub fn new(net: EthereumNet, address: Address) -> Result<Self> {
        // I don't know whose token are these. Copilot gave me these
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
        let contract = Contract::from_json(client.eth(), address, ERC20_ABI.as_bytes()).unwrap();
        Ok(Erc20Token {
            client,
            net,
            address,
            contract,
        })
    }

    pub fn try_from_str(s: &str, contract_address: &str) -> Result<Option<Self>> {
        let address = || {
            Address::from_str(contract_address)
                .with_context(|| format!("address {}", contract_address))
        };
        let x = match s {
            "ERC20@mainnet" => Some(Erc20Token::new(EthereumNet::Mainnet, address()?)?),
            "ERC20@ropsten" => Some(Erc20Token::new(EthereumNet::Ropsten, address()?)?),
            "ERC20@rinkeby" => Some(Erc20Token::new(EthereumNet::Rinkeby, address()?)?),
            "ERC20@goerli" => Some(Erc20Token::new(EthereumNet::Goerli, address()?)?),
            "ERC20@kovan" => Some(Erc20Token::new(EthereumNet::Kovan, address()?)?),
            "ERC20@local" => Some(Erc20Token::new(EthereumNet::Local, address()?)?),
            _ => None,
        };
        Ok(x)
    }
}

impl Debug for Erc20Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ERC20Token")
            .field("net", &self.net)
            .field("address", &self.address)
            .finish()
    }
}

#[async_trait::async_trait]
impl CryptoToken for Erc20Token {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_network_type(&self) -> String {
        match self.net {
            EthereumNet::Mainnet => "ERC20@mainnet",
            EthereumNet::Ropsten => "ERC20@ropsten",
            EthereumNet::Rinkeby => "ERC20@rinkeby",
            EthereumNet::Goerli => "ERC20@goerli",
            EthereumNet::Kovan => "ERC20@kovan",
            EthereumNet::Local => "ERC20@local",
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
        eth_public_exponent_to_address(public_exponent).map(|x| format!("{:?}", x))
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
        let balance: U256 = self
            .contract
            .query("balanceOf", addr, None, Options::default(), None)
            .await?;
        Ok(balance.to_string())
    }
    async fn request_airdrop(&self, _addr: &str, _amount: &str) -> Result<String> {
        bail!("not available for ethereum")
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
        if by.address == Address::from_str(from)? {
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
        let options = Options {
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            gas: Some(gas_limit.into()),
            value: amount.into(),
            ..Default::default()
        };
        let tx_hash = self
            .contract
            .signed_call("transfer", (to, amount), options, by)
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
    async fn mint_to(
        &self,
        fee_payer: Arc<dyn Signer>,
        minter: Arc<dyn Signer>,
        account: &str,
        amount: &str,
    ) -> Result<String> {
        let amount = U256::from_str_radix(amount, 10)?;
        let fee_payer = EthereumSigner::new(fee_payer)?;
        let minter = EthereumSigner::new(minter)?;
        if fee_payer.address == minter.address {
            bail!("minter address is not match")
        }
        let account = Address::from_str(account)?;
        let nonce = self
            .client
            .eth()
            .transaction_count(fee_payer.address, None)
            .await?;
        let gas_price = self.client.eth().gas_price().await?;
        let gas_limit = 21000;
        let options = Options {
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            gas: Some(gas_limit.into()),
            ..Default::default()
        };
        let tx_hash = self
            .contract
            .signed_call("mintTo", (account, amount), options, minter)
            .await?;

        Ok(format!("{:?}", tx_hash))
    }
}
