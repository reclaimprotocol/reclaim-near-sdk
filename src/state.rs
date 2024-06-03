use near_sdk::near;

#[near(serializers = [borsh, json])]
#[derive(Clone, Debug)]
pub struct Witness {
    pub address: String,
    pub host: String,
}

impl Witness {
    pub fn get_addresses(witness: Vec<Witness>) -> Vec<String> {
        let mut vec_addresses = vec![];
        for wit in witness {
            vec_addresses.push(format!("0x{}", wit.address));
        }
        vec_addresses
    }
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Epoch {
    pub id: u64,
    pub timestamp_start: u64,
    pub timestamp_end: u64,
    pub minimum_witness_for_claim_creation: u128,
    pub witnesses: Vec<Witness>,
}
