use std::str::FromStr;
use web3::contract::tokens::{Tokenizable, Tokenize};
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::signing::{Key, SecretKeyRef};
use web3::types::{
    Address, BlockNumber, Bytes, CallRequest, SignedTransaction, TransactionParameters, H256, U256,
};

use secp256k1::SecretKey;

// mod crypto;
mod wrappers;

#[tokio::main]
async fn main() {
    let transport = web3::transports::Http::new("http://127.0.0.1:8545").unwrap();
    let web3 = web3::Web3::new(transport);

    let contract_address = Address::from_str("0x700b6A60ce7EaaEA56F065753d8dcB9653dbAD35").unwrap();
    let contract = web3::contract::Contract::from_json(
        web3.eth(),
        contract_address,
        include_bytes!("../abi/internal/escrow.json"),
    )
    .unwrap();

    let token_address = Address::from_str("0xA15BB66138824a1c7167f5E85b957d04Dd34E468").unwrap();
    let recipient = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
    let amount = U256::from(300);

    let function = contract.abi().function("transferTokenTo").unwrap();
    let input_data = function
        .encode_input(&(token_address, recipient, amount).into_tokens())
        .unwrap();

    let call_request = CallRequest {
        from: Some(Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()),
        to: Some(contract_address),
        gas: None,
        gas_price: None,
        value: None,
        data: Some(Bytes::from(input_data.clone())),
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let estimated_gas = web3
        .eth()
        .estimate_gas(call_request.clone(), Some(BlockNumber::Latest))
        .await
        .unwrap();

    let tx = TransactionParameters {
        nonce: None,
        to: Some(contract_address),
        gas: estimated_gas,
        gas_price: Some(web3.eth().gas_price().await.unwrap()),
        value: U256::zero(),
        data: Bytes::from(input_data),
        chain_id: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    // let tx = LegacyTransaction {
    //     nonce: None,
    //     to: Some(contract_address),
    //     gas: 21000.into(),
    //     gas_price: Some(web3.eth().gas_price().await.unwrap()),
    //     value: U256::zero(),
    //     data: Bytes::from(input_data),
    //     chain_id: None,
    //     transaction_type: None,
    //     access_list: None,
    //     max_fee_per_gas: None,
    //     max_priority_fee_per_gas: None,
    // };

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let pkey = SecretKey::from_str(private_key).unwrap();
    let signed_tx = web3.accounts().sign_transaction(tx, &pkey).await.unwrap();
    let tx_hash = web3
        .eth()
        .send_raw_transaction(Bytes::from(signed_tx.raw_transaction))
        .await
        .unwrap();

    let receipt = web3.eth().transaction_receipt(tx_hash).await.unwrap();

    match receipt {
        Some(receipt) => {
            if receipt.status == Some(web3::types::U64::from(1)) {
                println!("Transaction succeeded.");
            } else {
                println!("Transaction reverted.");
            }
        }
        None => println!("Receipt not found."),
    }

    println!("Sent transaction: {:?}", tx_hash);
}

// // fn get_escrow_contract()
// use openssl::bn::{BigNum, BigNumContext};
// use openssl::ec::EcGroup;
// use openssl::ec::{EcKey, EcPoint};
// use openssl::hash::MessageDigest;
// use openssl::nid::Nid;
// use openssl::pkey::{PKey, Private};
// use openssl::sign::Signer;
// use std::io::Result;

// fn sign(private_key: &str, data: &[u8]) -> Result<Vec<u8>> {
//     // Create the secp256k1 group
//     let group = EcGroup::from_curve_name(Nid::SECP256K1)?;

//     // Convert the private key bytes into a BigNum and create the EcKey
//     let private_key_bn = BigNum::from_hex_str(private_key)?;

//     // Generate public key
//     let mut ctx = openssl::bn::BigNumContext::new()?;
//     let mut public_key = openssl::ec::EcPoint::new(&group)?;
//     public_key.mul_generator(&group, &private_key_bn, &mut ctx)?;

//     let ec_key = EcKey::from_private_components(&group, &private_key_bn, &public_key)?;

//     // Create a PKey from the EcKey
//     let key = PKey::from_ec_key(ec_key)?;

//     // Create a signer and sign the data
//     let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
//     signer.update(data)?;
//     let signature = signer.sign_to_vec()?;

//     Ok(signature)
// }

// use tiny_keccak::{Hasher, Keccak};

// // Function to calculate Keccak256 hash
// fn keccak256(data: &[u8]) -> Vec<u8> {
//     let mut keccak = Keccak::v256();
//     let mut res = [0u8; 32];
//     keccak.update(data);
//     keccak.finalize(&mut res);
//     res.to_vec()
// }

// fn create_ec_key_from_private_key(
//     private_key_bn: &BigNum,
//     group: &EcGroup,
// ) -> Result<EcKey<Private>> {
//     // Generate public key from private key
//     let mut ctx = BigNumContext::new()?;
//     let mut public_key = EcPoint::new(&group)?;
//     public_key.mul_generator(&group, &private_key_bn, &mut ctx)?;
//     let private_key_ec = EcKey::from_private_components(&group, &private_key_bn, &public_key)?;
//     Ok(private_key_ec)
// }

// fn sign(private_key: &str, data: &[u8]) -> Result<Vec<u8>> {
//     // Create the secp256k1 group
//     let group = EcGroup::from_curve_name(Nid::SECP256K1)?;

//     // Convert the private key bytes into a BigNum and create the EcKey
//     let private_key_bn = BigNum::from_hex_str(private_key)?;

// 		let private_key_ec = create_ec_key_from_private_key(&private_key_bn, &group).unwrap();
// 		let private_key_ec = create_ec_key_from_private_key(&private_key_bn, &group).unwrap();

//     // Generate public key
//     let mut ctx = openssl::bn::BigNumContext::new()?;
//     let mut public_key = openssl::ec::EcPoint::new(&group)?;
//     public_key.mul_generator(&group, &private_key_bn, &mut ctx)?;

//     let ec_key = EcKey::from_private_components(&group, &private_key_bn, &public_key)?;

//     // Create a PKey from the EcKey
//     let key = PKey::from_ec_key(ec_key)?;

//     // Create a signer and sign the data
//     let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
//     signer.update(data)?;
//     let signature = signer.sign_to_vec()?;

//     Ok(signature)
// // }

// async fn sign() -> Result<()> {
//     let transport = web3::transports::Http::new("http://localhost:8545").unwrap();
//     let web3 = web3::Web3::new(transport);

//     let contract_address = Address::from_str("0x700b6A60ce7EaaEA56F065753d8dcB9653dbAD35").unwrap();
//     let contract = web3::contract::Contract::from_json(
//         web3.eth(),
//         contract_address,
//         include_bytes!("../abi/internal/escrow.json"),
//     )
//     .unwrap();

//     let token_address = Address::from_str("0xA15BB66138824a1c7167f5E85b957d04Dd34E468").unwrap();
//     let recipient = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
//     let amount = U256::from(300);

//     let function = contract.abi().function("transferTokenTo").unwrap();
//     let input_data = function
//         .encode_input(&(token_address, recipient, amount).into_tokens())
//         .unwrap();

//     // Get current nonce and gas price
//     let nonce = web3
//         .eth()
//         .transaction_count(contract_address, None)
//         .await
//         .unwrap();
//     let gas_price = web3.eth().gas_price().await.unwrap();

//     let tx = TransactionParameters {
//         nonce: Some(nonce),
//         to: Some(contract_address),
//         gas: 21000.into(),
//         gas_price: Some(gas_price),
//         value: U256::zero(),
//         data: Bytes::from(input_data),
//         chain_id: Some(web3.eth().chain_id().await.unwrap().as_u64()),
//         transaction_type: None,
//         access_list: None,
//         max_fee_per_gas: None,
//         max_priority_fee_per_gas: None,
//     };

//     // Get RLP encoded transaction
//     let rlp_encoded_tx = rlp_encode_transaction(&tx);

//     // Create the secp256k1 group
//     let group = EcGroup::from_curve_name(Nid::SECP256K1)?;

//     let private_key_hex = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
//     // Convert the private key bytes into a BigNum and create the EcKey
//     let private_key_bn = BigNum::from_hex_str(private_key_hex)?;
//     let private_key_ec = create_ec_key_from_private_key(&private_key_bn, &group).unwrap();
//     let private_key = PKey::from_ec_key(private_key_ec).unwrap();

//     // Sign the transaction data
//     let mut signer = Signer::new(MessageDigest::sha256(), &private_key).unwrap();
//     signer.update(&rlp_encoded_tx).unwrap();
//     let signature = signer.sign_to_vec().unwrap();

//     // Decompose signature into r, s, v values
//     let (r, s, v) = decompose_signature(&signature);

//     // Calculate the hash
//     let message_hash = keccak256(&rlp_encoded_tx);

//     let rlp_encoded_signed_tx = rlp_encode_tx_with_signature(&tx, r, s, v);

//     // Construct the signed transaction
//     // let signed_tx = SignedTransaction {
//     //     message_hash: H256::from_slice(&hash),
//     //     v: v,
//     //     r: r,
//     //     s: s,
//     //     raw_transaction: Bytes::from(rlp_encoded_tx),
//     //     transaction_hash: H256::from_slice(&hash),
//     // };

//     let tx_hash = web3
//         .eth()
//         .send_raw_transaction(Bytes::from(rlp_encoded_signed_tx))
//         .await
//         .unwrap();

//     println!("Sent transaction: {:?}", tx_hash);
//     Ok(())
// }

// use rlp::RlpStream;

// fn rlp_encode_transaction(tx: &TransactionParameters) -> Vec<u8> {
//     let mut stream = RlpStream::new();
//     stream.begin_list(9);
//     stream.append(&tx.nonce.unwrap_or_default());
//     stream.append(&tx.gas_price.unwrap_or_default());
//     stream.append(&tx.gas);
//     stream.append(&tx.to.unwrap_or_default());
//     stream.append(&tx.value);
//     stream.append(&tx.data.0);
//     stream.append(&tx.chain_id.unwrap_or_default());
//     stream.append(&0u8); // v is 0 before signing
//     stream.append(&0u8); // r is 0 before signing
//     stream.append(&0u8); // s is 0 before signing
//     stream.out().to_vec()
// }

// fn rlp_encode_tx_with_signature(tx: &TransactionParameters, r: H256, s: H256, v: u64) -> Vec<u8> {
//     let mut stream = RlpStream::new();
//     stream.begin_list(9);
//     stream.append(&tx.nonce.unwrap());
//     stream.append(&tx.gas_price.unwrap());
//     stream.append(&tx.gas);
//     stream.append(&tx.to.unwrap());
//     stream.append(&tx.value);
//     stream.append(&tx.data.0);
//     stream.append(&v);
//     stream.append(&r);
//     stream.append(&s);
//     stream.out().to_vec()
// }

// fn decompose_signature(signature: &[u8]) -> (H256, H256, u64) {
//     // assert_eq!(signature.len(), 65);

//     let r = U256::from_big_endian(&signature[0..32]);
//     let s = U256::from_big_endian(&signature[32..64]);
//     let v = u64::from(signature[64]);

//     let mut r_bytes = [0u8; 32];
//     let mut s_bytes = [0u8; 32];
//     r.to_big_endian(&mut r_bytes);
//     s.to_big_endian(&mut s_bytes);

//     (H256::from_slice(&r_bytes), H256::from_slice(&s_bytes), v)
// }
