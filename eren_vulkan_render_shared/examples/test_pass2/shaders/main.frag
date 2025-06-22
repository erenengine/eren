#version 450

layout(set = 0, binding = 5) uniform AmbientColor {
  vec4 ambientColor;
};

layout(set = 0, binding = 6) uniform DiffuseColor {
  vec4 diffuseColor;
};

layout(set = 0, binding = 7) uniform SpecularColor {
  vec4 specularColor;
};

layout(set = 0, binding = 8) uniform Shininess {
  float shininess;
};

layout(set = 0, binding = 9) uniform sampler2DShadow shadowMap;
layout(set = 0, binding = 11) uniform LightModelView {
  mat4 lightModelViewMatrix;
};

layout(set = 0, binding = 12) uniform LightProjection {
  mat4 lightProjectionMatrix;
};

layout(location = 0) in vec3 vViewDir;
layout(location = 1) in vec3 vNormal;
layout(location = 2) in vec3 vLightDir;
layout(location = 3) in vec3 vWldLoc;
layout(location = 4) in vec3 vLightLoc;
layout(location = 5) in vec3 vInPos;

layout(location = 0) out vec4 outColor;

float diffuse(vec3 lightDir, vec3 normal, vec3 color) {
  return max(dot(lightDir, normal), 0.0) * color;
}

vec3 specular(vec3 lightDir, vec3 viewDir, vec3 normal, vec3 color, float shininess) {
  vec3 reflectDir = reflect(-lightDir, normal);
  float spec = max(dot(reflectDir, viewDir), 0.0);
  return pow(spec, shininess) * color;
}

void main() {
  vec3 lightDir = normalize(vLightDir);
  vec3 normal = normalize(vNormal);
  vec3 viewDir = normalize(vViewDir);

  vec4 fragPosLightSpace = lightProjectionMatrix * lightModelViewMatrix * vec4(vInPos, 1.0);
  fragPosLightSpace /= fragPosLightSpace.w;
  vec2 uv = fragPosLightSpace.xy * 0.5 + 0.5;
  float depth = fragPosLightSpace.z;

  float visibility = 0.0;
  float offset = 1.0 / 1024.0;
  for (int y = -2; y <= 2; ++y) {
    for (int x = -2; x <= 2; ++x) {
      vec2 offs = vec2(x, y) * offset;
      visibility += texture(shadowMap, vec3(uv + offs, depth - 0.0003));
    }
  }
  visibility /= 25.0;

  vec3 wldLoc2light = vWldLoc - vLightLoc;
  float align = dot(normalize(wldLoc2light), lightDir);

  if (gl_FrontFacing && align > 0.9) {
    vec3 radiance = ambientColor.rgb + 
                    diffuse(-lightDir, normal, diffuseColor.rgb) +
                    specular(-lightDir, viewDir, normal, specularColor.rgb, shininess);
    outColor = vec4(radiance * visibility, 1.0);
  } else {
    outColor = vec4(0.0, 0.0, 0.0, 1.0);
  }
}
