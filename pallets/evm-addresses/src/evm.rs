use codec::Encode;
use scale_info::prelude::format;
use sp_core_hashing::keccak_256;
use sp_io::crypto::secp256k1_ecdsa_recover;
use sp_std::vec::Vec;

use crate::{Config, Pallet};

/// EVM Address.
pub(crate) type EvmAddress = sp_core::H160;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub(crate) type EcdsaSignature = [u8; 65];

const MSG_PART_1: &str = "Link to Subsocial address ";
const MSG_PART_2: &str = " (in hex) with nonce ";

impl<T: Config> Pallet<T> {
    pub(crate) fn verify_evm_signature(
        sig: &EcdsaSignature,
        sub_address: &T::AccountId,
        sub_nonce: T::Index,
    ) -> Option<EvmAddress> {
        let msg = keccak_256(&Self::eth_signable_message(sub_address, sub_nonce));

        let mut evm_addr = EvmAddress::default();
        let pub_key = &secp256k1_ecdsa_recover(&sig, &msg).ok()?[..];
        evm_addr.0.copy_from_slice(&keccak_256(pub_key)[12..]);
        Some(evm_addr)
    }

    /// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
    /// In accordance with https://eips.ethereum.org/EIPS/eip-191
    fn eth_signable_message(sub_address: &T::AccountId, sub_nonce: T::Index) -> Vec<u8> {
        let addr = hex::encode(sub_address.encode());
        let nonce = format!("{:?}", sub_nonce);

        let personal_part = format!("{MSG_PART_1}{addr}{MSG_PART_2}{nonce}");
        let len = personal_part.len();

        format!("\x19Ethereum Signed Message:\n{len}{personal_part}")
            .as_bytes()
            .to_vec()
    }
}

//* ONLY FOR TESTS *//
/*
pub(crate) type MessageHash = [u8; 32];

#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
pub(crate) fn evm_secret_key(seed: &[u8]) -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(seed)).unwrap()
}

// Returns an Ethereum public key derived from an Ethereum secret key.
#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
pub fn evm_public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
    libsecp256k1::PublicKey::from_secret_key(secret)
}

// Returns an Ethereum address derived from an Ethereum secret key.
// Only for tests
#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
pub fn evm_address(secret: &libsecp256k1::SecretKey) -> EvmAddress {
    EvmAddress::from_slice(&keccak_256(&evm_public(secret).serialize()[1..65])[12..])
}

// Constructs a message and signs it.
#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
pub fn evm_sign(secret: &libsecp256k1::SecretKey, msg_hash: &MessageHash) -> EcdsaSignature {
    let (sig, recovery_id) =
        libsecp256k1::sign(&libsecp256k1::Message::parse(&msg_hash), secret);
    let mut r = [0u8; 65];
    r[0..64].copy_from_slice(&sig.serialize()[..]);
    r[64] = recovery_id.serialize();
    r
}
*/
