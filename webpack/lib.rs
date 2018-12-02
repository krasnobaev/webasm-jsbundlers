#[macro_use]
extern crate web_sys;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
  let window = web_sys::window().expect("should have a Window");
  let document = window.document().expect("should have a Document");

  let p: web_sys::Node = document.create_element("p")?.into();
  p.set_text_content(Some("Hello from Rust, WebAssembly, and Webpack!"));

  let body = document.body().expect("should have a body");
  let body: &web_sys::Node = body.as_ref();
  body.append_child(&p)?;

  Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
  a + b
}
