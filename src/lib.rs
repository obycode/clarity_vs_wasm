use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn add128(a: i128, b: i128) -> i128 {
    a + b
}

#[wasm_bindgen]
pub fn reverse_buff32(input: Vec<u8>) -> Vec<u8> {
    let mut result = input;
    result.reverse();
    result
}
