use atmosphere::AtmosphereSettings;
use bevy::prelude::*;

use bevy::pbr::{CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver};
use bevy::render::camera::CameraProjection;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use orbit::OrbitPlugin;
use sphere_camera::SphericalCameraPlugin;
use topocentric_camera::TopoCentricCameraPlugin;
use bevy::{
    core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass},
    pbr::{PbrPlugin},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

mod lines;
mod orbit;
mod sphere_camera;
mod topocentric_camera;
mod atmosphere;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins(TopoCentricCameraPlugin)
        .add_plugins((WorldInspectorPlugin::new(), SphericalCameraPlugin))
        .add_plugins(OrbitPlugin)
        .add_plugins(atmosphere::PostProcessPlugin)
        .insert_resource(Msaa::Off)
        .add_systems(Update, sync_data_to_atmosphere_settings)
        .run();
}

fn setup(mut commands: Commands, ass: Res<AssetServer>) {
    let earth_handle = ass.load("earth.glb#Scene0");
    let skybox_handle = ass.load("sky_actual_constel.glb#Scene0");
    let moon_handle = ass.load("moon.glb#Scene0");

    // Earth
    commands.spawn((
        SceneBundle {
            scene: earth_handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        orbit::EarthBody,
    ));

    // Sphere Camera
    commands.spawn(sphere_camera::SphereCamera {
        radius: 600.,
        ..Default::default()
    });

    // Altitude Azimuth Camera Controls
    commands.spawn(topocentric_camera::AltitudeAzimuthCamera {
        altitude: 0.,
        azimuth: 0.,
        roll: 0.,
    });

    // 3D Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::Y, Vec3::Y),
            ..default()
        },
        NotShadowCaster,
        // Add the setting to the camera.
        // This component is also used to determine on which camera to run the post processing effect.
        atmosphere::AtmosphereSettings {
            planetPosition: Vec3::new(0.,0.,0.),
            planetRadius: 300.,
            atmosphereRadius: 340.,
            falloffFactor: 0.8,
            sunIntensity: 100000.,
            scatteringStrength: 1.,
            densityModifier: 0.8,
            redWaveLength: 700.,
            greenWaveLength: 450.,
            blueWaveLength: 440.,
            sunPosition: Vec3::new(5000.0, 4.0, 0.0),
            cameraPosition: Vec3::new(0.,0.,0.),
            inverseProjection: Mat4::IDENTITY,
            inverseView: Mat4::IDENTITY,
            cameraFar: 0.,
            cameraNear: 0.,
        },
        // To enable the prepass you need to add the components associated with the ones you need
        // This will write the depth buffer to a texture that you can use in the main pass
        DepthPrepass,
        // This will generate a texture containing world normals (with normal maps applied)
        NormalPrepass,
        // This will generate a texture containing screen space pixel motion vectors
        MotionVectorPrepass,
    ));

    //commands.insert_resource(AmbientLight {
    //    color: Color::WHITE,
    //    brightness: 0.1,
    //});

    // Sun light
    //commands
    //    .spawn(DirectionalLightBundle {
    //        directional_light: DirectionalLight {
    //            color: Color::WHITE,
    //            illuminance: 100000.,
    //            shadows_enabled: true,
    //            ..default()
    //        },
    //        transform: Transform::from_xyz(1000.0, -100.0, 0.0)
    //            .with_rotation(Quat::from_rotation_x(23.4 * std::f32::consts::PI / 180.)),
    //        ..default()
    //    })
    //    .insert(Name::new("Sun Light"));

    commands.spawn(PointLightBundle {
        // transform: Transform::from_xyz(5.0, 8.0, 2.0),
        transform: Transform::from_xyz(5000.0, 4.0, 0.0),
        point_light: PointLight {
            range: 100000.,
            intensity: 10000000000., // lumens - roughly a 100W non-halogen incandescent bulb
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Skybox
    commands
        .spawn((
            SceneBundle {
                scene: skybox_handle,
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            },
            NotShadowCaster,
        ))
        .insert(Name::new("Sky"));

    // Moon
    commands
        .spawn((
            SceneBundle {
                scene: moon_handle.clone(),
                transform: Transform::from_xyz(0.0, orbit::REAL_TO_WORLD * 384472.282, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.)),
                ..Default::default()
            },
            orbit::CelestialBody {
                name: "Moon".to_string(),
                focus_idx: 1,
                viewport_position: None,
            },
            orbit::MoonBody,
        ))
        .insert(Name::new("Moon"));
}

//fn rotate_earth(mut query: Query<&mut Transform, With<orbit::EarthBody>>, time: Res<Time>) {
//    for mut transform in query.iter_mut() {
//        transform.rotate_y(time.delta_seconds() * 0.4);
//    }
//}


pub fn sync_data_to_atmosphere_settings(
    mut camera_q: Query<&mut GlobalTransform, With<Camera3d>>,
    mut projection_q: Query<&mut PerspectiveProjection, With<Camera3d>>,
    mut atmosphere_q: Query<&mut AtmosphereSettings>,
) {
    let mut atmosphere = match atmosphere_q.get_single_mut() {
        Ok(atmosphere) => atmosphere,
        Err(_) => return,
    };

    let mut camera = match camera_q.get_single_mut() {
        Ok(camera) => camera,
        Err(_) => return,
    };

    let mut projection = match projection_q.get_single_mut() {
        Ok(projection) => projection,
        Err(_) => return,
    };

    atmosphere.cameraFar = projection.far;
    atmosphere.cameraNear = projection.near;
    atmosphere.inverseProjection = projection.get_projection_matrix().inverse();
    atmosphere.inverseView = camera.compute_matrix().inverse();
    atmosphere.cameraPosition = camera.translation();
}