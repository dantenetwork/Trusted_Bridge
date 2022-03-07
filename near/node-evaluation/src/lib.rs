use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, PublicKey};
// use near_sdk::json_types::{Base58PublicKey};

const MIN_CONFIDENCE: u32 = 0;
const MAX_CONFIDENCE: u32 = 10000;
const MIDDLE_CONFIDENCE: u32 = (MIN_CONFIDENCE + MAX_CONFIDENCE) / 2;
const RANGE: u32 = MAX_CONFIDENCE - MIN_CONFIDENCE;
const SUCCESS_STEP: u32 = 100;
const DO_EVIL_STEP: u32 = 200;
const EXECEPTION_STEP: u32 = 100;
const PRECISION: u32 = 10_000;

// For message verification
#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(tag = "type", crate = "near_sdk::serde")]
pub struct NodeCredibility {
    pub validator: PublicKey,
    pub credibility_value: u32,
}

pub trait NodeEvaluation {
    /// @notice Called from cross-chain node for re-selecting nodes for this time stage.
    ///
    /// @dev Refresh the begining and end of the current time stage if the current period ended.
    /// Cross contract call to `cross-chain protocol contract` to `reload_validators` new nodes
    fn select_validators(&self);

    /// @notice Called from `msg-verify`. Update node credibility by node behaviors after message verification.
    ///
    /// @dev Use node credibility evaluation algorithm.
    ///
    /// @param trusted, validators delivering the trusted message;
    /// @param untrusted, validators delivering the untrusted message;
    /// @param exeception, validators did not reach any agreement with verification message.
    fn update_nodes(
        &mut self,
        trusted: Vec<PublicKey>,
        untrusted: Vec<PublicKey>,
        exeception: Vec<(Vec<PublicKey>, u32)>,
    );

    /// @notice Called from `msg-verify`. Update node credibility by node behaviors after message verification.
    ///
    /// @dev Use node credibility evaluation algorithm.
    ///
    /// #param node_behaviors The behavior for nodes delivering one message. `True` means valid, `False` means invalid.
    // fn handle_exeception(&mut self, exeception_nodes: Vec<(PublicKey, u32)>);

    /// @notice Called from `msg verify contract` to get the credibilities of validators to take weighted aggregation verification of messages
    ///
    /// @dev
    /// @param nodes Validators
    fn get_nodes_credibility(&self, nodes: Vec<PublicKey>) -> Vec<NodeCredibility>;

    /// @notice Called from off-chain nodes to register themselves as the cross chain nodes.
    /// Get node address through `env::signer_account_id()`.
    fn register_node(&mut self);

    /// @notice Called from off-chain nodes to unregister.
    /// Get node address through `env::signer_account_id()`.
    fn unregister_node(&mut self);

    /// set the value of the credibility of the newly added validator
    fn set_initial_credibility(&mut self, value: u32);

    fn update_storage_date(&mut self, pk: PublicKey, value: u32);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // SETUP CONTRACT STATE
    cross_contract_id: AccountId,
    vc_contract_id: AccountId,
    initial_credibility_value: u32,
    max_trustworthy_ratio: u32,
    min_trustworthy_ratio: u32,
    min_seleted_threshold: u32,
    trustworthy_threshold: u32,
    node_credibility: UnorderedMap<PublicKey, u32>,
    trustworthy_validators: UnorderedMap<PublicKey, u32>,
}

#[near_bindgen]
impl Contract {
    // ADD CONTRACT METHODS HERE
    #[init]
    pub fn inite(
        cross_contract_id: AccountId,
        vc_contract_id: AccountId,
        initial_credibility_value: u32,
        max_trustworthy_ratio: u32,
        min_trustworthy_ratio: u32,
        min_seleted_threshold: u32,
        trustworthy_threshold: u32,
    ) -> Self {
        Self {
            cross_contract_id,
            vc_contract_id,
            initial_credibility_value,
            max_trustworthy_ratio,
            min_trustworthy_ratio,
            min_seleted_threshold,
            trustworthy_threshold,
            node_credibility: UnorderedMap::new(b'n'),
            trustworthy_validators: UnorderedMap::new(b't'),
        }
    }

    pub fn get_node(&self, from_index: u64, limit: u64) -> Vec<NodeCredibility> {
        let keys = self.node_credibility.keys_as_vector();
        let values = self.node_credibility.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, self.node_credibility.len()))
            .map(|index| NodeCredibility {
                validator: keys.get(index).unwrap(),
                credibility_value: values.get(index).unwrap(),
            })
            .collect()
    }
}

