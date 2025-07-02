struct MainUBO {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    lightViewProj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> ubo: MainUBO;

struct VertexInput {
    @location(0) inPosition: vec3<f32>,
    @location(1) inNormal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fragPosWorld: vec3<f32>,
    @location(1) normalWorld: vec3<f32>,
    @location(2) shadowCoord: vec4<f32>,
};

fn inverse_mat3(m: mat3x3<f32>) -> mat3x3<f32> {
    let a = m[0];
    let b = m[1];
    let c = m[2];

    let r0 = cross(b, c);
    let r1 = cross(c, a);
    let r2 = cross(a, b);

    let inv_det = 1.0 / dot(r2, c);

    return mat3x3<f32>(r0, r1, r2) * inv_det;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let worldPos = ubo.model * vec4<f32>(input.inPosition, 1.0);
    out.fragPosWorld = worldPos.xyz;

    let model3x3 = mat3x3<f32>(
        ubo.model[0].xyz,
        ubo.model[1].xyz,
        ubo.model[2].xyz
    );

    out.normalWorld = inverse_mat3(model3x3) * input.inNormal;
    out.shadowCoord = ubo.lightViewProj * worldPos;

    out.position = ubo.proj * ubo.view * worldPos;
    return out;
}

@group(1) @binding(0)
var shadowMap: texture_depth_2d;

@group(1) @binding(1)
var shadowSampler: sampler_comparison;

struct Light {
    direction: vec3<f32>,
    color: vec3<f32>,
};

@group(0) @binding(1)
var<uniform> light: Light;

struct FragmentInput {
    @location(0) fragPosWorld: vec3<f32>,
    @location(1) normalWorld: vec3<f32>,
    @location(2) shadowCoord: vec4<f32>,
};

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let norm = normalize(input.normalWorld);
    let lightDir = normalize(-light.direction);

    let ambientStrength = 0.1;
    let ambient = ambientStrength * light.color;

    let diff = max(dot(norm, lightDir), 0.0);
    let diffuse = light.color * diff;

    let projCoords = input.shadowCoord;
    let shadowTexCoord = vec2(
        projCoords.x * 0.5 + 0.5,
        projCoords.y * -0.5 + 0.5
    );
    let currentDepth = projCoords.z;

    var shadow = 0.0;

    if (currentDepth <= 1.0 &&
        shadowTexCoord.x >= 0.0 && shadowTexCoord.x <= 1.0 &&
        shadowTexCoord.y >= 0.0 && shadowTexCoord.y <= 1.0) {

        let texelSize = 1.0 / 2048.0;

        for (var x: i32 = -1; x <= 1; x = x + 1) {
            for (var y: i32 = -1; y <= 1; y = y + 1) {
                let offset = vec2<f32>(f32(x), f32(y)) * texelSize;
                shadow = shadow + f32(textureSampleCompare(
                    shadowMap,
                    shadowSampler,
                    shadowTexCoord + offset,
                    currentDepth
                ) < 1.0);
            }
        }

        shadow = shadow / 9.0;
    }

    var baseColor: vec3<f32>;
    if (input.fragPosWorld.y == -1.0) {
        baseColor = vec3<f32>(0.8, 0.8, 0.8);
    } else {
        baseColor = vec3<f32>(1.0, 0.7, 0.2);
    }

    let lighting = ambient + (1.0 - shadow) * diffuse;

    return vec4<f32>(baseColor * lighting, 1.0);
}
