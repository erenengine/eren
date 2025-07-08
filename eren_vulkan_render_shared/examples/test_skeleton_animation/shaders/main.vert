#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(set = 0, binding = 0) uniform MainUBO {
    mat4 model;
    mat4 view;
    mat4 proj;
    mat4 lightViewProj; // 그림자 맵 투영을 위한 행렬
} ubo;

layout(location = 0) out vec3 fragPosWorld;
layout(location = 1) out vec3 normalWorld;
layout(location = 2) out vec4 shadowCoord;
layout(location = 3) out vec2 fragTexCoord;

void main() {
    vec4 worldPos = ubo.model * vec4(inPosition, 1.0);

    fragPosWorld = worldPos.xyz;
    normalWorld = mat3(transpose(inverse(ubo.model))) * inNormal;
    shadowCoord = ubo.lightViewProj * worldPos;
    fragTexCoord = inTexCoord;

    gl_Position = ubo.proj * ubo.view * worldPos;
}
