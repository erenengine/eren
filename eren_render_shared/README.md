## 제약 사항
- WebGPU/WebGL에서 push constants를 지원하지 않음
- WebGPU에서는 read_write storage buffer를 버텍스 셰이더에서 사용할 수 없음
- WebGL에서는 storage buffer를 사용할 수 없음

```
RUST_LOG=debug cargo run --example test_instance
RUST_LOG=debug cargo run --example test_device
RUST_LOG=debug cargo run --example test_pass
RUST_LOG=debug cargo run --example test_vertex_buffer
RUST_LOG=debug cargo run --example test_index_buffer
RUST_LOG=debug cargo run --example test_uniform_buffer
RUST_LOG=debug cargo run --example test_push_constants
RUST_LOG=debug cargo run --example test_storage_buffer
RUST_LOG=debug cargo run --example test_depth_buffer
RUST_LOG=debug cargo run --example test_compute_shader
RUST_LOG=debug cargo run --example test_shadow
RUST_LOG=debug cargo run --example test_texture
```

```
cargo build --example test_instance --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_instance.wasm

cargo build --example test_device --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_device.wasm

cargo build --example test_pass --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_pass.wasm

cargo build --example test_vertex_buffer --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_vertex_buffer.wasm

cargo build --example test_index_buffer --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_index_buffer.wasm

cargo build --example test_uniform_buffer --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_uniform_buffer.wasm

cargo build --example test_push_constants --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_push_constants.wasm

cargo build --example test_storage_buffer --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_storage_buffer.wasm

cargo build --example test_depth_buffer --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_depth_buffer.wasm

cargo build --example test_compute_shader --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_compute_shader.wasm

cargo build --example test_shadow --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_shadow.wasm

cargo build --example test_texture --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_texture.wasm
```