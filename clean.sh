#!/bin/bash
cd parcel-plugin-cargo-web
npm run clean || true | tail -n5
cd ../parcel-plugin-rustwasm
npm run clean || true | tail -n5
cd ../parcel-plugin-wasm.rs
npm run clean || true | tail -n5
cd ../rollup-plugin-rust
npm run clean || true | tail -n5
cd ../webpack
npm run clean || true | tail -n5
