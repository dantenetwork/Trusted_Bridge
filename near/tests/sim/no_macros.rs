/*
 * @Description:
 * @Author: kay
 * @Date: 2022-02-24 15:38:34
 * @LastEditTime: 2022-03-03 10:05:20
 * @LastEditors: kay
 */
use crate::utils::{init_no_macros as init, register_validators, validator_generate_message};
use cross_chain::{Content, Message, MessageVerify};
use near_sdk::serde_json::json;
use near_sdk_sim::DEFAULT_GAS;
use node_evaluation::NodeCredibility;

#[test]
pub fn simulate_get_node_credibility() {
    let initail_credibiltiy_value: u32 = 4000u32;
    let credibility_weight_threshold: u32 = 1000u32;
    let (root, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 4);
    let mut expect: Vec<NodeCredibility> = Vec::new();
    for validator in validators_pk {
        expect.push(NodeCredibility {
            validator: validator,
            credibility_value: initail_credibiltiy_value,
        });
    }
    let reture_value: Vec<NodeCredibility> = ec
        .view(
            ec.account_id(),
            "get_node",
            &json!({"from_index": 0u32, "limit": 10u32})
                .to_string()
                .into_bytes(),
        )
        .unwrap_json();
    assert_eq!(expect, reture_value);
}

// test credibility < middle
#[test]
pub fn simulate_verify_message() {
    let initail_credibiltiy_value: u32 = 4000u32;
    let credibility_weight_threshold: u32 = 1000u32;
    let (root, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 5);
    // use near_sdk_sim::near_crypto::InMemorySigner;
    let message = Message {
        from_chain: "CHAIN_1".to_string(),
        to_chain: "CHAIN_2".to_string(),
        sender: "SENDER_ADDRESS".to_string(),
        content: Content {
            contract: "CONTRACT_ADDRESS".to_string(),
            action: "ACTION".to_string(),
            data: "ACTION_PARAMETER".to_string(),
        },
    };

    let verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk, message.clone());
    // TODO root should be cross contract
    let return_value: Vec<Message> = root
        .call(
            vc.account_id(),
            "msg_verify",
            &json!({ "msgs": verify_message, "percentage": 100})
                .to_string()
                .into_bytes(),
            DEFAULT_GAS,
            0,
        )
        .unwrap_json();
    assert_eq!(message, return_value[0]);
    // check credibility value
    let credibility_value: Vec<NodeCredibility> = ec
        .view(
            ec.account_id(),
            "get_nodes_credibility",
            &json!({ "nodes": validators_pk }).to_string().into_bytes(),
        )
        .unwrap_json();
    let expect_value: u32 = 100 * initail_credibiltiy_value / 10000 + initail_credibiltiy_value;
    for cv in credibility_value {
        assert_eq!(expect_value, cv.credibility_value);
    }
}

// test credibility >= middle
#[test]
pub fn simulate_credibility_eq_or_greater_than_middle() {
    let initail_credibiltiy_value: u32 = 6000u32;
    let credibility_weight_threshold: u32 = 1000u32;
    let (root, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 5);
    // use near_sdk_sim::near_crypto::InMemorySigner;
    let message = Message {
        from_chain: "CHAIN_1".to_string(),
        to_chain: "CHAIN_2".to_string(),
        sender: "SENDER_ADDRESS".to_string(),
        content: Content {
            contract: "CONTRACT_ADDRESS".to_string(),
            action: "ACTION".to_string(),
            data: "ACTION_PARAMETER".to_string(),
        },
    };

    let verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk, message.clone());
    // TODO root should be cross contract
    let return_value: Vec<Message> = root
        .call(
            vc.account_id(),
            "msg_verify",
            &json!({ "msgs": verify_message, "percentage": 100})
                .to_string()
                .into_bytes(),
            DEFAULT_GAS,
            0,
        )
        .unwrap_json();
    assert_eq!(message, return_value[0]);
    // check credibility value
    let credibility_value: Vec<NodeCredibility> = ec
        .view(
            ec.account_id(),
            "get_nodes_credibility",
            &json!({ "nodes": validators_pk }).to_string().into_bytes(),
        )
        .unwrap_json();
    let expect_value: u32 =
        100 * (10000 - initail_credibiltiy_value) / 10000 + initail_credibiltiy_value;
    for cv in credibility_value {
        assert_eq!(expect_value, cv.credibility_value);
    }
}

// test with untrusted node
#[test]
pub fn simulate_with_untrusted() {
    let initail_credibiltiy_value: u32 = 6000u32;
    let credibility_weight_threshold: u32 = 1000u32;
    let (root, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 9);
    // use near_sdk_sim::near_crypto::InMemorySigner;
    let trusted_message = Message {
        from_chain: "CHAIN_1".to_string(),
        to_chain: "CHAIN_2".to_string(),
        sender: "SENDER_ADDRESS".to_string(),
        content: Content {
            contract: "CONTRACT_ADDRESS".to_string(),
            action: "ACTION".to_string(),
            data: "ACTION_PARAMETER".to_string(),
        },
    };

    let untrusted_message = Message {
        from_chain: "CHAIN_1".to_string(),
        to_chain: "CHAIN_2".to_string(),
        sender: "SENDER_ADDRESS".to_string(),
        content: Content {
            contract: "UNTRUSTED_CONTRACT_ADDRESS".to_string(),
            action: "ACTION".to_string(),
            data: "ACTION_PARAMETER".to_string(),
        },
    };
    let trusted_verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk[..5], trusted_message.clone());
    // TODO root should be cross contract
    let untrustd_verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk[5..], untrusted_message.clone());

    let mut verify_message: Vec<MessageVerify> = Vec::new();
    verify_message.extend(trusted_verify_message);
    verify_message.extend(untrustd_verify_message);
    let outcome = root.call(
        vc.account_id(),
        "msg_verify",
        &json!({ "msgs": verify_message, "percentage": 100})
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    // println!("{:#?}", outcome.promise_results());
    let return_value: Vec<Message> = outcome.unwrap_json();
    assert_eq!(trusted_message, return_value[0]);
    assert_ne!(untrusted_message, return_value[0]);
    // check credibility value
    let trusted_credibility_value: Vec<NodeCredibility> = ec
        .view(
            ec.account_id(),
            "get_nodes_credibility",
            &json!({ "nodes": validators_pk[..5].to_vec() })
                .to_string()
                .into_bytes(),
        )
        .unwrap_json();
    let expect_trusted_value: u32 =
        100 * (10000 - initail_credibiltiy_value) / 10000 + initail_credibiltiy_value;
    for cv in trusted_credibility_value {
        assert_eq!(expect_trusted_value, cv.credibility_value);
    }

    let untrusted_credibility_value: Vec<NodeCredibility> = ec
        .view(
            ec.account_id(),
            "get_nodes_credibility",
            &json!({ "nodes": validators_pk[5..].to_vec() })
                .to_string()
                .into_bytes(),
        )
        .unwrap_json();
    let expect_untrusted_value: u32 =
        initail_credibiltiy_value - 200 * initail_credibiltiy_value / 10000;
    for cv in untrusted_credibility_value {
        assert_eq!(expect_untrusted_value, cv.credibility_value);
    }
}
