use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::abi::Encodable;
use plasma_core::data_structure::state_object::StateObject;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn new_state_object(param: &JsValue) {
    if let Some(v) = param.as_string() {
        let state_object = StateObject::new(Address::zero(), Bytes::from(v));
        let _encoded = state_object.to_abi();
    };
}
