use codec::Encode;
use sp_core::{H160, H256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::traits::{Hash, Zero};

use module_evm_utility_macro::keccak256;

use crate::{Config, Pallet};

/// Evm Address.
pub(crate) type EvmAddress = H160;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Eip712Signature = [u8; 65];

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
pub fn evm_sign(secret: &libsecp256k1::SecretKey, msg_hash: &MessageHash) -> Eip712Signature {
    let (sig, recovery_id) =
        libsecp256k1::sign(&libsecp256k1::Message::parse(&msg_hash), secret);
    let mut r = [0u8; 65];
    r[0..64].copy_from_slice(&sig.serialize()[..]);
    r[64] = recovery_id.serialize();
    r
}

impl<T: Config> Pallet<T> {
    pub(crate) fn verify_eip712_signature(
        message: &SingableMessage<T>,
        sig: &Eip712Signature,
    ) -> Option<EvmAddress> {
        recover_evm_signer(sig, &message.message_hash())
    }
}

fn recover_evm_signer(sig: &Eip712Signature, msg_hash: &MessageHash) -> Option<EvmAddress> {
    secp256k1_ecdsa_recover(sig, msg_hash)
        .map(|pubkey| H160::from(H256::from_slice(&keccak_256(&pubkey))))
        .ok()
}

pub(crate) enum SingableMessage<T: Config> {
    LinkEvmAddress { evm_address: EvmAddress, substrate_address: T::AccountId },
    EvmAddressCall { call_hash: <<T as Config>::CallHasher as Hash>::Output, account_nonce: T::Index },
}

impl<T: Config> SingableMessage<T> {
    pub(crate) fn message_hash(&self) -> MessageHash {
        let mut msg = b"\x19\x01".to_vec();
        msg.extend_from_slice(&self.domain_separator());
        msg.extend_from_slice(&self.payload_hash());
        keccak_256(msg.as_slice())
    }

    pub(crate) fn payload_hash(&self) -> MessageHash {
        match self {
            SingableMessage::LinkEvmAddress { evm_address, substrate_address } => {
                let tx_type_hash = keccak256!(
                    "Transaction(string transactionName,bytes substrateAddress, bytes evmAddress)"
                );
                let mut tx_msg = tx_type_hash.to_vec();
                tx_msg.extend_from_slice(keccak256!("LinkEvmAddress")); // transactionName
                tx_msg.extend_from_slice(&keccak_256(&substrate_address.encode())); // substrateAddress
                tx_msg.extend_from_slice(&keccak_256(&evm_address.encode())); // evmAddress
                keccak_256(tx_msg.as_slice())
            }
            SingableMessage::EvmAddressCall { call_hash, account_nonce} => {
                let tx_type_hash = keccak256!("Transaction(string transactionName,bytes callHash, uint256 accountNonce)");
                let mut tx_msg = tx_type_hash.to_vec();
                tx_msg.extend_from_slice(keccak256!("EvmAddressCall")); // transactionName
                tx_msg.extend_from_slice(&keccak_256(&call_hash.encode())); // callHash
                tx_msg.extend_from_slice(&keccak_256(&account_nonce.encode())); // accountNonce
                keccak_256(tx_msg.as_slice())
            },
        }
    }

    pub(crate) fn domain_separator(&self) -> MessageHash {
        let domain_hash = keccak256!("EIP712Domain(string name,string version,bytes32 salt)");
        let mut domain_seperator_msg = domain_hash.to_vec();
        domain_seperator_msg.extend_from_slice(keccak256!("SubSocial Evm Address Linkage")); // name
        domain_seperator_msg.extend_from_slice(keccak256!("1")); // version
        domain_seperator_msg.extend_from_slice(
            frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero()).as_ref(),
        ); // genesis block hash
        keccak_256(domain_seperator_msg.as_slice())
    }
}
