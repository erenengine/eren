#version 450

layout(location = 0) in vec3 fragPosWorld;
layout(location = 1) in vec3 normalWorld;
layout(location = 2) in vec4 shadowCoord;

layout(set = 1, binding = 0) uniform sampler2D shadowMap;

layout(set = 0, binding = 1) uniform Light {
    vec3 direction;  // 태양광 방향 (단위벡터, ex. normalize(vec3(-1, -1, -0.5)))
    vec3 color;      // 빛 색상 (ex. vec3(1.0, 0.95, 0.9))
} light;

layout(location = 0) out vec4 outColor;

float calculateShadow(vec4 shadowCoord) {
    vec3 projCoords = shadowCoord.xyz / shadowCoord.w;
    projCoords = projCoords * 0.5 + 0.5;

    // 그림자 맵 범위 밖이면 그림자 없음
    if (projCoords.z > 1.0 || projCoords.x < 0.0 || projCoords.x > 1.0 || projCoords.y < 0.0 || projCoords.y > 1.0) {
        return 0.0;
    }

    float closestDepth = texture(shadowMap, projCoords.xy).r;
    float currentDepth = projCoords.z;

    float bias = 0.005;
    float shadow = (currentDepth - bias > closestDepth) ? 1.0 : 0.0;

    return shadow;
}

void main() {
    vec3 norm = normalize(normalWorld);
    vec3 lightDir = normalize(-light.direction); // 광원 방향은 음의 방향

    float diff = max(dot(norm, lightDir), 0.0);
    float shadow = calculateShadow(shadowCoord);

    vec3 baseColor = vec3(1.0, 0.6, 0.4); // 정육면체 기본 색
    vec3 lighting = (1.0 - shadow) * light.color * diff;

    outColor = vec4(baseColor * lighting, 1.0);
}