#[near_bindgen]
impl NodeEvaluation for Contract {
    fn get_nodes_credibility(&self, nodes: Vec<PublicKey>) -> Vec<NodeCredibility> {
        let mut current_node_credibility: Vec<NodeCredibility> = Vec::new();
        for node in nodes {
            // 是否可以
            // self.node_credibility.get(&node).unwrap();
            current_node_credibility.push(NodeCredibility {
                validator: node.clone(),
                credibility_value: self.node_credibility.get(&node).unwrap_or(0u32),
            })
        }
        current_node_credibility
    }

    fn set_initial_credibility(&mut self, value: u32) {
        self.initial_credibility_value = value;
    }

    // TODO delegation mechanism
    fn register_node(&mut self) {
        let pk = &env::signer_account_pk();
        match self.node_credibility.get(&pk) {
            None => {
                self.node_credibility
                    .insert(&pk, &self.initial_credibility_value);
            }
            _ => assert!(false, "already registered"),
        };
    }

    fn unregister_node(&mut self) {
        let pk = &env::signer_account_pk();
        self.node_credibility.remove(&pk);
    }

    fn select_validators(&self) {
        let mut trustworthy_sum: u32 = 0;
        let mut trustworthy_all: u32 = 0;
        for (_, value) in self.trustworthy_validators.iter() {
            trustworthy_sum += value
        }

        // probability of being selected
        let mut probability_seleted: Vec<(PublicKey, u32)> = Vec::new();
        for (validator, value) in self.trustworthy_validators.iter() {
            let probability = PRECISION * value / trustworthy_sum;
            if probability > self.trustworthy_threshold {
                trustworthy_all += probability;
            }
            probability_seleted.push((validator, probability));
        }
        let total_num = self.trustworthy_validators.len() as u32;
        let credibility_selected_num = total_num
            * std::cmp::max(
                std::cmp::min(trustworthy_all, self.max_trustworthy_ratio) as u32,
                self.min_trustworthy_ratio,
            );
        let random_selected_num = total_num - credibility_selected_num;
    }

    fn update_nodes(
        &mut self,
        trusted: Vec<PublicKey>,
        untrusted: Vec<PublicKey>,
        exeception: Vec<(Vec<PublicKey>, u32)>,
    ) {
        assert_eq!(
            env::predecessor_account_id(),
            self.vc_contract_id,
            "EVALUATION: Only call by vc contract"
        );
        let mut credibility_value: u32;
        // update current trusted validators credibility
        for validator in trusted {
            let origin_node_credibility = self.node_credibility.get(&validator).unwrap_or(0);
            if self.node_credibility.get(&validator).unwrap_or(0) < MIDDLE_CONFIDENCE {
                credibility_value = SUCCESS_STEP * (origin_node_credibility - MIN_CONFIDENCE)
                    / RANGE
                    + origin_node_credibility;
            } else {
                credibility_value = SUCCESS_STEP * (MAX_CONFIDENCE - origin_node_credibility)
                    / RANGE
                    + origin_node_credibility;
            }
            self.update_storage_date(validator, credibility_value);
        }

        // update current untrusted validators credibility
        for validator in untrusted {
            let origin_node_credibility = self.node_credibility.get(&validator).unwrap_or(0);
            credibility_value = origin_node_credibility
                - DO_EVIL_STEP * (origin_node_credibility - MIN_CONFIDENCE) / RANGE;
            self.update_storage_date(validator, credibility_value);
        }
        // update current exeception validators credibility
        for (validators, credibility_weight) in exeception {
            for validator in validators {
                let origin_node_credibility = self.node_credibility.get(&validator).unwrap_or(0);
                credibility_value = origin_node_credibility
                    - EXECEPTION_STEP * (origin_node_credibility - MIN_CONFIDENCE) / RANGE
                        * (10000 - credibility_weight)
                        / 10000;
                self.update_storage_date(validator, credibility_value);
            }
        }
    }

    // #[private]
    fn update_storage_date(&mut self, pk: PublicKey, value: u32) {
        if value < self.min_seleted_threshold {
            self.trustworthy_validators.remove(&pk);
        } else {
            self.trustworthy_validators.insert(&pk, &value);
        }
        self.node_credibility.insert(&pk, &value);
    }
}

// Fro NodeCredibility Display
impl std::fmt::Debug for NodeCredibility {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("NodeCredibility")
            .field("validator", &self.validator)
            .field("credibility_value", &self.credibility_value)
            .finish()
    }
}
