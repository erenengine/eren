```
cargo run --example test_instance
cargo run --example test_device
cargo run --example test_pass1
```

```
cargo build --example test_instance --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_instance.wasm

cargo build --example test_device --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_device.wasm

cargo build --example test_pass1 --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_pass1.wasm
```