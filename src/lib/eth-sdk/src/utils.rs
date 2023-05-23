use eyre::*;
use secp256k1::PublicKey;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, EnvFilter};
use web3::api::Eth;
use web3::signing::keccak256;
use web3::types::{Address, TransactionReceipt, H256, U256};
use web3::Transport;
pub fn eth_public_exponent_to_address(
    public_exponent: &crypto::PublicExpontent,
) -> Result<Address> {
    let public_key = PublicKey::from_slice(&public_exponent.content).map_err(|_| {
        eyre!(
            "malformed public key: {}",
            hex::encode(&public_exponent.content)
        )
    })?;
    let public_key = public_key.serialize_uncompressed();

    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

    Ok(Address::from_slice(&hash[12..]))
}

pub fn wei_to_eth(wei_val: web3::types::U256) -> f64 {
    let u = U256::from_str_radix("1000000000000000000", 10).unwrap();
    let n = wei_val / u;
    let f = wei_val % u;
    (n.as_u128() as f64) + f.as_u128() as f64 / 1e18
}

// for testing only
pub fn setup_logs() -> Result<()> {
    LogTracer::init().context("Cannot setup_logs")?;
    let filter = EnvFilter::from_default_env().add_directive(LevelFilter::TRACE.into());

    let subscriber = fmt()
        .with_thread_names(true)
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).context("Cannot setup_logs")?;
    Ok(())
}

/// Should be used to wait for confirmations
pub async fn wait_for_confirmations_simple<T>(
    eth: &Eth<T>,
    hash: H256,
    poll_interval: Duration,
    max_retry: usize,
) -> Result<TransactionReceipt>
where
    T: Transport,
{
    for _ in 0..max_retry {
        if let Some(receipt) = eth.transaction_receipt(hash).await? {
            return Ok(receipt);
        }
        tokio::time::sleep(poll_interval).await;
    }
    bail!(
        "Transaction {:?} not found within {} retries",
        hash,
        max_retry
    )
}

#[cfg(test)]
mod tests {
    use crypto::openssl::OpensslPrivateKey;
    use crypto::PublicKey;
    use eyre::*;

    #[test]
    fn test_eth_public_exponent_to_address() -> Result<()> {
        let key = OpensslPrivateKey::new_secp256k1_none("test_eth_key")?;
        let public_exponent = key.public_exponent()?;
        let address = super::eth_public_exponent_to_address(&public_exponent).unwrap();
        println!("address: {}", address);
        Ok(())
    }
}
