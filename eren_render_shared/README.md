```
RUST_LOG=debug cargo run --example test_instance
RUST_LOG=debug cargo run --example test_device
RUST_LOG=debug cargo run --example test_pass
RUST_LOG=debug cargo run --example test_vertex_buffer
RUST_LOG=debug cargo run --example test_uniform
```

```
cargo build --example test_instance --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_instance.wasm

cargo build --example test_device --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_device.wasm

cargo build --example test_pass --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_pass.wasm

cargo build --example test_vertex_buffer --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_vertex_buffer.wasm

cargo build --example test_uniform --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_uniform.wasm
```