# examples of rust wasm building tools usage

This repository contains five examples of loading rust code to web application via compiling to WebAssembly.
All examples based on different bundlers and plugins.

## overview

### status

Sizes info is taken from the Finder. Build time is taken from terminal logs with exception of webpack which is arbitrary.

All examples provide calculation in rust and showing result in DOM.

Examples which use `wasm-bindgen` call `alert` function from JS in Rust.

Webpack example shows additional example of writting to DOM from Rust via `web-sys`.

|plugin                 |project size  |dist size/zip|deps in Cargo.toml + build time                     |status|
|-----------------------|--------------|-------------|----------------------------------------------------|------|
|parcel-plugin-cargo-web|87MiB         |1.4MiB/416KiB|no deps in Cargo.toml - ✨  Built in 3.87s.         |OK    |
|parcel-plugin-rustwasm |108MiB        |43KiB/17KiB  |wasm-bindgen          - ✨  Built in 124.13s.       |OK    |
|parcel-plugin-wasm.rs  |110MiB        |42KiB/17KiB  |wasm-bindgen          - ✨  Built in 95.29s.        |OK    |
|rollup-plugin-rust     |10MiB         |220KiB/35KiB |no deps in Cargo.toml - created dist/index.js in 2s|Require additional steps and revise, see section below|
|wasm-pack via webpack  |392.4MiB      |3.4MiB/975KiB|wasm-bindgen, web-sys - 68s                        |OK    |

### features

|plugin                 |livereload|async load|import .rs|import .toml|
|-----------------------|----------|----------|----------|------------|
|parcel-plugin-cargo-web|JS        |YES       |sync/async|NO          |
|parcel-plugin-rustwasm |JS        |YES       |sync/async|NO          |
|parcel-plugin-wasm.rs  |JS        |NO        |sync      |sync        |
|rollup-plugin-rust     |Won't Work|YES       |async     |NO          |
|wasm-pack via webpack  |JS/Rust   |YES<sup>*</sup>|NO   |NO          |

<sup>*</sup> webpack require loading of compiled wasm binary

## plugins info

### [parcel-plugin-cargo-web](https://www.npmjs.com/package/parcel-plugin-cargo-web) via [parcel-bundler](https://parceljs.org)

Initial example - https://github.com/koute/parcel-plugin-cargo-web/tree/master/example

See also - https://github.com/koute/stdweb/tree/master/examples/hasher-parcel

```bash
npm i
npm run start
```

### [parcel-plugin-rustwasm](https://www.npmjs.com/package/parcel-plugin-rustwasm) via [parcel-bundler](https://parceljs.org)

Initial example - https://github.com/proteamer/parcel-plugin-rustwasm/tree/master/example

```bash
npm i
npm run start
```

### [parcel-plugin-wasm.rs](https://www.npmjs.com/package/parcel-plugin-wasm.rs) via [parcel-bundler](https://parceljs.org)

Initial example - https://github.com/catsigma/parcel-plugin-wasm.rs

```bash
npm i
npm run start
```

### [rollup-plugin-rust](https://www.npmjs.com/package/rollup-plugin-rust) via [rollup](http://rollupjs.org)

Initial example - https://github.com/DrSensor/rollup-plugin-rust/tree/master/examples/node_wasm

```bash
npm i
npm run start
<CTRL>+C
#add 'window.global = window;' as first line of dist/index.js
#change 'Buffer' to 'Buffer$1' in dist/index.js
npm run start
```

### [@wasm-tool/wasm-pack-plugin](https://www.npmjs.com/package/@wasm-tool/wasm-pack-plugin) via [webpack](https://webpack.js.org)

Initial example - https://github.com/rustwasm/wasm-bindgen/tree/master/examples/add

```bash
npm i
npm run start
```

## see also

Project templates by Rust and WebAssembly working groups - https://rustwasm.github.io/book/reference/project-templates.html

Stencil - https://github.com/DrSensor/example-stencil-rust
