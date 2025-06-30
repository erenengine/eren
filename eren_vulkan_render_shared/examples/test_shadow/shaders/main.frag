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

    // Z좌표(깊이)는 이미 [0, 1] 범위이므로 변환하지 않습니다.
    // X, Y 좌표만 [-1, 1] -> [0, 1] 범위로 변환합니다.
    vec2 shadowTexCoord = projCoords.xy * 0.5 + 0.5;
    float currentDepth = projCoords.z;

    // 그림자 맵 범위 밖이면 그림자 없음
    if (currentDepth > 1.0 || shadowTexCoord.x < 0.0 || shadowTexCoord.x > 1.0 || shadowTexCoord.y < 0.0 || shadowTexCoord.y > 1.0) {
        return 0.0;
    }

    float closestDepth = texture(shadowMap, shadowTexCoord).r;

    // self-shadowing 방지를 위한 bias
    float bias = max(0.005 * (1.0 - dot(normalize(normalWorld), normalize(-light.direction))), 0.0005);
    float shadow = (currentDepth - bias > closestDepth) ? 1.0 : 0.0;

    return shadow;
}

void main() {
    vec3 norm = normalize(normalWorld);
    vec3 lightDir = normalize(-light.direction);

    // Ambient 조명 추가 (완전한 검은색 방지)
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * light.color;

    // Diffuse 조명
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.color * diff;
    
    float shadow = calculateShadow(shadowCoord);

    vec3 baseColor;
    if (fragPosWorld.y == -1.0) { // y=-1이면 땅
        baseColor = vec3(0.8, 0.8, 0.8); // 밝은 회색 땅
    } else { // 그 외는 큐브
        baseColor = vec3(1.0, 0.7, 0.2); // 오렌지색 큐브
    }
    
    vec3 lighting = ambient + (1.0 - shadow) * diffuse;

    outColor = vec4(baseColor * lighting, 1.0);
}
