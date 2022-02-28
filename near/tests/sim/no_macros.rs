/*
 * @Description:
 * @Author: kay
 * @Date: 2022-02-24 15:38:34
 * @LastEditTime: 2022-02-28 17:01:06
 * @LastEditors: kay
 */
use crate::utils::{init_no_macros as init, register};
use cross_chain::{Content, Message, MessageVerify};
use near_sdk::serde_json::json;
use near_sdk::PublicKey;
use near_sdk_sim::DEFAULT_GAS;
use node_evaluation::NodeCredibility;
use std::convert::TryFrom;
use std::str::FromStr;

#[test]
pub fn simulate_get_node_credibility() {
  let (_, _, ec, validator) = init();
  // log!("len: {}", validator.signer.public_key);
  register(&validator);
  let validators: Vec<NodeCredibility> = ec
    .view(
      ec.account_id(),
      "get_node",
      &json!({"from_index": 0u32, "limit": 10u32})
        .to_string()
        .into_bytes(),
    )
    .unwrap_json();
  assert_eq!(
    String::try_from(&validators[0].validator).unwrap(),
    validator.signer.public_key.to_string()
  );
}

#[test]
pub fn simulate_verify_message() {
  let (root, vc, _, validator) = init();
  register(&validator);
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
  // TODO root should be cross contract
  let mut msg: Vec<MessageVerify> = Vec::new();
  let pk = format!("{}", validator.signer.public_key);
  msg.push(MessageVerify {
    validator: PublicKey::from_str(&pk).unwrap(),
    message: message.clone(),
  });
  let return_value: Vec<Message> = root
    .call(
      vc.account_id(),
      "msg_verify",
      &json!({ "msgs": msg, "percentage": 100})
        .to_string()
        .into_bytes(),
      DEFAULT_GAS,
      0,
    )
    .unwrap_json();
  assert_eq!(message, return_value[0]);
}
