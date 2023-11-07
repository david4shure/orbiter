// Constants
const PI: f32 = 3.1415926535897932;
const POINTS_FROM_CAMERA: u32 = 10u;
const OPTICAL_DEPTH_POINTS: u32 = 10u;

// Input and output types
struct VertexOutput {
    [[location(0)]] vUV: vec2<f32>;
};

struct FragmentInput {
    [[location(0)]] vUV: vec2<f32>;
};

[[group(0), binding(0)]] var<filtering> textureSampler: texture_2d<f32>;
[[group(0), binding(1)]] var<filtering> depthSampler: texture_2d<f32>;

// AtmosphereSettings
[[block]] struct AtmosphereSettings {
    sunPosition: vec3<f32>;
    cameraPosition: vec3<f32>;
    inverseProjection: mat4x4<f32>;
    inverseView: mat4x4<f32>;
    cameraNear: f32;
    cameraFar: f32;
    planetPosition: vec3<f32>;
    planetRadius: f32;
    atmosphereRadius: f32;
    falloffFactor: f32;
    sunIntensity: f32;
    scatteringStrength: f32;
    densityModifier: f32;
    redWaveLength: f32;
    greenWaveLength: f32;
    blueWaveLength: f32;
};

[[group(0), binding(2)]] var<uniform> uniforms: AtmosphereSettings;

// Functions
fn remap(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
    return low2 + (value - low1) * (high2 - low2) / (high1 - low1);
}

fn rayIntersectSphere(rayOrigin: vec3<f32>, rayDir: vec3<f32>, spherePosition: vec3<f32>, sphereRadius: f32) -> bool {
    let relativeOrigin: vec3<f32> = rayOrigin - spherePosition;
    let a: f32 = 1.0;
    let b: f32 = 2.0 * dot(relativeOrigin, rayDir);
    let c: f32 = dot(relativeOrigin, relativeOrigin) - sphereRadius * sphereRadius;
    let discriminant: f32 = b * b - 4.0 * a * c;
    if (discriminant < 0.0) {
        return false;
    }
    let sqrtDiscriminant: f32 = sqrt(discriminant);
    let t0: f32 = (-b - sqrtDiscriminant) / (2.0 * a);
    let t1: f32 = (-b + sqrtDiscriminant) / (2.0 * a);
    return t1 >= 0.0;
}

fn densityAtPoint(densitySamplePoint: vec3<f32>) -> f32 {
    let heightAboveSurface: f32 = length(densitySamplePoint - uniforms.planetPosition) - uniforms.planetRadius;
    let height01: f32 = heightAboveSurface / (uniforms.atmosphereRadius - uniforms.planetRadius);
    var localDensity: f32 = uniforms.densityModifier * exp(-height01 * uniforms.falloffFactor);
    localDensity *= (1.0 - height01);
    return localDensity;
}

fn opticalDepth(rayOrigin: vec3<f32>, rayDir: vec3<f32>, rayLength: f32) -> f32 {
    let stepSize: f32 = rayLength / f32(OPTICAL_DEPTH_POINTS - 1u);
    var densitySamplePoint: vec3<f32> = rayOrigin;
    var accumulatedOpticalDepth: f32 = 0.0;
    for (var i: u32 = 0u; i < OPTICAL_DEPTH_POINTS; i = i + 1u) {
        let localDensity: f32 = densityAtPoint(densitySamplePoint);
        accumulatedOpticalDepth = accumulatedOpticalDepth + localDensity * stepSize;
        densitySamplePoint = densitySamplePoint + rayDir * stepSize;
    }
    return accumulatedOpticalDepth;
}

