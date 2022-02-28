use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, AccountId, Balance, Gas, PanicOnDefault,
    Promise, PromiseResult, PublicKey,
};

use std::collections::HashMap;
use std::convert::TryFrom;
// extern crate cross_chain;
// extern crate node_evaluation;

use cross_chain::{Message, MessageVerify};
use node_evaluation::NodeCredibility;

const GAS_FOR_MSG_VERIFY: Gas = Gas(30_000_000_000_000);
const GAS_FOR_GET_NODES: Gas = Gas(20_000_000_000_000);
const GAS_FOR_CREDIBILITY_CALLBACK: Gas = Gas(30_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

pub trait MsgVerify {
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
    fn msg_verify(&mut self, msgs: Vec<MessageVerify>, percentage: u32) -> Promise;
}

#[ext_contract(ext_self)]
pub trait ContractCallback {
    fn credibility_callback(&mut self, msgs: Vec<MessageVerify>) -> Promise;

    fn result_callback(&mut self, msg: Vec<Message>) -> Vec<Message>;
}

#[ext_contract(ext_ec)]
pub trait EvaluationContract {
    fn get_nodes_credibility(&self, nodes: Vec<PublicKey>) -> Vec<PublicKey, u32>;
    fn update_nodes(
        &mut self,
        trusted: Vec<PublicKey>,
        untrusted: Vec<PublicKey>,
        exeception: Vec<(Vec<PublicKey>, u32)>,
    );
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // SETUP CONTRACT STATE
    node_ev_address: AccountId,
    cross_contract_id: AccountId,
    credibility_weight_threshold: u32,
    // aggregation_message:
}

#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(tag = "type", crate = "near_sdk::serde")]
pub struct GroupCredibility {
    // message: Message,
    group_credibility_value: u32,
    credibility_weight: u32,
    validators: Vec<PublicKey>,
}

#[near_bindgen]
impl Contract {
    // ADD CONTRACT METHODS HERE
    #[init]
    pub fn init(
        cross_contract_id: AccountId,
        node_eva_addr: AccountId,
        credibility_weight_threshold: u32,
    ) -> Self {
        Self {
            cross_contract_id,
            node_ev_address: node_eva_addr,
            credibility_weight_threshold: credibility_weight_threshold,
        }
    }

    #[private]
    pub fn credibility_callback(&self, msgs: Vec<MessageVerify>) -> Vec<Message> {
        require!(env::promise_results_count() == 1);
        let mut valid_message: Vec<Message> = Vec::new();
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                match near_sdk::serde_json::from_slice::<Vec<NodeCredibility>>(&result) {
                    Ok(validators_credibility) => {
                        // validate Messages
                        let mut aggregation_result: HashMap<String, (Message, GroupCredibility)> =
                            HashMap::new();
                        let mut credibility_map: HashMap<PublicKey, u32> = HashMap::new();
                        for vc in validators_credibility {
                            credibility_map.insert(
                                PublicKey::try_from(vc.validator).unwrap(),
                                vc.credibility_value,
                            );
                        }
                        let mut total_credibility = 0;
                        for msg in msgs {
                            let hash = msg.message.to_hash();
                            let pk = msg.validator.clone();
                            let credibility_value = credibility_map.get(&pk).unwrap_or(&0u32);
                            let group_info = aggregation_result.entry(hash).or_insert((
                                msg.message,
                                GroupCredibility {
                                    group_credibility_value: *credibility_value,
                                    credibility_weight: 0,
                                    validators: vec![pk.clone()],
                                },
                            ));
                            if !(*group_info).1.validators.contains(&pk) {
                                (*group_info).1.group_credibility_value += *credibility_value;
                                (*group_info).1.validators.push(pk);
                            }
                            total_credibility += credibility_value;
                            log!("total_credibility: {}", total_credibility);
                        }

                        let mut sort_vec: Vec<(Message, GroupCredibility)> = aggregation_result
                            .iter()
                            .map(|(_, value)| {
                                let mut return_value = value.clone();
                                return_value.1.credibility_weight =
                                    value.1.group_credibility_value * 10000 / total_credibility;
                                return_value
                            })
                            .collect();
                        sort_vec
                            .sort_by(|a, b| b.1.credibility_weight.cmp(&a.1.credibility_weight));
                        log!("sort_vec len: {}", sort_vec.len());
                        // let mut node_behaviors: Vec<NodeBehavior> = Vec::new();
                        let mut trusted: Vec<PublicKey> = Vec::new();
                        let mut untrusted: Vec<PublicKey> = Vec::new();
                        let mut exeception: Vec<(Vec<PublicKey>, u32)> = Vec::new();
                        log!(
                            "credibility_weight: {}, credibility_weight_threshold: {}",
                            sort_vec[0].1.credibility_weight,
                            self.credibility_weight_threshold
                        );
                        if sort_vec[0].1.credibility_weight >= self.credibility_weight_threshold {
                            valid_message.push(sort_vec[0].0.clone());
                            trusted = sort_vec.remove(0).1.validators;
                            for group in sort_vec {
                                untrusted.extend(group.1.validators);
                            }
                        } else {
                            for group in sort_vec {
                                exeception.push((group.1.validators, group.1.credibility_weight));
                            }
                        }
                        // let promise = Promise::new(self.node_ev_address);
                        ext_ec::update_nodes(
                            trusted,
                            untrusted,
                            exeception,
                            self.node_ev_address.clone(),
                            NO_DEPOSIT,
                            env::prepaid_gas() - GAS_FOR_CREDIBILITY_CALLBACK,
                        );
                    }
                    Err(err) => {
                        log!("resolve promise result failed, {}", err);
                        env::panic_str("in callback!, `from_slice` error!");
                    }
                }
            }
            _ => {
                env::panic_str("in callback!, but params error!");
            }
        }
        return valid_message;
    }

    #[private]
    pub fn result_callback(&mut self, msg: Vec<Message>) -> Vec<Message> {
        msg
    }
}

pub trait ToHash {
    fn to_hash(&self) -> String;
}

impl ToHash for Message {
    fn to_hash(&self) -> String {
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
impl MsgVerify for Contract {
    fn msg_verify(&mut self, msgs: Vec<MessageVerify>, percentage: u32) -> Promise {
        assert_ne!(env::predecessor_account_id(), self.cross_contract_id);
        let mut keys: Vec<PublicKey> = Vec::new();
        for value in msgs.iter() {
            keys.push(value.validator.clone());
            // keys.push(value.validator.into());
        }
        log!("msg_verify: {}", env::prepaid_gas().0);
        ext_ec::get_nodes_credibility(
            keys,
            self.node_ev_address.clone(),
            NO_DEPOSIT,
            GAS_FOR_GET_NODES,
        )
        .then(ext_self::credibility_callback(
            msgs,
            env::current_account_id(),
            0,
            env::prepaid_gas() - GAS_FOR_GET_NODES - GAS_FOR_MSG_VERIFY,
        ))
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
