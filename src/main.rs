use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use sphere_camera::SphericalCameraPlugin;
use topocentric_camera::TopoCentricCameraPlugin;

mod sphere_camera;
mod topocentric_camera;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_earth)
        .add_plugins(TopoCentricCameraPlugin)
        .add_plugins((WorldInspectorPlugin::new(),SphericalCameraPlugin))
        .run();
}

fn setup(
    mut commands: Commands, 
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    //let earth_handle = ass.load("earth_hd_rotated.glb#Scene0");
    let skybox_handle = ass.load("skybox1.glb#Scene0");

    // add earth
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere{radius:50.,stacks:200,sectors:200})),
        material: standard_materials.add(StandardMaterial{
            base_color_texture: Some(ass.load("textures/8k_earth_daymap.png")),
            emissive_texture: Some(ass.load("textures/8k_earth_nightmap.png")),
            normal_map_texture: Some(ass.load("textures/8k_earth_normal_map.png")),
            metallic_roughness_texture: Some(ass.load("textures/8k_earth_specular_map_inverted.png")),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            reflectance: 0.1,
            emissive: Color::Rgba { red: 0.15, green: 0.15, blue: 0.15, alpha: 1.0 },
            double_sided: false,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    },sphere_camera::EarthBody));

    // Skybox
    commands.spawn(
sphere_camera::SphereCamera {
            radius: 350.,
            ..Default::default()
        },
    );

    commands.spawn(
        Camera3dBundle {
            transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        }
    );

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 100000.,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(1000.0, -100.0, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI/2.)),
        ..default()
    }).insert(Name::new("Sun Light"));

    commands
    .spawn(SceneBundle {
        scene: skybox_handle,
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    })
    .insert(Name::new("Sky"));

}

fn rotate_earth(mut query: Query<&mut Transform, With<sphere_camera::EarthBody>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.rotate_z(time.delta_seconds() * 0.4);
    }
}
