use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn reverse_buff32(input: Vec<u8>) -> Vec<u8> {
    let mut result = input;
    result.reverse();
    result
}
