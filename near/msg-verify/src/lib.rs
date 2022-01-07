use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base58PublicKey, Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise,
    PromiseResult, PublicKey,
};

extern crate cross_chain;
extern crate node_evaluation;

use cross_chain::{Message, Content, MessageVerify};
use node_evaluation::{NodeBehavior};


const GAS_FOR_CALLBACK: Gas = Gas(5_000_000_000_000);
const GAS_FOR_RECEIVE_MESSAGE: Gas = Gas(25_000_000_000_000 + GAS_FOR_CALLBACK.0);
const NO_DEPOSIT: Balance = 0;
const CHAIN_NAME: &str = "NEAR";


pub trait MsgVerify{

    /// @notice Verify cross-chain message from multi-copies committed by multi-nodes.
    /// The message is valid unless there are at least `requires` copies being the same.
    /// 
    /// @dev Cross contract call to get the credibility of the validators from `node_evaluation contract`.
    /// Return to tell `cross-chain protocol contract` the result of the verification.
    /// Cross contract call to `node_evaluation contract` to update the credibility of the validators by their behavior.
    /// 
    /// @param msgs The message copies to be verified.
    /// @param percentage [0~10000]. Example: 9558 means 95.58%. Minimum percent of weights for the identical copies.
    /// The percentage is the weighted sum of identical copies according to the credibility of the validators.
    /// 
    /// @return The result of the verification. The `Vec` will be empty if failed.
    fn msg_verify(&mut self, msgs: Vec<MessageVerify>, percentage: u32) ->Vec<Message>;
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    // SETUP CONTRACT STATE
}

#[near_bindgen]
impl Contract {
    // ADD CONTRACT METHODS HERE
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // TESTS HERE
}
