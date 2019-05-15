#!/bin/bash
cd parcel-plugin-cargo-web
npm run build || true
cd ../parcel-plugin-rustwasm
npm run build || true
cd ../parcel-plugin-wasm.rs
npm run build || true
cd ../rollup-plugin-rust
mkdir -p dist && npm run build || true
cd ../webpack
npm run build || true
