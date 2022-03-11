/*
 * @Description:
 * @Author: kay
 * @Date: 2022-02-24 15:38:34
 * @LastEditTime: 2022-03-10 17:57:21
 * @LastEditors: kay
 */
use crate::utils::{
    call_receive_message, create_message, init_no_macros as init, register_validators,
};
use cross_chain::{Message, MessageKey};
use near_sdk::serde_json::json;
use near_sdk_sim::DEFAULT_GAS;
use node_evaluation::NodeCredibility;

// #[test]
// pub fn simulate_get_node_credibility() {
//     let initail_credibiltiy_value: u32 = 4000u32;
//     let credibility_weight_threshold: u32 = 1000u32;
//     let (root, _, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
//     let (_, validators_pk) = register_validators(&root, 4);
//     let mut expect: Vec<NodeCredibility> = Vec::new();
//     for validator in validators_pk {
//         expect.push(NodeCredibility {
//             validator: validator,
//             credibility_value: initail_credibiltiy_value,
//         });
//     }
//     let reture_value: Vec<NodeCredibility> = ec
//         .view(
//             ec.account_id(),
//             "get_node",
//             &json!({"from_index": 0u32, "limit": 10u32})
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     assert_eq!(expect, reture_value);
// }

// #[test]
// pub fn simulate_receive_message() {
//     let initail_credibiltiy_value: u32 = 4000u32;
//     let credibility_weight_threshold: u32 = 1000u32;
//     let (root, cc, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
//     let (validators, _) = register_validators(&root, 1);
//     root.call(ec.account_id(), "select_validators", b"", DEFAULT_GAS, 0);
//     let (message_1, _) = create_message();

//     let received_message = vec![(&validators[..], &message_1)];
//     call_receive_message(received_message);
//     let reture_value: Vec<(MessageKey, Message)> = cc
//         .view(
//             cc.account_id(),
//             "get_executable_message",
//             &json!({"from_index": 0u32, "limit": 10u32})
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     // println!("executable message: {:?}", reture_value);
//     assert_eq!(message_1, reture_value[0].1);
// }

