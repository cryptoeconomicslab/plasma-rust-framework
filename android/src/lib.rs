#![cfg(target_os = "android")]
#![allow(non_snake_case)]

use bytes::Bytes;
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

    let parameters_bytes = Bytes::from(&b"parameters"[..]);
    let state_object = StateObject::new(Address::zero(), &parameters_bytes);
    let _encoded = rlp::encode(&state_object);

    let output = env
        .new_string("Hello ".to_owned() + recipient.to_str().unwrap())
        .unwrap();
    output.into_inner()
}
