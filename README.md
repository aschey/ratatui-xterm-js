# ratatui-wasm

This is a demo/POC of a ratatui backend based on crossterm that can run on both web and native environments. It requires a [fork of crossterm](https://github.com/aschey/crossterm/tree/wasm) that has some wasm-incomatible code disabled when building for wasm.

On the web, it runs on [xtermjs](http://xtermjs.org/) using [xterm-js-rs](https://github.com/segeljakt/xterm-js-rs).

To run the demos (requires [wasm-pack](https://github.com/rustwasm/wasm-pack)):

```bash
cd ./examples/simple
npm install
cd www
npm install
wasm-pack build --all-features
# run wasm build
npm run start
# or run native build
cargo run
```

```bash
cd ./examples/inline
npm install
cd www
npm install
wasm-pack build --all-features
# run wasm build
npm run start
# or run native build
cargo run
```