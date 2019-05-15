// extern crate wasm_bindgen;

// use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern {
//   fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet(name: &str) {
//   alert(&format!("Hello, {}!", name));
// }

// #[wasm_bindgen]
#[no_mangle]
pub fn add(a: i32, b: i32) -> i32 {
  return a + b
}
