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

// For message verification
#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(tag = "type", crate = "near_sdk::serde")]
pub struct NodeBehavior {
    pub validator: PublicKey,
    pub behavior: bool,
}

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
    ///
    /// @dev    
    ///
    /// @return True/False
    // fn register_node(&mut self) ->bool;

    /// @notice Called from off-chain nodes to unregister.
    /// Get node address through `env::signer_account_id()`.
    ///
    /// @dev    
    ///
    /// @return True/False
    fn unregister_node(&mut self) -> bool;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // SETUP CONTRACT STATE
    cross_contract_id: AccountId,
    vc_contract_id: AccountId,
    node_credibility: UnorderedMap<PublicKey, u32>,
}

#[near_bindgen]
impl Contract {
    // ADD CONTRACT METHODS HERE
    #[init]
    pub fn inite(cross_contract_id: AccountId, vc_contract_id: AccountId) -> Self {
        Self {
            cross_contract_id,
            vc_contract_id,
            node_credibility: UnorderedMap::new(b'n'),
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

    fn unregister_node(&mut self) -> bool {
        true
    }

    fn select_validators(&self) {}

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
            if self.node_credibility.get(&validator).unwrap_or(0) >= MIDDLE_CONFIDENCE {
                credibility_value = SUCCESS_STEP * (origin_node_credibility - MIN_CONFIDENCE)
                    / RANGE
                    + origin_node_credibility;
            } else {
                credibility_value = SUCCESS_STEP * (MAX_CONFIDENCE - origin_node_credibility)
                    / RANGE
                    + origin_node_credibility;
            }
            self.node_credibility.insert(&validator, &credibility_value);
        }

        // update current untrusted validators credibility
        for validator in untrusted {
            let origin_node_credibility = self.node_credibility.get(&validator).unwrap_or(0);
            credibility_value = DO_EVIL_STEP * (MIN_CONFIDENCE - origin_node_credibility) / RANGE
                + origin_node_credibility;
            self.node_credibility.insert(&validator, &credibility_value);
        }
        // update current exeception validators credibility
        for (validators, credibility_weight) in exeception {
            for validator in validators {
                let origin_node_credibility = self.node_credibility.get(&validator).unwrap_or(0);
                credibility_value = EXECEPTION_STEP * (MIN_CONFIDENCE - origin_node_credibility)
                    / RANGE
                    * (10000 - credibility_weight)
                    / 10000
                    + origin_node_credibility;
                self.node_credibility.insert(&validator, &credibility_value);
            }
        }
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
