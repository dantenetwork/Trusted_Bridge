use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{self, json};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, AccountId, Balance, Gas, PanicOnDefault, Promise,
    PromiseResult, PublicKey,
};

extern crate cross_chain;
extern crate node_evaluation;

use cross_chain::{Message, Content, MessageVerify};
use node_evaluation::{NodeBehavior};

const GAS_FOR_FUNCTION_CALL: Gas = Gas(5_000_000_000_000);
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
    fn msg_verify(&mut self, msgs: std::collections::hash_map::HashMap<PublicKey, Message>, percentage: u32) -> Promise;
}

#[ext_contract(ext_self)]
pub trait ContractCallback{
    fn credibility_callback(&mut self, 
        msgs: std::collections::hash_map::HashMap<PublicKey, Message>,
    )->Promise;

    fn result_callback(&mut self, msg: Vec<Message>) -> Vec<Message>;
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // SETUP CONTRACT STATE
    node_ev_address: AccountId,
}

#[near_bindgen]
impl Contract {
    // ADD CONTRACT METHODS HERE
    #[init]
    pub fn new(node_eva_addr: AccountId) ->Self{
        Self{
            node_ev_address: node_eva_addr,
        }
    }

    #[private]
    fn credibility_callback(&mut self, 
        msgs: std::collections::hash_map::HashMap<PublicKey, Message>,
    )->Promise{
        require!(env::promise_results_count() == 1);
        match env::promise_result(0){
            PromiseResult::Successful(result) =>{
                match near_sdk::serde_json::from_slice::<std::collections::hash_map::HashMap<PublicKey, u32>>(&result) {
                    Ok(validator_map) => {
                        // validate Messages

                        // cross-call `node_evaluation::update_nodes`, set callback as `result_callback`
                        Promise::new(self.node_ev_address.clone())
                        
                    }
                    Err(err) => {
                        log!("resolve promise result failed, {}", err);
                        env::panic_str("in callback!, `from_slice` error!");
                    }
                }
            }
            _ =>{
                env::panic_str("in callback!, but params error!");
            }
        }
    }

    #[private]
    fn result_callback(&mut self, msg: Vec<Message>)->Vec<Message>{
        msg
    }
}

pub trait ToHash{
    fn to_hash(&self)->String;
}

impl ToHash for Message{
    fn to_hash(&self)->String{
        let mut s = self.from_chain.clone();
        s += &self.to_chain.clone();
        s += &self.sender.clone();
        s += &self.content.action.clone();
        s += &self.content.contract.clone();
        s += &self.content.data.clone();

        s = hex::encode(env::sha256(s.as_bytes()));
        s
    }
}

#[near_bindgen]
impl MsgVerify for Contract{
    fn msg_verify(&mut self, msgs: std::collections::hash_map::HashMap<PublicKey, Message>, percentage: u32) -> Promise{
        let mut keys = Vec::new();
        for (key, _) in msgs.iter(){
            keys.push(key);
        }

        Promise::new(self.node_ev_address.clone())
        .function_call("get_nodes_credibility".to_string(), 
            json!({"nodes": keys}).to_string().as_bytes().to_vec(), 
            0, 
            GAS_FOR_FUNCTION_CALL)
        .then(ext_self::credibility_callback(msgs, env::current_account_id(), 0, GAS_FOR_CALLBACK))
    }
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
