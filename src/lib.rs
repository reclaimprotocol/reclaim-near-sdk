use near_sdk::json_types::U64;
use near_sdk::store::LookupMap;
use near_sdk::{env, log, near, require, AccountId, PanicOnDefault};

mod claims;
mod state;

use claims::*;
use state::{Epoch, Witness};
use utils::*;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Reclaim {
    owner_id: AccountId,
    current_epoch: u64,
    epochs: LookupMap<U64, Epoch>,
}

#[near]
impl Reclaim {
    #[init]
    pub fn init() -> Self {
        Self {
            owner_id: env::predecessor_account_id(),
            current_epoch: 0,
            epochs: LookupMap::new(b"reclaim_epochs".to_vec()),
        }
    }

    pub fn add_epoch(
        &mut self,
        minimum_witness_for_claim_creation: u128,
        epoch_start: u64,
        epoch_end: u64,
        witnesses: Vec<Witness>,
    ) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only the owner can add epochs"
        );
        require!(
            epoch_start < epoch_end,
            "Epoch start must be less than epoch end"
        );
        require!(
            minimum_witness_for_claim_creation > 0,
            "Minimum witness for claim creation must be positive"
        );
        self.current_epoch += 1;
        let epoch = Epoch {
            id: self.current_epoch,
            timestamp_start: epoch_start,
            timestamp_end: epoch_end,
            minimum_witness_for_claim_creation,
            witnesses,
        };
        self.epochs
            .insert(U64::from(self.current_epoch), epoch.clone());

        // log!("Adding epoch: {:?}", epoch);
    }

    pub fn get_epoch_by_id(&self, epoch_id: U64) -> Option<&Epoch> {
        self.epochs.get(&epoch_id)
    }

    #[handle_result]
    pub fn verify_proof(&mut self, proof: Proof) -> Result<(), &'static str> {
        let epoch_id_jsonable = U64::from(proof.signedClaim.claim.epoch);
        let fetched_epoch = self.epochs.get(&epoch_id_jsonable);
        match fetched_epoch {
            Some(epoch) => {
                let hashed = proof.claimInfo.hash();
                require!(
                    hashed == proof.signedClaim.claim.identifier,
                    "Identifier Mismatch"
                );

                let expected_witnesses = fetch_witness_for_claim(
                    epoch.clone(),
                    proof.signedClaim.claim.identifier.clone(),
                    0_u64,
                );

                let expected_witness_addresses = Witness::get_addresses(expected_witnesses);

                let signed_witness = proof.signedClaim.recover_signers_of_signed_claim();

                assert!(expected_witness_addresses.len() == signed_witness.len());
                log!("Signed Witnesses: {:?}", signed_witness);
                log!("Expected Witnesses: {:?}", expected_witness_addresses);
                // Ensure for every signature in the sign, a expected witness exists from the database
                for signed in signed_witness {
                    if !expected_witness_addresses.contains(&signed) {
                        return Err("Invalid Signature");
                    };
                }

                Ok(())
            }
            None => Err("Epoch Not Found"),
        }
    }
}

mod utils {
    use super::*;

    fn generate_random_seed(bytes: Vec<u8>, offset: usize) -> u32 {
        let hash_slice = &bytes[offset..offset + 4];
        let mut seed = 0u32;
        for (i, &byte) in hash_slice.iter().enumerate() {
            seed |= u32::from(byte) << (i * 8);
        }

        seed
    }

