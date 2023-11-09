// This shader computes the chromatic aberration effect

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
//
// A fullscreen triangle is a single triangle that covers the entire screen.
// The box in the top left in that diagram is the screen. The 4 x are the corner of the screen
//
// Y axis
//  1 |  x-----x......
//  0 |  |  s  |  . ´
// -1 |  x_____x´
// -2 |  :  .´
// -3 |  :´
//    +---------------  X axis
//      -1  0  1  2  3
//
// As you can see, the triangle ends up bigger than the screen.
//
// You don't need to worry about this too much since bevy will compute the correct UVs for you.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_texture_sampler: sampler;

@group(0) @binding(2) var depth_texture: texture_depth_2d;
@group(0) @binding(3) var depth_texture_sampler: sampler;

struct AtmosphereSettings {
    sunPosition: vec3<f32>,
    cameraPosition: vec3<f32>,
    inverseProjection: mat4x4<f32>,
    inverseView: mat4x4<f32>,
    cameraNear: f32,
    cameraFar: f32,

    planetPosition: vec3<f32>,
    planetRadius: f32,
    atmosphereRadius: f32,

    falloffFactor: f32,
    sunIntensity: f32,
    scatteringStrength: f32,
    densityModifier: f32,

    redWaveLength: f32,
    greenWaveLength: f32,
    blueWaveLength: f32,
    #ifdef SIXTEEN_BYTE_ALIGNMENT
        // WebGL2 structs must be 16 byte aligned.
        _webgl2_padding: vec3<f32>
    #endif
}

@group(0) @binding(4) var<uniform> atmosphere_settings: AtmosphereSettings;

const PI: f32 = 3.1415926535897932;
const POINTS_FROM_CAMERA: u32 = 10u;
const OPTICAL_DEPTH_POINTS: u32 = 10u;

fn remap(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
    return low2 + (value - low1) * (high2 - low2) / (high1 - low1);
}

fn worldFromUV(UV: vec2<f32>, depth: f32, ats: AtmosphereSettings) -> vec3<f32> {
    var ndc: vec4<f32> = vec4<f32>((UV * 2.0 - 1.0), 0.0, 1.0);
    var posVS: vec4<f32> = ats.inverseProjection * ndc;
    var posVSNew = posVS * remap(depth, 0.0, 1.0, ats.cameraNear, ats.cameraFar);
    var posWS: vec4<f32> = ats.inverseView * vec4<f32>(posVSNew.xyz, 1.0);
    return posWS.xyz;
}

struct RaySphereIntersection {
    hit: bool,
    t0: f32,
    t1: f32,
};

fn rayIntersectSphere(rayOrigin: vec3<f32>, rayDir: vec3<f32>, spherePosition: vec3<f32>, sphereRadius: f32) -> RaySphereIntersection {
    var relativeOrigin: vec3<f32> = rayOrigin - spherePosition; // rayOrigin in sphere space

    var a: f32 = 1.0;
    var b: f32 = 2.0 * dot(relativeOrigin, rayDir);
    var c: f32 = dot(relativeOrigin, relativeOrigin) - sphereRadius * sphereRadius;

    var d: f32 = b * b - 4.0 * a * c;

    var intersectionInfo: RaySphereIntersection;
    if (d < 0.0) {
        intersectionInfo.hit = false; // no intersection
    } else {
        var r0: f32 = (-b - sqrt(d)) / (2.0 * a);
        var r1: f32 = (-b + sqrt(d)) / (2.0 * a);

        intersectionInfo.t0 = min(r0, r1);
        intersectionInfo.t1 = max(r0, r1);
        intersectionInfo.hit = intersectionInfo.t1 >= 0.0;
    }

    return intersectionInfo;
}

fn densityAtPoint(densitySamplePoint: vec3<f32>, ats: AtmosphereSettings) -> f32 {
    var heightAboveSurface: f32 = length(densitySamplePoint - ats.planetPosition) - ats.planetRadius; // actual height above surface
    var height01: f32 = heightAboveSurface / (ats.atmosphereRadius - ats.planetRadius); // normalized height between 0 and 1

    var localDensity: f32 = ats.densityModifier * exp(-height01 * ats.falloffFactor); // density with exponential falloff
    localDensity *= (1.0 - height01); // make it 0 at maximum height

    return localDensity;
}

fn opticalDepth(rayOrigin: vec3<f32>, rayDir: vec3<f32>, rayLength: f32, ats: AtmosphereSettings) -> f32 {
    var stepSize: f32 = rayLength / f32(OPTICAL_DEPTH_POINTS - 1u); // ray length between sample points

    var densitySamplePoint: vec3<f32> = rayOrigin; // that's where we start
    var accumulatedOpticalDepth: f32 = 0.0;

    for (var i: u32 = 0u; i < OPTICAL_DEPTH_POINTS; i = i + 1u) {
        var localDensity: f32 = densityAtPoint(densitySamplePoint, ats); // we get the density at the sample point

        accumulatedOpticalDepth = accumulatedOpticalDepth + localDensity * stepSize; // linear approximation : density is constant between sample points

        densitySamplePoint = densitySamplePoint + rayDir * stepSize; // we move the sample point
    }

    return accumulatedOpticalDepth;
}

