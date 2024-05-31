# ratatui-wasm

This is a demo/POC of a ratatui backend based on crossterm that can run on both web and native environments with minimal implementation differences. It requires a [fork of crossterm](https://github.com/aschey/crossterm/tree/wasm) that has certain features disabled or stubbed out when building for wasm. I hope to get these changes merged upstream.

On the web, it runs on [xtermjs](http://xtermjs.org/) using [xterm-js-rs](https://github.com/segeljakt/xterm-js-rs). We can't spawn threads in the browser so we make use of crossterm's async input streams when running natively and [wasm-bindgen-futures](https://crates.io/crates/wasm-bindgen-futures) on the web.

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
