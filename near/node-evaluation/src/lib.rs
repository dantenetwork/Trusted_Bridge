use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{Base58PublicKey};

// For message verification
#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(tag = "type", crate = "near_sdk::serde")]
pub struct NodeBehavior {
    validator: Base58PublicKey,
    behavior: bool,
}

pub trait NodeEvaluation{

    /// @notice Called from cross-chain node for re-selecting nodes for this time stage.
    /// 
    /// @dev Refresh the begining and end of the current time stage if the current period ended. 
    /// Cross contract call to `cross-chain protocol contract` to `reload_validators` new nodes
    fn select_validators(&mut self);

    /// @notice Called from `msg-verify`. Update node credibility by node behaviors after message verification.
    /// 
    /// @dev Use node credibility evaluation algorithm.
    /// 
    /// #param node_behaviors The behavior for nodes delivering one message. `True` means valid, `False` means invalid.
    fn update_nodes(&mut self, node_behaviors: Vec<NodeBehavior>);

    /// @notice Called from `msg verify contract` to get the credibilities of validators to take weighted aggregation verification of messages
    /// 
    /// @dev 
    /// @param nodes Validators
    fn get_nodes_credibility(&self, nodes: Vec<Base58PublicKey>) ->std::collections::hash_map::HashMap<Base58PublicKey, u32>;

    /// @notice Called from off-chain nodes to register themselves as the cross chain nodes. 
    /// Get node address through `env::signer_account_id()`. 
    /// 
    /// @dev    
    /// 
    /// @return True/False
    fn register_node(&mut self) ->bool;

    /// @notice Called from off-chain nodes to unregister. 
    /// Get node address through `env::signer_account_id()`. 
    /// 
    /// @dev    
    /// 
    /// @return True/False
    fn unregister_node(&mut self) ->bool;
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
