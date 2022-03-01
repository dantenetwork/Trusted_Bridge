/*
 * @Description:
 * @Author: kay
 * @Date: 2022-02-24 11:22:04
 * @LastEditTime: 2022-02-28 15:56:59
 * @LastEditors: kay
 */
// use msg_verify::Contract as VC;
// use node_evaluation::Contract as EC;

use near_sdk::serde_json::json;
use near_sdk::AccountId;
use near_sdk_sim::{init_simulator, to_yocto, UserAccount, DEFAULT_GAS};

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    VC_WASM_BYTES => "msg-verify/res/msg_verify.wasm",
    EC_WASM_BYTES => "./node-evaluation/res/node_evaluation.wasm",
}

const VC_ID: &str = "vc";
const EC_ID: &str = "ec";
const CC_ID: &str = "cc";
const VALIDATOR_ID: &str = "validator";

pub fn init_no_macros() -> (UserAccount, UserAccount, UserAccount, UserAccount) {
  let root = init_simulator(None);
  let vc = root.deploy(&VC_WASM_BYTES, VC_ID.parse().unwrap(), to_yocto("10"));

  // let vc = deploy!(
  //   contract: VC,
  //   contract_id: VC_ID,
  //   bytes: &VC_WASM_BYTES,
  //   signer_account: root,
  //   init_method: init(CC_ID.parse().unwrap(), EC_ID.parse().unwrap(), 1000u32)
  // );

  // let ec = deploy!(
  //   contract: EC,
  //   contract_id: EC_ID,
  //   bytes: &EC_WASM_BYTES,
  //   signer_account: root,
  //   init_method: inite(CC_ID.parse().unwrap(), VC_ID.parse().unwrap())
  // );

  let validator = root.create_user(
    AccountId::new_unchecked(VALIDATOR_ID.to_string()),
    to_yocto("100"),
  );
  vc.call(
    VC_ID.parse().unwrap(),
    "init",
    &json!({
      "cross_contract_id": CC_ID.parse::<AccountId>().unwrap(),
      "node_eva_addr": EC_ID.parse::<AccountId>().unwrap(),
      "credibility_weight_threshold": 1000u32,
    })
    .to_string()
    .into_bytes(),
    DEFAULT_GAS / 2,
    0,
  )
  .assert_success();

  let ec = root.deploy(&EC_WASM_BYTES, EC_ID.parse().unwrap(), to_yocto("10"));
  ec.call(
    EC_ID.parse().unwrap(),
    "inite",
    &json!({
      "cross_contract_id": CC_ID.parse::<AccountId>().unwrap(),
      "vc_contract_id": VC_ID.parse::<AccountId>().unwrap(),
    })
    .to_string()
    .into_bytes(),
    DEFAULT_GAS / 2,
    0,
  )
  .assert_success();
  (root, vc, ec, validator)
}

pub fn register(validator: &near_sdk_sim::UserAccount) {
  validator
    .call(
      EC_ID.parse::<AccountId>().unwrap(),
      "register_node",
      b"",
      near_sdk_sim::DEFAULT_GAS / 2,
      0,
    )
    .assert_success();
}
