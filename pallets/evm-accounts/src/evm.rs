use codec::Encode;
use sp_core::{H160, H256, keccak_256};
use sp_io::{crypto::secp256k1_ecdsa_recover};
use sp_runtime::traits::{Hash, Zero};

use crate::{Config, Pallet};

/// Evm Address.
pub(crate) type EvmAddress = H160;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type EcdsaSignature = [u8; 65];

pub(crate) type MessageHash = [u8; 32];

const MSG_PART_1: &[u8] = b"Link to Subsocial address ";
const MSG_PART_2: &[u8] = b" with nonce ";

impl<T: Config> Pallet<T> {
    pub(crate) fn verify_signature(
        sig: &EcdsaSignature,
        sub_address: &[u8],
        sub_nonce: &[u8]
    ) -> Option<EvmAddress> {
        let msg = keccak_256(&eth_signable_message(sub_address, sub_nonce));

        let mut evm_addr = EvmAddress::default();
        let pub_key = &secp256k1_ecdsa_recover(&sig, &msg).ok()?[..];
        evm_addr.0.copy_from_slice(&keccak_256(pub_key)[12..]);
        Some(evm_addr)
    }
}

/// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
fn eth_signable_message(sub_address: &[u8], sub_nonce: &[u8]) -> Vec<u8> {
    let mut l = MSG_PART_1.len() + sub_address.len() + MSG_PART_2.len() + sub_nonce.len();
    let mut rev = Vec::new();
    while l > 0 {
        rev.push(b'0' + (l % 10) as u8);
        l /= 10;
    }
    let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
    v.extend(rev.into_iter().rev());
    v.extend_from_slice(MSG_PART_1);
    v.extend_from_slice(sub_address);
    v.extend_from_slice(MSG_PART_2);
    v.extend_from_slice(sub_nonce);
    v
}

//* ONLY FOR TESTS *//

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