fn calculateLight(rayOrigin: vec3<f32>, rayDir: vec3<f32>, rayLength: f32) -> vec3<f32> {
    var samplePoint: vec3<f32> = rayOrigin;
    let sunDir: vec3<f32> = normalize(uniforms.sunPosition - uniforms.planetPosition);
    let wavelength: vec3<f32> = vec3<f32>(uniforms.redWaveLength, uniforms.greenWaveLength, uniforms.blueWaveLength);
    var scatteringCoeffs: vec3<f32> = pow(1063.0 / wavelength, vec3<f32>(4.0)) * uniforms.scatteringStrength;
    scatteringCoeffs = scatteringCoeffs / uniforms.planetRadius;
    let stepSize: f32 = rayLength / f32(POINTS_FROM_CAMERA - 1u);
    var inScatteredLight: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < POINTS_FROM_CAMERA; i = i + 1u) {
        let sunRayLengthInAtm: f32 = uniforms.atmosphereRadius - length(samplePoint - uniforms.planetPosition);
        var t0: f32;
        var t1: f32;
        if (rayIntersectSphere(samplePoint, sunDir, uniforms.planetPosition, uniforms.atmosphereRadius)) {
            t0 = 0.0;
            t1 = t0;
        } else {
            t0 = -1.0;
            t1 = 1.0;
        }
        let sunRayOpticalDepth: f32 = opticalDepth(samplePoint, sunDir, t1);
        let viewRayLengthInAtm: f32 = stepSize * f32(i);
        let viewRayOpticalDepth: f32 = opticalDepth(samplePoint, -rayDir, viewRayLengthInAtm);
        let transmittance: vec3<f32> = exp(-(sunRayOpticalDepth + viewRayOpticalDepth) * scatteringCoeffs);
        let localDensity: f32 = densityAtPoint(samplePoint);
        inScatteredLight = inScatteredLight + localDensity * transmittance * scatteringCoeffs * stepSize;
        samplePoint = samplePoint + rayDir * stepSize;
    }
    let costheta: f32 = dot(rayDir, sunDir);
    let phaseRayleigh: f32 = 3.0 / (16.0 * PI) * (1.0 + costheta * costheta);
    inScatteredLight = inScatteredLight * phaseRayleigh * uniforms.sunIntensity;
    return inScatteredLight;
}

fn scatter(originalColor: vec3<f32>, rayOrigin: vec3<f32>, rayDir: vec3<f32>, maximumDistance: f32) -> vec3<f32> {
    var impactPoint: f32;
    var escapePoint: f32;
    if (rayIntersectSphere(rayOrigin, rayDir, uniforms.planetPosition, uniforms.atmosphereRadius)) {
        impactPoint = max(0.0, 0.0);
        escapePoint = min(maximumDistance, 0.0);
    } else {
        return originalColor;
    }
    var distanceThroughAtmosphere: f32 = max(0.0, escapePoint - impactPoint);
    var firstPointInAtmosphere: vec3<f32> = rayOrigin + rayDir * impactPoint;
    let light: vec3<f32> = calculateLight(firstPointInAtmosphere, rayDir, distanceThroughAtmosphere);
    return originalColor * (1.0 - light) + light;
}

[[stage(fragment)]]
fn main(input: FragmentInput) -> [[location(0)]] vec4<f32> {
    var screenColor: vec3<f32> = textureSample(textureSampler, input.vUV).rgb;
    var depth: f32 = textureSample(depthSampler, input.vUV).r;
    var deepestPoint: vec3<f32> = worldFromUV(input.vUV, depth) - uniforms.cameraPosition;
    var maximumDistance: f32 = length(deepestPoint);
    var rayDir: vec3<f32> = deepestPoint / maximumDistance;
    var t0: f32;
    var t1: f32;
    if (rayIntersectSphere(uniforms.cameraPosition, rayDir, uniforms.planetPosition, uniforms.planetRadius)) {
        if (maximumDistance > t0 - 1.0) {
            maximumDistance = t0;
        }
    }
    var finalColor: vec3<f32> = scatter(screenColor, uniforms.cameraPosition, rayDir, maximumDistance);
    return vec4<f32>(finalColor, 1.0);
}