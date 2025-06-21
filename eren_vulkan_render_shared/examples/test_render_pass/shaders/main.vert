#version 450

layout(set = 0, binding = 0) uniform ModelView {
  mat4 modelView;
};

layout(set = 0, binding = 1) uniform Projection {
  mat4 projection;
};

layout(set = 0, binding = 2) uniform NormalMatrix {
  mat4 normalMatrix;
};

layout(set = 0, binding = 3) uniform LightDirection {
  vec3 lightDirection;
};

layout(set = 0, binding = 4) uniform ViewDirection {
  vec3 viewDirection;
};

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inNormal;

layout(location = 0) out vec3 vViewDir;
layout(location = 1) out vec3 vNormal;
layout(location = 2) out vec3 vLightDir;
layout(location = 3) out vec3 vWldLoc;
layout(location = 4) out vec3 vLightLoc;
layout(location = 5) out vec3 vInPos;

void main() {
  vViewDir = normalize((normalMatrix * vec4(-viewDirection, 0.0)).xyz);
  vLightDir = normalize((normalMatrix * vec4(-lightDirection, 0.0)).xyz);
  vNormal = normalize((normalMatrix * vec4(inNormal, 0.0)).xyz);

  vec4 wldLoc = modelView * vec4(inPos, 1.0);
  gl_Position = projection * wldLoc;

  vWldLoc = wldLoc.xyz / wldLoc.w;
  vInPos = inPos;

  vec4 lightLoc = modelView * vec4(lightDirection, 1.0);
  vLightLoc = lightLoc.xyz / lightLoc.w;
}
