#![allow(non_snake_case)]

use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use near_sdk::{env, near};

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct ClaimInfo {
    pub provider: String,
    pub parameters: String,
    pub context: String,
}

impl ClaimInfo {
    pub fn hash(&self) -> String {
        let hash_str = format!(
            "{}\n{}\n{}",
            &self.provider, &self.parameters, &self.context
        );

        hex::encode(env::keccak256(hash_str.as_bytes()).as_slice())
    }
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct CompleteClaimData {
    pub identifier: String,
    pub owner: String,
    pub epoch: u64,
    pub timestampS: u64,
}

impl CompleteClaimData {
    pub fn serialise(&self) -> String {
        format!(
            "0x{}\n0x{}\n{}\n{}",
            &self.identifier,
            &self.owner.to_string(),
            &self.timestampS.to_string(),
            &self.epoch.to_string()
        )
    }
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct SignedClaim {
    pub claim: CompleteClaimData,
    pub signatures: Vec<String>,
}

impl SignedClaim {
    pub fn recover_signers_of_signed_claim(self) -> Vec<String> {
        // Create empty array
        let mut expected = vec![];
        // Hash the signature
        let serialised_claim = self.claim.serialise();

        let bm = keccak256_eth(serialised_claim.as_str());
        let message_hash = bm.to_vec();

        // For each signature in the claim
        for complete_signature in self.signatures {
            let rec_param = complete_signature
                .get((complete_signature.len() as usize - 2)..(complete_signature.len() as usize))
                .unwrap();
            let mut mut_sig_str = complete_signature.clone();
            mut_sig_str.pop();
            mut_sig_str.pop();

            let rec_dec = hex::decode(rec_param).unwrap();
            let rec_norm = rec_dec.first().unwrap() - 27;
            let r_s = hex::decode(mut_sig_str).unwrap();

            let id = match rec_norm {
                0 => RecoveryId::new(false, false),
                1 => RecoveryId::new(true, false),
                2_u8..=u8::MAX => todo!(),
            };

            let signature = Signature::from_bytes(r_s.as_slice().into()).unwrap();

            // Recover the public key
            let verkey = VerifyingKey::recover_from_prehash(&message_hash, &signature, id).unwrap();
            let key: Vec<u8> = verkey.to_encoded_point(false).as_bytes().into();
            let hash = env::keccak256(&key[1..]);

            let address_bytes = hash.get(12..).unwrap();
            let public_key = append_0x(&hex::encode(address_bytes));
            expected.push(public_key);
        }
        expected
    }
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct Proof {
    pub claimInfo: ClaimInfo,
    pub signedClaim: SignedClaim,
}

pub fn keccak256_eth(message: &str) -> Vec<u8> {
    let message: &[u8] = message.as_ref();
    let mut eth_message = format!("\x19Ethereum Signed Message:\n{}", message.len()).into_bytes();
    eth_message.extend_from_slice(message);
    env::keccak256(&eth_message)
}

pub fn append_0x(content: &str) -> String {
    let mut initializer = String::from("0x");
    initializer.push_str(content);
    initializer
}
