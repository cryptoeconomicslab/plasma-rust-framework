#![cfg(target_os = "android")]
#![allow(non_snake_case)]

use ethereum_types::Address;
use jni::objects::{JObject, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use plasma_core::data_structure::state_object::StateObject;
use std::ffi::{CStr, CString};

#[no_mangle]
pub unsafe extern "C" fn Java_com_example_android_MainActivity_hello(
    env: JNIEnv,
    _: JObject,
    j_recipient: JString,
) -> jstring {
    let recipient = CString::from(CStr::from_ptr(
        env.get_string(j_recipient).unwrap().as_ptr(),
    ));

    let parameters_bytes = Vec::from(&b"parameters"[..]);
    let state_object = StateObject::from_range(
        0,
        100,
        Address::zero(),
        &parameters_bytes);
    let _encoded = state_object.to_abi();

    let output = env
        .new_string("Hello ".to_owned() + recipient.to_str().unwrap())
        .unwrap();
    output.into_inner()
}
