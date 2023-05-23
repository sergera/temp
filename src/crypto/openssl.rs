// use crate::utils::ensure_success;
// use crate::{
//     CryptoAlgorithm, DerPrivateKey, DerPublicKey, PrivateExpontent, PrivateKey, PublicExpontent,
//     PublicKey, Signer,
// };
use eyre::*;

use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::sign::Verifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoAlgorithm {
    Secp256k1Sha256,
    Secp256k1None,
    Ed25519,
}

#[derive(Clone)]
pub struct OpensslPrivateKey {
    filename: String,
    keytype: CryptoAlgorithm,
}

impl OpensslPrivateKey {
    pub fn new_secp256k1_sha256(filename: &str) -> Result<Self> {
        Self::new(filename, CryptoAlgorithm::Secp256k1Sha256)
    }
    pub fn new_secp256k1_none(filename: &str) -> Result<Self> {
        Self::new(filename, CryptoAlgorithm::Secp256k1None)
    }
    pub fn new_ed25519(filename: &str) -> Result<Self> {
        Self::new(filename, CryptoAlgorithm::Ed25519)
    }
    fn new(name: &str, keytype: CryptoAlgorithm) -> Result<Self> {
        let filename = format!("{}.pem", name);
        if !std::path::Path::new(&filename).exists() {
            let args: Vec<&str> = match keytype {
                CryptoAlgorithm::Secp256k1Sha256 | CryptoAlgorithm::Secp256k1None => vec![
                    "ecparam",
                    "-genkey",
                    "-name",
                    "secp256k1",
                    "-noout",
                    "-out",
                    &filename,
                ],
                CryptoAlgorithm::Ed25519 => {
                    vec!["genpkey", "-algorithm", "ed25519", "-out", &filename]
                }
            };
            let out = Command::new("openssl").args(args).output()?;
            ensure_success(&out).context("generating key")?;
        }
        Ok(Self {
            filename: filename,
            keytype,
        })
    }
    pub fn rename(&mut self, name: &str) -> Result<()> {
        let new_name = format!("{}.pem", name);
        std::fs::rename(&self.filename, &new_name)?;
        self.filename = new_name;
        Ok(())
    }
    pub fn name(&self) -> &str {
        &self.filename
    }
}
impl PublicKey for OpensslPrivateKey {
    fn public_key(&self) -> Result<DerPublicKey> {
        let args = match self.keytype {
            CryptoAlgorithm::Secp256k1Sha256 | CryptoAlgorithm::Secp256k1None => {
                vec!["ec", "-in", &self.filename, "-pubout", "-outform", "DER"]
            }
            CryptoAlgorithm::Ed25519 => {
                vec!["pkey", "-in", &self.filename, "-pubout", "-outform", "DER"]
            }
        };
        let out = Command::new("openssl").args(args).output()?;
        ensure_success(&out).context("getting public key")?;
        Ok(out.stdout.into())
    }
    // #[deprecated]
    fn public_exponent(&self) -> Result<PublicExpontent> {
        let public_key = self.public_key()?;
        let l = match self.keytype {
            CryptoAlgorithm::Secp256k1Sha256 | CryptoAlgorithm::Secp256k1None => 65,
            CryptoAlgorithm::Ed25519 => 32,
        };
        let public_key = &public_key.content[public_key.content.len() - l..];
        Ok(public_key.to_owned().into())
    }
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        let data_file = format!(
            "/tmp/{}-{}-data.tmp",
            self.filename,
            chrono::Utc::now().to_rfc3339()
        );
        std::fs::write(&data_file, data)?;
        let sig_file = format!(
            "/tmp/{}-{}-signature.tmp",
            self.filename,
            chrono::Utc::now().to_rfc3339()
        );
        std::fs::write(&sig_file, signature)?;
        let (prog, args) = match self.keytype {
            CryptoAlgorithm::Secp256k1Sha256 | CryptoAlgorithm::Secp256k1None => todo!(),
            CryptoAlgorithm::Ed25519 => (
                "openssl-3.0",
                vec![
                    "pkeyutl",
                    "-verify",
                    // -pubin for pure public key
                    "-inkey",
                    &self.filename,
                    "-sigfile",
                    &sig_file,
                    "-rawin",
                    "-in",
                    &data_file,
                ],
            ),
        };
        let out = Command::new(prog).args(args).output()?;
        match ensure_success(&out) {
            Ok(()) => Ok(true),
            Err(err) => {
                if err.to_string().contains("openssl ecparam failed")
                    || err.to_string().contains("Signature Verification Failure")
                {
                    Ok(false)
                } else {
                    Err(err)
                }
            }
        }
    }
}
#[async_trait::async_trait]
impl Signer for OpensslPrivateKey {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let tf = format!(
            "/tmp/{}-{}.tmp",
            self.filename,
            chrono::Utc::now().to_rfc3339()
        );
        std::fs::write(&tf, data)?;

