## Vulkan SDK 설정(맥)
```
export VULKAN_SDK=$HOME/VulkanSDK/1.4.313.1/macOS
export DYLD_FALLBACK_LIBRARY_PATH=$VULKAN_SDK/lib
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
```

## 쉐이더 컴파일
```
glslc shaders/depth_prepass.vert -o shaders/depth_prepass.vert.spv
```

## 테스트
```
RUST_LOG=debug cargo run --example test_depth_prepass
```