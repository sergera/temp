use crate::utils;
use crypto::{
    sign_sync_compact, DerPublicKey, PrivateExpontent, PublicExpontent, PublicKey, Signer,
};
use eyre::*;
use once_cell::sync::Lazy;
use secp256k1::{All, Message, Secp256k1, SecretKey};
use std::sync::Arc;
use tracing::warn;
use web3::signing::{keccak256, recover, Key, SigningError};
use web3::types::{Address, H256};

pub struct EthereumSigner {
    inner: Arc<dyn Signer>,
    pub address: Address,
}

impl EthereumSigner {
    pub fn new(inner: Arc<dyn Signer>) -> Result<Self> {
        let address = utils::eth_public_exponent_to_address(&inner.public_exponent()?)?;
        Ok(Self { inner, address })
    }
}

fn get_recovery_id(msg: &[u8], s: &[u8], address: Address) -> Result<i32> {
    if recover(msg, s, 0) == Ok(address) {
        Ok(0)
    } else if recover(msg, s, 1) == Ok(address) {
        Ok(1)
    } else {
        Err(eyre!("Failed to recover address"))
    }
}

impl Key for EthereumSigner {
    fn sign(
        &self,
        message: &[u8],
        chain_id: Option<u64>,
    ) -> Result<web3::signing::Signature, SigningError> {
        if message.len() != 32 {
            return Err(SigningError::InvalidMessage);
        }
        let signature = sign_sync_compact(&*self.inner, message).map_err(|x| {
            warn!("sign error: {:?}", x);
            SigningError::InvalidMessage
        })?;
        let recovery_id = get_recovery_id(message, &signature, self.address).map_err(|x| {
            warn!("get_recovery_id error: {:?}", x);
            SigningError::InvalidMessage
        })?;
        let standard_v = recovery_id as u64;
        let v = if let Some(chain_id) = chain_id {
            // When signing with a chain ID, add chain replay protection.
            standard_v + 35 + chain_id * 2
        } else {
            // Otherwise, convert to 'Electrum' notation.
            standard_v + 27
        };
        let r = H256::from_slice(&signature[..32]);
        let s = H256::from_slice(&signature[32..]);

        Ok(web3::signing::Signature { v, r, s })
    }

    fn sign_message(&self, message: &[u8]) -> Result<web3::signing::Signature, SigningError> {
        if message.len() != 32 {
            return Err(SigningError::InvalidMessage);
        }
        let signature = sign_sync_compact(&*self.inner, message).map_err(|x| {
            warn!("sign error: {:?}", x);
            SigningError::InvalidMessage
        })?;

        let recovery_id = get_recovery_id(message, &signature, self.address).map_err(|x| {
            warn!("get_recovery_id error: {:?}", x);
            SigningError::InvalidMessage
        })?;
        let v = recovery_id as u64;
        let r = H256::from_slice(&signature[..32]);
        let s = H256::from_slice(&signature[32..]);

        Ok(web3::signing::Signature { v, r, s })
    }

    fn address(&self) -> Address {
        self.address
    }
}

pub struct SecretKeyOwned {
    pub key: SecretKey,
    pub pubkey: secp256k1::PublicKey,
    pub address: Address,
}

impl SecretKeyOwned {
    pub fn new_from_private_exponent(key: &PrivateExpontent) -> Result<Self> {
        let key = SecretKey::from_slice(&key.content)?;
        let pubkey = secp256k1::PublicKey::from_secret_key(&*CONTEXT, &key);
        let address = utils::eth_public_exponent_to_address(&PublicExpontent {
            content: pubkey.serialize().to_vec(),
        })?;
        Ok(Self {
            key,
            pubkey,
            address,
        })
    }
    pub fn new(key: SecretKey) -> Self {
        let pubkey = secp256k1::PublicKey::from_secret_key(&*CONTEXT, &key);
        let address = utils::eth_public_exponent_to_address(&PublicExpontent {
            content: pubkey.serialize().to_vec(),
        })
        .unwrap();
        Self {
            key,
            pubkey,
            address,
        }
    }
}
impl PublicKey for SecretKeyOwned {
    fn public_key(&self) -> Result<DerPublicKey> {
        todo!()
    }

    fn public_exponent(&self) -> Result<PublicExpontent> {
        self.pubkey
            .serialize_uncompressed()
            .to_vec()
            .try_into()
            .map_err(|_| eyre!("Failed to convert public exponent"))
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        let message = Message::from_slice(data).unwrap();
        let signature = secp256k1::ecdsa::Signature::from_compact(signature).unwrap();
        CONTEXT
            .verify_ecdsa(&message, &signature, &self.pubkey)
            .expect("invalid signature");
        Ok(true)
    }
}
#[async_trait::async_trait]
impl Signer for SecretKeyOwned {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let message = Message::from_slice(data).map_err(|_| SigningError::InvalidMessage)?;
        let signature = CONTEXT.sign_ecdsa(&message, &self.key);
        let signature_compact = signature.serialize_der();
        Ok(signature_compact.to_vec())
    }
}

