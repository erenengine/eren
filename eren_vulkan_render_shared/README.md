## Vulkan SDK 설정(맥)
```
export VULKAN_SDK=$HOME/VulkanSDK/1.4.313.1/macOS
export DYLD_FALLBACK_LIBRARY_PATH=$VULKAN_SDK/lib
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
```

## 테스트
```
RUST_LOG=debug cargo run --example test_instance
RUST_LOG=debug cargo run --example test_physical_device
RUST_LOG=debug cargo run --example test_device
RUST_LOG=debug cargo run --example test_pass1
RUST_LOG=debug cargo run --example test_pass2
```

## GLSL 컴파일
```
glslc examples/test_pass1/shaders/shader.vert -o examples/test_pass1/shaders/shader.vert.spv
glslc examples/test_pass1/shaders/shader.frag -o examples/test_pass1/shaders/shader.frag.spv
```