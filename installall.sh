#!/bin/bash
cd parcel-plugin-cargo-web
npm i || true
cd ../parcel-plugin-rustwasm
npm i || true
cd ../parcel-plugin-wasm.rs
npm i || true
cd ../rollup-plugin-rust
npm i || true
cd ../webpack
npm i || true
