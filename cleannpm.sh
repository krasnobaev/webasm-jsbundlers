#!/bin/bash
cd parcel-plugin-cargo-web
rm -r ./node_modules || true
cd ../parcel-plugin-rustwasm
rm -r ./node_modules || true
cd ../parcel-plugin-wasm.rs
rm -r ./node_modules || true
cd ../rollup-plugin-rust
rm -r ./node_modules || true
cd ../webpack
rm -r ./node_modules || true
