pub mod utils;

use der::Reader;
use eyre::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DerPublicKey {
    pub content: Vec<u8>,
}
impl From<Vec<u8>> for DerPublicKey {
    fn from(content: Vec<u8>) -> Self {
        Self { content }
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DerPrivateKey {
    pub content: Vec<u8>,
}
impl From<Vec<u8>> for DerPrivateKey {
    fn from(content: Vec<u8>) -> Self {
        Self { content }
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PublicExponent {
    pub content: Vec<u8>,
}
impl PublicExponent {
    pub fn hex(&self) -> String {
        hex::encode(&self.content)
    }
}
impl From<Vec<u8>> for PublicExponent {
    fn from(content: Vec<u8>) -> Self {
        Self { content }
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrivateExponent {
    pub content: Vec<u8>,
}
impl From<Vec<u8>> for PrivateExponent {
    fn from(content: Vec<u8>) -> Self {
        Self { content }
    }
}
pub trait PublicKey {
    fn public_key(&self) -> Result<DerPublicKey>;
    fn public_exponent(&self) -> Result<PublicExponent>;
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool>;
}
#[async_trait::async_trait]
pub trait Signer: PublicKey + Sync + Send {
    /// Sign data with the private key, in ANS1 DER format
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
    /// Sign data with the private key, in raw format
    async fn sign_compact(&self, data: &[u8]) -> Result<Vec<u8>> {
        let res = self.sign(data).await?;
        decode_ans1_signature(&res)
    }
}

pub fn sign_sync(this: &dyn Signer, data: &[u8]) -> Result<Vec<u8>> {
    let handle = tokio::runtime::Handle::current();
    tokio::task::block_in_place(|| handle.block_on(this.sign(data)))
}
pub fn sign_sync_compact(this: &dyn Signer, data: &[u8]) -> Result<Vec<u8>> {
    let handle = tokio::runtime::Handle::current();
    tokio::task::block_in_place(|| handle.block_on(this.sign_compact(data)))
}
pub trait PrivateKey: PublicKey + Signer {
    fn private_key(&self) -> Result<DerPrivateKey>;
    fn private_exponent(&self) -> Result<PrivateExponent>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoAlgorithm {
    Secp256k1Sha256,
    Secp256k1None,
    Ed25519,
}

pub fn decode_ans1_signature(result: &[u8]) -> Result<Vec<u8>> {
    let seq: [der::asn1::UIntRef; 2] = der::SliceReader::new(result)
        .unwrap() // safety: always success
        .decode()
        .map_err(|x| eyre!("Failed to parse ansi message: {:?}", x))?;
    Ok(seq.map(|x| x.as_bytes()).concat())
}
