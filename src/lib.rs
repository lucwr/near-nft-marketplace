use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near_bindgen, AccountId, PanicOnDefault
};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    sales: UnorderedMap<>,
    by_owner_id: LookupMap<>,
    by_nft_contract_id: LookupMap<>,
    storage_deposits: LookupMap<>,
}

#[derive(BorshSerialize)]
pub enum StorageKey {
    Sales,
    ByOwnerId,
    ByNFTContractId,
    StorageDeposits,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            sales: UnorderedMap::new(StorageKey::Sales),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId),
            by_nft_contract_id: LookupMap::new(StorageKey::ByNFTContractId),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
        }
    }
}