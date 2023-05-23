use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::sign::Verifier;

#[derive(Clone)]
pub struct OpensslPrivateKey {
    key: PKey,
    keytype: CryptoAlgorithm,
}

impl OpensslPrivateKey {
    pub fn new_secp256k1_sha256(private_key: &[u8]) -> Result<Self> {
        Self::new(private_key, CryptoAlgorithm::Secp256k1Sha256)
    }

    pub fn new_secp256k1_none(private_key: &[u8]) -> Result<Self> {
        Self::new(private_key, CryptoAlgorithm::Secp256k1None)
    }

    pub fn new_ed25519(private_key: &[u8]) -> Result<Self> {
        Self::new(private_key, CryptoAlgorithm::Ed25519)
    }

    fn new(private_key: &[u8], keytype: CryptoAlgorithm) -> Result<Self> {
        let ec_key = EcKey::private_key_from_der(private_key)?;
        let key = PKey::from_ec_key(ec_key)?;

        Ok(Self { key, keytype })
    }
}

impl PublicKey for OpensslPrivateKey {
    fn public_key(&self) -> Result<DerPublicKey> {
        let ec_key = self.key.ec_key()?;
        let group = EcGroup::from_curve_name(Nid::SECP256K1)?;
        let public_key = ec_key.public_key().to_bytes(
            &group,
            openssl::ec::PointConversionForm::UNCOMPRESSED,
            &mut openssl::bn::BigNumContext::new()?,
        )?;
        Ok(public_key.into())
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &self.key)?;
        verifier.update(data)?;
        Ok(verifier.verify(signature)?)
    }
}

#[async_trait::async_trait]
impl Signer for OpensslPrivateKey {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut signer = Signer::new(MessageDigest::sha256(), &self.key)?;
        signer.update(data)?;
        Ok(signer.sign_to_vec()?)
    }
}

impl PrivateKey for OpensslPrivateKey {
    fn private_key(&self) -> Result<DerPrivateKey> {
        let private_key = self.key.private_key_to_der()?;
        Ok(private_key.into())
    }
}