    pub fn fetch_witness_for_claim(
        epoch: Epoch,
        identifier: String,
        timestamp: u64,
    ) -> Vec<Witness> {
        let mut selected_witness = vec![];

        // Create a hash from identifier+epoch+minimum+timestamp
        let hash_str = format!(
            "{}\n{}\n{}\n{}",
            hex::encode(identifier),
            epoch.minimum_witness_for_claim_creation.to_string(),
            timestamp.to_string(),
            epoch.id.to_string()
        );
        let result = hash_str.as_bytes().to_vec();
        let hash_result = env::sha256(&result);
        let witenesses_left_list = epoch.witnesses;
        let mut byte_offset = 0;
        let witness_left = witenesses_left_list.len();
        for _i in 0..epoch.minimum_witness_for_claim_creation.into() {
            let random_seed = generate_random_seed(hash_result.clone(), byte_offset) as usize;
            let witness_index = random_seed % witness_left;
            let witness = witenesses_left_list.get(witness_index);
            match witness {
                Some(data) => selected_witness.push(data.clone()),
                None => {}
            }
            byte_offset = (byte_offset + 4) % hash_result.len();
        }

        selected_witness
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_epoch_is_none() {
        let contract = Reclaim::init();
        assert!(contract.get_epoch_by_id(U64::from(1)).is_none());
    }

    #[test]
    fn add_epoch() {
        let mut contract = Reclaim::init();
        let witnesses = vec![
            Witness {
                address: "test1.testnet".to_string(),
                host: "test1.testnet".to_string(),
            },
            Witness {
                address: "test2.testnet".to_string(),
                host: "test2.testnet".to_string(),
            },
        ];
        contract.add_epoch(1, 10000, 20000, witnesses);

        assert!(contract.get_epoch_by_id(U64::from(1)).is_some());

        let new_added_epoch = contract.get_epoch_by_id(U64::from(1)).unwrap();

        assert_eq!(new_added_epoch.id, 1);
        assert_eq!(new_added_epoch.witnesses.len(), 2);
        assert_eq!(new_added_epoch.witnesses[0].address, "test1.testnet");
        assert_eq!(new_added_epoch.witnesses[0].host, "test1.testnet");
        assert_eq!(new_added_epoch.minimum_witness_for_claim_creation, 1);
        assert_eq!(new_added_epoch.timestamp_start, 10000);
        assert_eq!(new_added_epoch.timestamp_end, 20000);
    }

    #[test]
    fn verify_proof() {
        let mut contract = Reclaim::init();
        let witnesses = vec![Witness {
            address: "244897572368eadf65bfbc5aec98d8e5443a9072".to_string(),
            host: "test1.testnet".to_string(),
        }];
        contract.add_epoch(1, 10000, 20000, witnesses);

        let claim_info = ClaimInfo {
            provider: "http".to_string(),
            parameters:"{\"body\":\"\",\"geoLocation\":\"in\",\"method\":\"GET\",\"responseMatches\":[{\"type\":\"contains\",\"value\":\"_steamid\\\">Steam ID: 76561199632643233</div>\"}],\"responseRedactions\":[{\"jsonPath\":\"\",\"regex\":\"_steamid\\\">Steam ID: (.*)</div>\",\"xPath\":\"id(\\\"responsive_page_template_content\\\")/div[@class=\\\"page_header_ctn\\\"]/div[@class=\\\"page_content\\\"]/div[@class=\\\"youraccount_steamid\\\"]\"}],\"url\":\"https://store.steampowered.com/account/\"}".to_string(),
            context: "{\"contextAddress\":\"user's address\",\"contextMessage\":\"for acmecorp.com on 1st january\"}".to_string()
        };

        let complete_claim_data = CompleteClaimData {
            identifier: "531322a6c34e5a71296a5ee07af13f0c27b5b1e50616f816374aff6064daaf55"
                .to_string(),
            owner: "e4c20c9f558160ec08106de300326f7e9c73fb7f".to_string(),
            epoch: 1,
            timestampS: 1710157447,
        };

        let sig = "52e2a591f51351c1883559f8b6c6264b9cb5984d0b7ccc805078571242166b357994460a1bf8f9903c4130f67d358d7d6e9a52df9a38c51db6a10574b946884c1b".to_string();
        let mut sigs = Vec::new();
        sigs.push(sig);

        let signed_claim = SignedClaim {
            claim: complete_claim_data,
            signatures: sigs,
        };

        let proof = Proof {
            claimInfo: claim_info,
            signedClaim: signed_claim,
        };

        contract.verify_proof(proof).unwrap();
    }
}