impl Key for SecretKeyOwned {
    fn sign(
        &self,
        message: &[u8],
        chain_id: Option<u64>,
    ) -> Result<web3::signing::Signature, SigningError> {
        let message = Message::from_slice(message).map_err(|_| SigningError::InvalidMessage)?;
        let signature = CONTEXT.sign_ecdsa(&message, &self.key);
        let signature_compact = signature.serialize_compact();
        CONTEXT
            .verify_ecdsa(&message, &signature, &self.pubkey)
            .expect("invalid signature");
        let recovery_id = get_recovery_id(message.as_ref(), &signature_compact, self.address())
            .map_err(|x| {
                warn!("get_recovery_id error: {:?}", x);
                SigningError::InvalidMessage
            })?;
        let standard_v = recovery_id as u64;
        let v = if let Some(chain_id) = chain_id {
            // When signing with a chain ID, add chain replay protection.
            standard_v + 35 + chain_id * 2
        } else {
            // Otherwise, convert to 'Electrum' notation.
            standard_v + 27
        };

        let r = H256::from_slice(&signature_compact[..32]);
        let s = H256::from_slice(&signature_compact[32..]);

        Ok(web3::signing::Signature { v, r, s })
    }

    fn sign_message(&self, message: &[u8]) -> Result<web3::signing::Signature, SigningError> {
        let message = Message::from_slice(message).map_err(|_| SigningError::InvalidMessage)?;
        let signature = CONTEXT.sign_ecdsa(&message, &self.key);
        let signature_compact = signature.serialize_compact();
        CONTEXT
            .verify_ecdsa(&message, &signature, &self.pubkey)
            .expect("invalid signature");
        let recovery_id = get_recovery_id(message.as_ref(), &signature_compact, self.address())
            .map_err(|x| {
                warn!("get_recovery_id error: {:?}", x);
                SigningError::InvalidMessage
            })?;
        let v = recovery_id as u64;
        let r = H256::from_slice(&signature_compact[..32]);
        let s = H256::from_slice(&signature_compact[32..]);

        Ok(web3::signing::Signature { v, r, s })
    }

    fn address(&self) -> Address {
        secret_key_address(&self.key)
    }
}
static CONTEXT: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

/// Gets the address of a public key.
///
/// The public address is defined as the low 20 bytes of the keccak hash of
/// the public key. Note that the public key returned from the `secp256k1`
/// crate is 65 bytes long, that is because it is prefixed by `0x04` to
/// indicate an uncompressed public key; this first byte is ignored when
/// computing the hash.
pub fn public_key_address(public_key: &secp256k1::PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();

    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

    Address::from_slice(&hash[12..])
}

/// Gets the public address of a private key.
pub(crate) fn secret_key_address(key: &SecretKey) -> Address {
    let secp = &*CONTEXT;
    let public_key = secp256k1::PublicKey::from_secret_key(secp, key);
    public_key_address(&public_key)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::{eth_public_exponent_to_address, setup_logs};
    use crypto::openssl::OpensslPrivateKey;
    use crypto::PrivateKey;
    use secp256k1::SecretKey;
    use web3::signing::{keccak256, Key};

    #[test]
    fn test_convert_key_to_secp255k1() -> Result<()> {
        let key = OpensslPrivateKey::new_secp256k1_none("test_eth_key")?;
        let exp = key.public_exponent()?;
        println!("exp len {}", exp.content.len());
        let addr = eth_public_exponent_to_address(&exp)?;
        println!("addr: {:?}", addr);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_btc_sign_messages() -> Result<()> {
        setup_logs()?;
        let key = OpensslPrivateKey::new_secp256k1_sha256("test_eth_key")?;
        println!("Private key {}", hex::encode(key.private_key()?.content));
        let key2 = &SecretKey::from_slice(&key.private_exponent()?.content)?;
        let key_owned = SecretKeyOwned::new(key2.clone());
        let msg = keccak256(b"hello world");
        let sig2 = key2.sign_message(&msg)?;
        println!("sig2: {:?}", sig2.v);
        let sig1 = key_owned.sign_message(&msg)?;
        println!("sig1: {:?}", sig1.v);
        assert_eq!(sig1.v, sig2.v);
        Ok(())
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sign_messages() -> Result<()> {
        setup_logs()?;
        let key = OpensslPrivateKey::new_secp256k1_none("test_eth_key")?;
        println!("Private key {}", hex::encode(key.private_key()?.content));
        let key2 = &SecretKey::from_slice(&key.private_exponent()?.content)?;
        let key3 = SecretKeyOwned::new(key2.clone());
        let key1 = EthereumSigner::new(Arc::new(key))?;
        let msg = keccak256(b"hello world");
        let sig3 = key3.sign_message(&msg)?;
        println!("sig3: {:?}", sig3.v);
        let sig2 = key2.sign_message(&msg)?;
        println!("sig2: {:?}", sig2.v);
        let sig1 = key1.sign_message(&msg)?;
        println!("sig1: {:?}", sig1.v);
        Ok(())
    }
}
