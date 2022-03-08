/*
 * @Description:
 * @Author: kay
 * @Date: 2022-02-24 15:38:34
 * @LastEditTime: 2022-03-08 17:59:02
 * @LastEditors: kay
 */
use crate::utils::{init_no_macros as init, register_validators, validator_generate_message};
use cross_chain::{Content, Message, MessageKey, MessageVerify, SQOS};
use near_sdk::serde_json::json;
use near_sdk_sim::DEFAULT_GAS;
use node_evaluation::NodeCredibility;

pub fn create_message() -> (Message, Message) {
    let message_1: Message = Message {
        from_chain: "OTHER_CHAIN".to_string(),
        to_chain: "NEAR_CHAIN".to_string(),
        sender: "OTHER_CHAIN_LOCKER".to_string(),
        signer: "OTHER_CHAIN_CALLER".to_string(),
        sqos: SQOS { reveal: false },
        content: Content {
            contract: "ft.shanks.testnet".to_string(),
            action: "ft_balance_of".to_string(),
            data: "{\"account_id\": \"shanks.testnet\"}".to_string(),
        }, // content: "alice".to_string(),
    };

    let message_2: Message = Message {
        from_chain: "OTHER_CHAIN".to_string(),
        to_chain: "NEAR_CHAIN".to_string(),
        sender: "OTHER_CHAIN_LOCKER".to_string(),
        signer: "OTHER_CHAIN_CALLER".to_string(),
        sqos: SQOS { reveal: false },
        content: Content {
            contract: "ft_shanks.testnet".to_string(),
            action: "ft_balance_of".to_string(),
            data: "{\"account_id\": \"other_account\"}".to_string(),
        }, // content: "alice".to_string(),
    };
    (message_1, message_2)
}

// pub fn create_message()

#[test]
pub fn simulate_receive_message() {
    let initail_credibiltiy_value: u32 = 4000u32;
    let credibility_weight_threshold: u32 = 1000u32;
    let (root, cc, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (validators, validators_pk) = register_validators(&root, 1);
    root.call(ec.account_id(), "select_validators", b"", DEFAULT_GAS, 0);
    let data = "{\"account_id\": \"shanks.testnet\"}".to_string();
    let id: u32 = 1;
    let msg = Message {
        from_chain: "Platon".to_string(),
        to_chain: "Near".to_string(),
        sender: "bob".to_string(),
        signer: "lili".to_string(),
        sqos: SQOS { reveal: false },
        content: Content {
            contract: "ft_shanks.testnet".to_string(),
            action: "ft_balance_of".to_string(),
            data,
        }, // content: "alice".to_string(),
    };

    validators[0]
        .call(
            cc.account_id(),
            "receive_message",
            &json!({"id": id, 
            "from_chain": msg.from_chain,
            "to_chain": msg.to_chain,
            "sender": msg.sender,
            "signer": msg.signer,
            "sqos": msg.sqos,
            "content": msg.content})
            .to_string()
            .into_bytes(),
            DEFAULT_GAS,
            0,
        )
        .assert_success();

    let reture_value: Vec<(MessageKey, Message)> = cc
        .view(
            cc.account_id(),
            "get_executable_message",
            &json!({"from_index": 0u32, "limit": 10u32})
                .to_string()
                .into_bytes(),
        )
        .unwrap_json();
    assert_eq!(msg, reture_value[0].1);
}

#[test]
pub fn simulate_get_node_credibility() {
    let initail_credibiltiy_value: u32 = 4000u32;
    let credibility_weight_threshold: u32 = 1000u32;
    let (root, _, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
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
    let (root, cc, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 5);
    let (message_1, _) = create_message();
    let verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk, message_1.clone());
    // TODO root should be cross contract
    let return_value: Vec<Message> = cc
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
    assert_eq!(message_1, return_value[0]);
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
    let (root, cc, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 5);

    let (message_1, _) = create_message();
    let verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk, message_1.clone());
    // TODO root should be cross contract
    let return_value: Vec<Message> = cc
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
    assert_eq!(message_1, return_value[0]);
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
    let (root, cc, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 9);
    let (message_1, message_2) = create_message();
    let trusted_verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk[..5], message_1.clone());
    // TODO root should be cross contract
    let untrustd_verify_message: Vec<MessageVerify> =
        validator_generate_message(&validators_pk[5..], message_2.clone());

    let mut verify_message: Vec<MessageVerify> = Vec::new();
    verify_message.extend(trusted_verify_message);
    verify_message.extend(untrustd_verify_message);
    let outcome = cc.call(
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
    assert_eq!(message_1, return_value[0]);
    assert_ne!(message_2, return_value[0]);
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

// test with untrusted node
#[test]
pub fn simulate_with_inconsistency() {
    let initail_credibiltiy_value: u32 = 6000u32;
    let credibility_weight_threshold: u32 = 6000u32;
    let (root, cc, vc, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (_, validators_pk) = register_validators(&root, 9);

    let (message_1, message_2) = create_message();
    let exception_verify_message1: Vec<MessageVerify> =
        validator_generate_message(&validators_pk[..5], message_1.clone());
    let exception_total_credibility = initail_credibiltiy_value * validators_pk.len() as u32;
    let exception_total_credibility1 =
        initail_credibiltiy_value * exception_verify_message1.len() as u32;
    let exception_credibility_weight1 =
        10000 * exception_total_credibility1 / exception_total_credibility;
    // TODO root should be cross contract
    let exception_verify_message2: Vec<MessageVerify> =
        validator_generate_message(&validators_pk[5..], message_2.clone());
    let exception_total_credibility2 =
        initail_credibiltiy_value * exception_verify_message2.len() as u32;
    let exception_credibility_weight2 =
        10000 * exception_total_credibility2 / exception_total_credibility;

    let mut verify_message: Vec<MessageVerify> = Vec::new();
    verify_message.extend(exception_verify_message1);
    verify_message.extend(exception_verify_message2);
    let outcome = cc.call(
        vc.account_id(),
        "msg_verify",
        &json!({ "msgs": verify_message, "percentage": 100})
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    // println!("{:#?}", outcome.promise_results());
    println!(
        "exception_credibility_weight1: {}, exception_credibility_weight2: {}",
        exception_credibility_weight1, exception_credibility_weight2
    );
    let return_value: Vec<Message> = outcome.unwrap_json();
    // println!("message: {:?}", return_value);
    assert_eq!(return_value, Vec::new());
    // check credibility value
    let exception_credibility1_value: Vec<NodeCredibility> = ec
        .view(
            ec.account_id(),
            "get_nodes_credibility",
            &json!({ "nodes": validators_pk[..5] })
                .to_string()
                .into_bytes(),
        )
        .unwrap_json();
    let expect_exception_credibity1: u32 = initail_credibiltiy_value
        - 100 * (initail_credibiltiy_value) / 10000 * (10000 - exception_credibility_weight1)
            / 10000;
    for cv in exception_credibility1_value {
        assert_eq!(expect_exception_credibity1, cv.credibility_value);
    }

    let exception_credibility2_value: Vec<NodeCredibility> = ec
        .view(
            ec.account_id(),
            "get_nodes_credibility",
            &json!({ "nodes": validators_pk[5..] })
                .to_string()
                .into_bytes(),
        )
        .unwrap_json();
    let expect_exception_credibity2: u32 = initail_credibiltiy_value
        - 100 * (initail_credibiltiy_value) / 10000 * (10000 - exception_credibility_weight2)
            / 10000;
    for cv in exception_credibility2_value {
        assert_eq!(expect_exception_credibity2, cv.credibility_value);
    }
}
