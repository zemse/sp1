use std::borrow::Cow;
use std::str::FromStr;

use alloy::signers::{wallet::LocalWallet, Signer};
use alloy::sol;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::Result;

sol! {
    struct CreateProof {
        uint64 deadline;
    }

    struct SubmitProof {
        string proof_id;
    }

    struct RelayProof {
        string proof_id;
        uint32 chain_id;
        address verifier;
        address callback;
        bytes callback_data;
    }
}

/// Handles authentication for the prover network. All interactions that could potentially use
/// computational resources must be authenticated by signing a message with a secp256k1 key.
///
/// The messages themselves follow EIP-712, where the domain is "succinct" and the TypeStruct changes
/// depending on which endpoint is being used. Documentation for EIP-712 can be found at:
/// https://eips.ethereum.org/EIPS/eip-712
pub struct NetworkAuth {
    // Holds a secp256k1 private key.
    wallet: LocalWallet,
}

impl NetworkAuth {
    pub fn new(private_key: &str) -> Self {
        let wallet = LocalWallet::from_str(private_key).unwrap();
        Self { wallet }
    }

    /// Gets the EIP-712 domain separator for the Succinct prover network.
    fn get_domain_separator() -> Eip712Domain {
        Eip712Domain {
            name: Some(Cow::Borrowed("succinct")),
            version: Some(Cow::Borrowed("1")),
            ..Default::default()
        }
    }

    /// Signs a message to create a proof.
    pub async fn sign_create_proof_message(&self, deadline: u64) -> Result<Vec<u8>> {
        let domain_seperator = Self::get_domain_separator();

        let type_struct = CreateProof { deadline };

        let message_hash = type_struct.eip712_signing_hash(&domain_seperator);
        let signature = self.wallet.sign_hash(&message_hash).await?;

        Ok(signature.as_bytes().to_vec())
    }

    /// Signs a message to submit a proof.
    pub async fn sign_submit_proof_message(&self, proof_id: &str) -> Result<Vec<u8>> {
        let domain_seperator = Self::get_domain_separator();

        let type_struct = SubmitProof {
            proof_id: proof_id.to_string(),
        };

        let message_hash = type_struct.eip712_signing_hash(&domain_seperator);
        let signature = self.wallet.sign_hash(&message_hash).await?;

        Ok(signature.as_bytes().to_vec())
    }

    /// Signs a message to relay a proof.
    pub async fn sign_relay_proof_message(
        &self,
        proof_id: &str,
        chain_id: u32,
        verifier: [u8; 20],
        callback: [u8; 20],
        callback_data: &[u8],
    ) -> Result<Vec<u8>> {
        let domain_seperator = Self::get_domain_separator();

        let type_struct = RelayProof {
            proof_id: proof_id.to_string(),
            chain_id,
            verifier: verifier.into(),
            callback: callback.into(),
            callback_data: callback_data.to_vec().into(),
        };

        let message_hash = type_struct.eip712_signing_hash(&domain_seperator);
        let signature = self.wallet.sign_hash(&message_hash).await?;

        Ok(signature.as_bytes().to_vec())
    }
}