fn calculateLight(rayOrigin: vec3<f32>, rayDir: vec3<f32>, rayLength: f32, ats: AtmosphereSettings) -> vec3<f32> {
    var samplePoint: vec3<f32> = rayOrigin; // first sampling point coming from camera ray

    var sunDir: vec3<f32> = normalize(ats.sunPosition - ats.planetPosition); // direction to the light source

    var wavelength: vec3<f32> = vec3<f32>(ats.redWaveLength, ats.greenWaveLength, ats.blueWaveLength); // the wavelength that will be scattered (rgb so we get everything)
    var scatteringCoeffs: vec3<f32> = pow(vec3<f32>(1063.0) / wavelength, vec3<f32>(4.0)) * ats.scatteringStrength; // the scattering is inversely proportional to the fourth power of the wave length;
    // about the 1063, it is just a constant that makes the scattering look good
    scatteringCoeffs = scatteringCoeffs / ats.planetRadius; // Scale invariance by Yincognyto https://github.com/BarthPaleologue/volumetric-atmospheric-scattering/issues/6#issuecomment-1432409930

    var stepSize: f32 = rayLength / f32(POINTS_FROM_CAMERA - 1u); // the ray length between sample points

    var inScatteredLight: vec3<f32> = vec3<f32>(0.0); // amount of light scattered for each channel

    for (var i: u32 = 0u; i < POINTS_FROM_CAMERA; i = i + 1u) {
        var sunRayLengthInAtm: f32 = ats.atmosphereRadius - length(samplePoint - ats.planetPosition); // distance traveled by light through atmosphere from light source

        var ret : RaySphereIntersection = rayIntersectSphere(samplePoint, sunDir, ats.planetPosition, ats.atmosphereRadius);

        if (ret.hit) {
            sunRayLengthInAtm = ret.t0;
        }

        var sunRayOpticalDepth: f32 = opticalDepth(samplePoint, sunDir, sunRayLengthInAtm, ats); // scattered from the sun to the point

        var viewRayLengthInAtm: f32 = stepSize * f32(i); // distance traveled by light through atmosphere from sample point to cameraPosition
        var viewRayOpticalDepth: f32 = opticalDepth(samplePoint, -rayDir, viewRayLengthInAtm, ats); // scattered from the point to the camera

        var transmittance: vec3<f32> = exp(-(sunRayOpticalDepth + viewRayOpticalDepth) * scatteringCoeffs); // exponential scattering with coefficients

        var localDensity: f32 = densityAtPoint(samplePoint, ats); // density at sample point

        inScatteredLight = inScatteredLight + localDensity * transmittance * scatteringCoeffs * stepSize; // add the resulting amount of light scattered toward the camera

        samplePoint = samplePoint + rayDir * stepSize; // move sample point along view ray
    }

    // scattering depends on the direction of the light ray and the view ray : it's the rayleigh phats. function
    // https://glossary.ametsoc.org/wiki/Rayleigh_phats._function
    var costheta: f32 = dot(rayDir, sunDir);
    var Rayleigh: f32 = 3.0 / (16.0 * PI) * (1.0 + costheta * costheta);

    inScatteredLight = inScatteredLight * Rayleigh; // apply rayleigh pahse
    inScatteredLight = inScatteredLight * ats.sunIntensity; // multiply by the intensity of the sun

    return inScatteredLight;
}

fn scatter(originalColor : vec3<f32>, rayOrigin : vec3<f32>, rayDir : vec3<f32>, maximumDistance : f32, ats: AtmosphereSettings) -> vec3<f32> {
    var impactPoint : f32;
    var escapePoint : f32;

    var ret : RaySphereIntersection = rayIntersectSphere(rayOrigin, rayDir, ats.planetPosition, ats.atmosphereRadius);
    
    if (!ret.hit) {
        return originalColor; // if not intersecting with atmosphere, return original color
    }

    impactPoint = ret.t0;
    escapePoint = ret.t1;

    impactPoint = max(0.0, impactPoint); // cannot be negative (the ray starts where the camera is in such a cats.)
    escapePoint = min(maximumDistance, escapePoint); // occlusion with other scene objects

    let distanceThroughAtmosphere : f32 = max(0.0, escapePoint - impactPoint); // probably doesn't need the max but for the sake of coherence the distance cannot be negative
    
    let firstPointInAtmosphere : vec3<f32> = rayOrigin + rayDir * impactPoint; // the first atmosphere point to be hit by the ray

    let light : vec3<f32> = calculateLight(firstPointInAtmosphere, rayDir, distanceThroughAtmosphere, ats); // calculate scattering
    
    return originalColor * (1.0 - light) + light; // blending scattered color with original color
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var screenColorSampled: vec3<f32> = textureSample(screen_texture, screen_texture_sampler, in.uv).rgb;
    var depthSampled: f32 = textureSample(depth_texture, depth_texture_sampler, in.uv);

    var deepestPoint: vec3<f32> = worldFromUV(in.uv, depthSampled, atmosphere_settings);
    var maximumDistance: f32 = length(deepestPoint);
    var rayDir: vec3<f32> = deepestPoint / maximumDistance;

    var ret : RaySphereIntersection = rayIntersectSphere(atmosphere_settings.cameraPosition, rayDir, atmosphere_settings.planetPosition, atmosphere_settings.planetRadius);

    if (ret.hit) {
        if (maximumDistance > ret.t0 - 1.0) {
            maximumDistance = ret.t0;
        }
    }

    var finalColor: vec3<f32> = scatter(screenColorSampled, atmosphere_settings.cameraPosition, rayDir, maximumDistance, atmosphere_settings);
    return vec4<f32>(finalColor, 1.0);
}
