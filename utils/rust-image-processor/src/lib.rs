#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn hello() -> String {
    "Hello from Rust!".to_string()
}

#[napi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
