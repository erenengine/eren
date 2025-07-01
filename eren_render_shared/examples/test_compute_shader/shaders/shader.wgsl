const EYE_POS: vec3<f32> = vec3<f32>(2.0, 2.0, 2.0);
const CENTER_POS: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const UP_VEC: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0); // Z-up

struct Settings {
    time: f32,
    aspect_ratio: f32,
    _padding: vec2<f32>, // Alignment for 16-byte layout
}

@group(0) @binding(0)
var<uniform> settings: Settings;

// 결과를 저장할 StorageBufferObject
struct StorageBufferObject {
    model : mat4x4<f32>,
    view  : mat4x4<f32>,
    proj  : mat4x4<f32>,
};

@group(0) @binding(1)
var<storage, read_write> ssbo : StorageBufferObject;

fn rotationZ(rad: f32) -> mat4x4<f32> {
    let s = sin(rad);
    let c = cos(rad);
    return mat4x4<f32>(
        vec4<f32>( c,  s, 0.0, 0.0),
        vec4<f32>(-s,  c, 0.0, 0.0),
        vec4<f32>( 0.0, 0.0, 1.0, 0.0),
        vec4<f32>( 0.0, 0.0, 0.0, 1.0)
    );
}

fn lookAt(eye: vec3<f32>, center: vec3<f32>, up: vec3<f32>) -> mat4x4<f32> {
    let f = normalize(center - eye);
    let s = normalize(cross(f, up));
    let u = cross(s, f);

    return mat4x4<f32>(
        vec4<f32>(s.x, u.x, -f.x, 0.0),
        vec4<f32>(s.y, u.y, -f.y, 0.0),
        vec4<f32>(s.z, u.z, -f.z, 0.0),
        vec4<f32>(-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0)
    );
}

fn perspectiveRH(fovy: f32, aspect: f32, znear: f32, zfar: f32) -> mat4x4<f32> {
    let t = 1.0 / tan(fovy * 0.5);
    return mat4x4<f32>(
        vec4<f32>(t / aspect, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, t, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, zfar / (znear - zfar), -1.0),
        vec4<f32>(0.0, 0.0, (znear * zfar) / (znear - zfar), 0.0)
    );
}

@compute @workgroup_size(1)
fn compute_mvp() {
    // Model matrix: rotate around Z axis over time
    let model = rotationZ(radians(90.0) * settings.time);

    // View matrix: look from eye to center
    let view = lookAt(EYE_POS, CENTER_POS, UP_VEC);

    // Projection matrix: perspective with Y-flip
    let proj = perspectiveRH(radians(45.0), settings.aspect_ratio, 0.1, 10.0);

    // Write to buffer
    ssbo.model = model;
    ssbo.view = view;
    ssbo.proj = proj;
}

// 정점 입력 구조체 (vertex buffer layout에서 제공)
struct VertexInput {
    @location(0) inPosition : vec2<f32>,
    @location(1) inColor    : vec3<f32>,
};

// 정점 출력 구조체
struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) fragColor : vec3<f32>,
};

@vertex
fn vs_main(input : VertexInput) -> VertexOutput {
    var output : VertexOutput;
    
    // GLSL과 동일한 방식의 모델-뷰-프로젝션 변환
    let pos = vec4<f32>(input.inPosition, 0.0, 1.0);
    output.position = ssbo.proj * ssbo.view * ssbo.model * pos;
    
    output.fragColor = input.inColor;
    return output;
}

@fragment
fn fs_main(@location(0) fragColor : vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(fragColor, 1.0);
}
