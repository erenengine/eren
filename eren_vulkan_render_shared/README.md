## Vulkan SDK 설정(맥)

## 쉐이더 컴파일
```
glslc examples/test_pass/shaders/shader.vert -o examples/test_pass/shaders/shader.vert.spv
glslc examples/test_pass/shaders/shader.frag -o examples/test_pass/shaders/shader.frag.spv

glslc examples/test_vertex_buffer/shaders/shader.vert -o examples/test_vertex_buffer/shaders/shader.vert.spv
glslc examples/test_vertex_buffer/shaders/shader.frag -o examples/test_vertex_buffer/shaders/shader.frag.spv

glslc examples/test_index_buffer/shaders/shader.vert -o examples/test_index_buffer/shaders/shader.vert.spv
glslc examples/test_index_buffer/shaders/shader.frag -o examples/test_index_buffer/shaders/shader.frag.spv

glslc examples/test_uniform_buffer/shaders/shader.vert -o examples/test_uniform_buffer/shaders/shader.vert.spv
glslc examples/test_uniform_buffer/shaders/shader.frag -o examples/test_uniform_buffer/shaders/shader.frag.spv

glslc examples/test_push_constants/shaders/shader.vert -o examples/test_push_constants/shaders/shader.vert.spv
glslc examples/test_push_constants/shaders/shader.frag -o examples/test_push_constants/shaders/shader.frag.spv

glslc examples/test_storage_buffer/shaders/shader.vert -o examples/test_storage_buffer/shaders/shader.vert.spv
glslc examples/test_storage_buffer/shaders/shader.frag -o examples/test_storage_buffer/shaders/shader.frag.spv

glslc examples/test_depth_buffer/shaders/shader.vert -o examples/test_depth_buffer/shaders/shader.vert.spv
glslc examples/test_depth_buffer/shaders/shader.frag -o examples/test_depth_buffer/shaders/shader.frag.spv

glslc examples/test_compute_shader/shaders/shader.comp -o examples/test_compute_shader/shaders/shader.comp.spv
glslc examples/test_compute_shader/shaders/shader.vert -o examples/test_compute_shader/shaders/shader.vert.spv
glslc examples/test_compute_shader/shaders/shader.frag -o examples/test_compute_shader/shaders/shader.frag.spv

glslc examples/test_shadow/shaders/shadow.vert -o examples/test_shadow/shaders/shadow.vert.spv
glslc examples/test_shadow/shaders/debug_quad.vert -o examples/test_shadow/shaders/debug_quad.vert.spv
glslc examples/test_shadow/shaders/debug_quad.frag -o examples/test_shadow/shaders/debug_quad.frag.spv
glslc examples/test_shadow/shaders/main.vert -o examples/test_shadow/shaders/main.vert.spv
glslc examples/test_shadow/shaders/main.frag -o examples/test_shadow/shaders/main.frag.spv

glslc examples/test_texture/shaders/shader.vert -o examples/test_texture/shaders/shader.vert.spv
glslc examples/test_texture/shaders/shader.frag -o examples/test_texture/shaders/shader.frag.spv
```

## 테스트
```
export VULKAN_SDK=$HOME/VulkanSDK/1.4.313.1/macOS
export DYLD_FALLBACK_LIBRARY_PATH=$VULKAN_SDK/lib
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
```
```
RUST_LOG=debug cargo run --example test_instance
RUST_LOG=debug cargo run --example test_physical_device
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
