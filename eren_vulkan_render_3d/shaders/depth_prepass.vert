#version 450

// --- Vertex Attributes ---
layout(location = 0) in vec3 inPosition;

// --- Descriptor Sets ---
// Set 0: Per-Frame Data (UBO)
layout(set = 0, binding = 0, std140) uniform CameraUBO {
    mat4 view_proj;
    vec3 camera_pos; // 이 셰이더에선 사용하지 않으나, 이를 명시하는 것이 레이아웃 일관성과 명확성 측면에서 좋습니다.
    float _padding; // vec3 뒤 정렬을 위한 패딩
};

// --- Push Constants ---
// Per-Instance Data
layout(push_constant) uniform PushConstants {
    mat4 model;
    vec4 color; // 사용하지 않음
};

void main() {
    gl_Position = view_proj * model * vec4(inPosition, 1.0);
}
