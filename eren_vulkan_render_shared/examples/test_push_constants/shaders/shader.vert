#version 450

// Push Constant for per-object transform
layout(push_constant) uniform PushConstants {
    mat4 model;
} pushModel;

// UBO for camera data (less frequent updates)
layout(binding = 0) uniform CameraUBO {
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = ubo.proj * ubo.view * pushModel.model * vec4(inPosition, 0.0, 1.0);
    fragColor = inColor;
}
