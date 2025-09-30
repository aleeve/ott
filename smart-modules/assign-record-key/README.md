# Fluvio SmartModules

> **Note**: To compile a SmartModule, you will need to install the `wasm32-wasip1`
> target by running `rustup target add wasm32-wasip1`, or `rustup target add wasm32-unknown-unknown` if using the --nowasi flags

## About SmartModules

Fluvio SmartModules are custom plugins that can be used to manipulate
streaming data in a topic. SmartModules are written in Rust and compiled
to WebAssembly. To use a SmartModule, you simply provide the `.wasm` file
to the Fluvio consumer, which uploads it to the Streaming Processing Unit
(SPU) where it runs your SmartModule code on each record before sending
it to the consumer.

## Using SmartModules with the Fluvio CLI

Make sure to follow the [Fluvio getting started] guide, then create a new
topic to send data to.

[Fluvio getting started]: https://www.fluvio.io/docs/fluvio/quickstart

```bash
fluvio topic create smartmodule-test
cargo build --release
fluvio consume smartmodule-test -B --map="target/wasm32-wasip1/release-lto/assign-record-key"
```
