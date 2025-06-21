#version 450

layout(set = 0, binding = 0) uniform ModelView {
  mat4 modelView;
};

layout(set = 0, binding = 1) uniform Projection {
  mat4 projection;
};

layout(location = 0) in vec3 inPos;

void main() {
  vec4 worldPosition = modelView * vec4(inPos, 1.0);
  gl_Position = projection * worldPosition;
}
