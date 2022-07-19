use crate::*;
use near_sdk::promise_result_as_success;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
  pub owner_id: AccountId,
  pub approval_id: u64,
  pub nft_contract_id: String,
  pub token_id: String,
  pub sale_conditions: SalePriceInYoctoNear,
}

#[near_bindgen]
impl Contract {
  #[payable]
  pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String) {
    assert_one_yocto();
    let sale = self.internal_remove_sale(nft_contract_id.into(), token_id);
    let owner_id = env::predecessor_account_id();
    assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
  }

  #[payable]
  pub fn update_price(&mut self, nft_contract_id: AccountId, token_id: String, price: U128) {
    assert_one_yocto();
    let contract_id: AccountId = nft_contract_id.into();
    let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
    let mut sale = self.sales.get(&contract_and_token_id).expect("No Sale");
    assert_eq!(env::predecessor_account_id(), sale.owner_id, "Must be sale owner");
    sale.sale_conditions = price;
    self.sales.insert(&contract_and_token_id, &sale);
  }

  #[payable]
  pub fn offer(&mut self, nft_contract_id: AccountId, token_id: String) {
    let deposit = env::attached_deposit();
    assert!(deposit > 0, "Attached deposit must be greater than 0");
    let contract_id: AccountId = nft_contract_id.into();
    let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
    let sale = self.sales.get(&contract_and_token_id).expect("No sale");
    let buyer_id = env::predecessor_account_id();
    assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale."); 
    //(dot 0 converts from U128 to u128)
    let price = sale.sale_conditions.0;
    assert!(deposit >= price, "Attached deposit must be greater than or equal to the current price: {:?}", price);
    self.process_purchase(
      contract_id,
      token_id,
      U128(deposit),
      buyer_id,
    );
  }

  #[private]
  pub fn process_purchase(&mut self, nft_contract_id: AccountId, token_id: String, price: U128, buyer_id: AccountId) {
    let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());
    ext_contract::ext(nft_contract_id)
      .with_attached_deposit(1)
      .with_static_gas(GAS_FOR_NFT_TRANSFER)
      .nft_transfer_payout(
        buyer_id.clone(), 
        token_id, 
        sale.approval_id, 
        "payout from market".to_string(),
        price,
        10
      ).then(
        Self::ext(env::current_account_id())
          .with_static_gas(GAS_FOR_RESOLVE_PURCHASE)
          .resolve_purchase(
            buyer_id,
            price
          )
      );
  }

  #[private]
  pub fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> U128 {
    let payout_option = promise_result_as_success().and_then(|value| {
      near_sdk::serde_json::from_slice::<Payout>(&value)
        .ok()
        .and_then(|payout_object| {
          if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
            env::log_str("Cannot have more than 10 royalties");
            None
          } else {
            let mut remainder = price.0;
            for &value in payout_object.payout.values() {
              remainder = remainder.checked_sub(value.0)?;
            }
            if remainder == 0 || remainder == 1 {
              Some(payout_object.payout)
            } else {
              None
            }
          }
        })
    });
    let payout = if let Some(payout_option) = payout_option {
      payout_option
    } else {
      Promise::new(buyer_id).transfer(u128::from(price));
      return price;
    };

    for (receiver_id, amount) in payout {
      Promise::new(receiver_id).transfer(amount.0);
    }

    price
  }
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: U128,
    ) -> Promise;
}