        let (prog, args) = match self.keytype {
            CryptoAlgorithm::Secp256k1Sha256 => (
                "openssl",
                vec!["dgst", "-sha256", "-sign", &self.filename, &tf],
            ),
            CryptoAlgorithm::Secp256k1None => {
                // This is not working with ethereum
                (
                    "openssl",
                    vec!["pkeyutl", "-sign", "-inkey", &self.filename, "-in", &tf],
                )
            }
            // must use a file for ed25519 at this point
            CryptoAlgorithm::Ed25519 => (
                "openssl-3.0",
                vec![
                    "pkeyutl",
                    "-sign",
                    "-inkey",
                    &self.filename,
                    "-rawin",
                    "-in",
                    &tf,
                ],
            ),
        };

        let out = Command::new(prog).args(args).output()?;
        ensure_success(&out).context("signing")?;
        // println!("signature: {}", hex::encode(&out.stdout));
        Ok(out.stdout)
    }
}
impl PrivateKey for OpensslPrivateKey {
    fn private_key(&self) -> Result<DerPrivateKey> {
        match self.keytype {
            CryptoAlgorithm::Secp256k1Sha256 | CryptoAlgorithm::Secp256k1None => {
                let args = format!(
                    "openssl ec -in {} -no_public | openssl pkcs8 -topk8 -nocrypt --outform DER",
                    self.filename
                );
                let out = Command::new("bash").arg("-c").arg(args).output()?;
                ensure_success(&out).context("getting private key")?;
                Ok(out.stdout.into())
            }
            CryptoAlgorithm::Ed25519 => {
                let private = self.private_exponent()?;
                let public = self.public_exponent()?;
                let mut real_out = hex::decode("304E020100300506032B657004420440")?;
                real_out.extend_from_slice(&private.content);
                real_out.extend_from_slice(&public.content);
                Ok(real_out.into())
            }
        }
    }

    fn private_exponent(&self) -> Result<PrivateExpontent> {
        match self.keytype {
            CryptoAlgorithm::Secp256k1Sha256 | CryptoAlgorithm::Secp256k1None => {
                let x = self.private_key()?;
                let x = x.content[x.content.len() - 32..].to_owned();
                Ok(x.into())
            }
            CryptoAlgorithm::Ed25519 => {
                let args = format!("openssl pkey -in {} --outform DER", self.filename);
                let out = Command::new("bash").arg("-c").arg(args).output()?;
                ensure_success(&out).context("getting private key")?;
                let private = out.stdout[out.stdout.len() - 32..].to_owned();
                Ok(private.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sign_and_verify_ed25519() -> Result<()> {
        let key = OpensslPrivateKey::new("test_solana_key", CryptoAlgorithm::Ed25519)?;
        let data = b"hello world";
        let mut sig = key.sign(data).await?;
        assert!(key.verify(data, &sig)?);
        assert!(!key.verify(b"hello", &sig)?);
        sig[0] = 0;
        sig[6] = 0;
        sig[8] = 0;
        assert!(!key.verify(data, &sig).unwrap());
        Ok(())
    }
}