// test credibility < middle
#[test]
pub fn simulate_validator_crediblity_greater_middle_crediblity() {
    let initail_credibiltiy_value: u32 = 4000u32;
    let credibility_weight_threshold: u32 = 1000u32;
    let (root, cc, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
    let (validators, validators_pk) = register_validators(&root, 27);
    root.call(ec.account_id(), "select_validators", b"", DEFAULT_GAS, 0);
    let (message_1, _) = create_message();
    let received_message = vec![(&validators[..], &message_1)];
    call_receive_message(received_message);
    let return_value: Vec<(MessageKey, Message)> = cc
        .view(
            cc.account_id(),
            "get_executable_message",
            &json!({"from_index": 0u32, "limit": 20u32})
                .to_string()
                .into_bytes(),
        )
        .unwrap_json();
    assert_eq!(message_1, return_value[0].1);
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

// // test credibility >= middle
// #[test]
// pub fn simulate_credibility_eq_or_greater_than_middle() {
//     let initail_credibiltiy_value: u32 = 6000u32;
//     let credibility_weight_threshold: u32 = 1000u32;
//     let (root, cc, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
//     let (validators, validators_pk) = register_validators(&root, 10);
//     root.call(ec.account_id(), "select_validators", b"", DEFAULT_GAS, 0);
//     let (message_1, _) = create_message();
//     let received_message = vec![(&validators[..], &message_1)];
//     call_receive_message(received_message);

//     let return_value: Vec<(MessageKey, Message)> = cc
//         .view(
//             cc.account_id(),
//             "get_executable_message",
//             &json!({"from_index": 0u32, "limit": 20u32})
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     assert_eq!(message_1, return_value[0].1);
//     // check credibility value
//     let credibility_value: Vec<NodeCredibility> = ec
//         .view(
//             ec.account_id(),
//             "get_nodes_credibility",
//             &json!({ "nodes": validators_pk }).to_string().into_bytes(),
//         )
//         .unwrap_json();
//     let expect_value: u32 =
//         100 * (10000 - initail_credibiltiy_value) / 10000 + initail_credibiltiy_value;
//     for cv in credibility_value {
//         assert_eq!(expect_value, cv.credibility_value);
//     }
// }

// // test with untrusted node
// #[test]
// pub fn simulate_with_untrusted() {
//     let initail_credibiltiy_value: u32 = 6000u32;
//     let credibility_weight_threshold: u32 = 1000u32;
//     let (root, cc, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
//     let (validators, validators_pk) = register_validators(&root, 9);
//     root.call(ec.account_id(), "select_validators", b"", DEFAULT_GAS, 0);
//     let (message_1, message_2) = create_message();
//     let received_message = vec![
//         (&validators[..5], &message_1),
//         (&validators[5..], &message_2),
//     ];
//     call_receive_message(received_message);

//     let return_value: Vec<(MessageKey, Message)> = cc
//         .view(
//             cc.account_id(),
//             "get_executable_message",
//             &json!({"from_index": 0u32, "limit": 20u32})
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     assert_eq!(message_1, return_value[0].1);
//     assert_ne!(message_2, return_value[0].1);
//     // check credibility value
//     let trusted_credibility_value: Vec<NodeCredibility> = ec
//         .view(
//             ec.account_id(),
//             "get_nodes_credibility",
//             &json!({ "nodes": validators_pk[..5].to_vec() })
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     let expect_trusted_value: u32 =
//         100 * (10000 - initail_credibiltiy_value) / 10000 + initail_credibiltiy_value;
//     for cv in trusted_credibility_value {
//         assert_eq!(expect_trusted_value, cv.credibility_value);
//     }

//     let untrusted_credibility_value: Vec<NodeCredibility> = ec
//         .view(
//             ec.account_id(),
//             "get_nodes_credibility",
//             &json!({ "nodes": validators_pk[5..].to_vec() })
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     let expect_untrusted_value: u32 =
//         initail_credibiltiy_value - 200 * initail_credibiltiy_value / 10000;
//     for cv in untrusted_credibility_value {
//         assert_eq!(expect_untrusted_value, cv.credibility_value);
//     }
// }

// // test with inconsistency
// #[test]
// pub fn simulate_with_inconsistency() {
//     let initail_credibiltiy_value: u32 = 6000u32;
//     let credibility_weight_threshold: u32 = 6000u32;
//     let (root, cc, _, ec) = init(credibility_weight_threshold, initail_credibiltiy_value);
//     let (validators, validators_pk) = register_validators(&root, 9);
//     root.call(ec.account_id(), "select_validators", b"", DEFAULT_GAS, 0);
//     let (message_1, message_2) = create_message();
//     let exception_verify_message1 = (&validators[..5], &message_1);
//     let exception_verify_message2 = (&validators[5..], &message_2);
//     let received_message = vec![exception_verify_message1, exception_verify_message2];
//     call_receive_message(received_message);

//     let return_value: Vec<(MessageKey, Message)> = cc
//         .view(
//             cc.account_id(),
//             "get_executable_message",
//             &json!({"from_index": 0u32, "limit": 1u32})
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     assert_eq!(return_value, Vec::new());

//     let exception_total_credibility = initail_credibiltiy_value * validators_pk.len() as u32;
//     let exception_total_credibility1 =
//         initail_credibiltiy_value * exception_verify_message1.0.len() as u32;
//     let exception_credibility_weight1 =
//         10000 * exception_total_credibility1 / exception_total_credibility;

//     let exception_total_credibility2 =
//         initail_credibiltiy_value * exception_verify_message2.0.len() as u32;
//     let exception_credibility_weight2 =
//         10000 * exception_total_credibility2 / exception_total_credibility;

//     let exception_credibility1_value: Vec<NodeCredibility> = ec
//         .view(
//             ec.account_id(),
//             "get_nodes_credibility",
//             &json!({ "nodes": validators_pk[..5] })
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     let expect_exception_credibity1: u32 = initail_credibiltiy_value
//         - 100 * (initail_credibiltiy_value) / 10000 * (10000 - exception_credibility_weight1)
//             / 10000;
//     for cv in exception_credibility1_value {
//         assert_eq!(expect_exception_credibity1, cv.credibility_value);
//     }

//     let exception_credibility2_value: Vec<NodeCredibility> = ec
//         .view(
//             ec.account_id(),
//             "get_nodes_credibility",
//             &json!({ "nodes": validators_pk[5..] })
//                 .to_string()
//                 .into_bytes(),
//         )
//         .unwrap_json();
//     let expect_exception_credibity2: u32 = initail_credibiltiy_value
//         - 100 * (initail_credibiltiy_value) / 10000 * (10000 - exception_credibility_weight2)
//             / 10000;
//     for cv in exception_credibility2_value {
//         assert_eq!(expect_exception_credibity2, cv.credibility_value);
//     }
// }
