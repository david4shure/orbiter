use crate::lines;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use ndarray::{arr1, arr2};
use std::str;

pub struct OrbitPlugin;

impl Plugin for OrbitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, moon_orbit)
            .add_systems(Update, rotate_moon)
            .add_systems(Update, rotate_earth)
            .add_systems(Update, draw_lunar_orbit_lines)
            //.add_systems(Update, draw_lunar_orbit_lines)
            .insert_resource(LunarOrbit {
                ..Default::default()
            })
            .insert_resource(lines::LineStrip {
                ..Default::default()
            })
            .insert_resource(TimeScale {
                ..Default::default()
            })
            .add_plugins(MaterialPlugin::<lines::LineMaterial>::default())
            .register_type::<TimeScale>()
            .register_type::<LunarOrbit>()
            .register_type::<OrbitalParameters>();
    }
}

// Consts
const G: f64 = 6.67e-20; // In KM!
use std::f32::consts::PI;
const PI64: f64 = PI as f64;
pub const REAL_TO_WORLD: f32 = 500. / 12742.; // 100 in world unit to 12,742 KM (Earth width)
pub const WORLD_TO_REAL: f32 = 12742. / 500.; // 12742 KM to 100 world units

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct TimeScale {
    pub scale: f64,
}

impl Default for TimeScale {
    fn default() -> Self {
        return TimeScale { scale: 3600. };
    }
}

#[derive(Reflect, Resource, InspectorOptions, Component)]
#[reflect(Resource, InspectorOptions)]
pub struct LunarOrbit {
    pub orbit: OrbitalParameters,
}

impl Default for LunarOrbit {
    fn default() -> Self {
        return LunarOrbit {
            orbit: OrbitalParameters::new(
                0.3844e6, 0.0549, 0.08970992, 2.6052996, 3.17633867, 5.9722e+24, 2360592.,
            ),
        };
    }
}

// Components
// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component, Reflect)]
pub struct CelestialBody {
    pub focus_idx: i32,
    pub name: String,
    pub viewport_position: Option<Vec2>,
}

#[derive(Component)]
pub struct EarthBody;

#[derive(Component)]
pub struct MoonBody;

#[derive(Reflect, Resource, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct OrbitalParameters {
    semimajor_axis: f64,     // KM
    longitude_asc_node: f64, // Radians
    arg_of_periapsis: f64,   // Radians
    inclination: f64,        // Radians
    eccentricity: f64,       // Unitless
    mass_of_parent: f64,     // KG
    grav_parameter: f64,     // KM^3s^-2
    period: f64,             // Seconds
    rotational_period: f64,  // Seconds
}

impl Default for OrbitalParameters {
    fn default() -> Self {
        OrbitalParameters {
            semimajor_axis: 0.,
            longitude_asc_node: 0.,
            arg_of_periapsis: 0.,
            inclination: 0.,
            eccentricity: 0.,
            mass_of_parent: 0.,
            grav_parameter: 0.,
            period: 0.,
            rotational_period: 0.,
        }
    }
}

impl OrbitalParameters {
    pub fn new(
        semimajor_axis: f64,
        eccentricity: f64,
        inclination: f64,
        arg_of_periapsis: f64,
        longitude_asc_node: f64,
        mass_of_parent: f64,
        rotational_period: f64,
    ) -> OrbitalParameters {
        let mu = G * mass_of_parent;
        OrbitalParameters {
            semimajor_axis,
            eccentricity,
            inclination,
            arg_of_periapsis,
            longitude_asc_node,
            mass_of_parent,
            grav_parameter: mu,
            period: 2. * PI64 * (semimajor_axis.powf(3.) / mu).sqrt(),
            rotational_period,
        }
    }

    pub fn position(mut self, t: f64) -> Vec3 {
        self.period = 2. * PI64 * (self.semimajor_axis.powf(3.) / self.grav_parameter).sqrt();

        let mean_anomaly = self.mean_anomaly(t % self.period, self.period);
        let eccentric_anomaly = self.eccentric_anomaly(mean_anomaly);
        let true_anomaly = self.true_anomaly(eccentric_anomaly);
        let distance = self.distance(eccentric_anomaly);

        let x = distance * true_anomaly.cos();
        let y = distance * true_anomaly.sin();
        let z = 0.;

        let coords = arr1(&[x, y, z]);

        let w_trans = arr2(&[
            [
                self.arg_of_periapsis.cos(),
                -self.arg_of_periapsis.sin(),
                0.,
            ],
            [self.arg_of_periapsis.sin(), self.arg_of_periapsis.cos(), 0.],
            [0., 0., 1.],
        ]);

        let i_trans = arr2(&[
            [1., 0., 0.],
            [0., self.inclination.cos(), -self.inclination.sin()],
            [0., self.inclination.sin(), self.inclination.cos()],
        ]);

        let omega_trans = arr2(&[
            [
                self.longitude_asc_node.cos(),
                -self.longitude_asc_node.sin(),
                0.,
            ],
            [
                self.longitude_asc_node.sin(),
                self.longitude_asc_node.cos(),
                0.,
            ],
            [0., 0., 1.],
        ]);

        let final_coords = (omega_trans.dot(&w_trans.dot(&i_trans))).dot(&coords);

        return Vec3::new(
            final_coords[0] as f32,
            final_coords[1] as f32,
            final_coords[2] as f32,
        );
    }

    pub fn mean_anomaly(&self, t: f64, period: f64) -> f64 {
        (t / period) * 2. * PI64
    }

    pub fn eccentric_anomaly(&self, mean_anomaly: f64) -> f64 {
        let eta = 1e-15_f64;
        let e_naught;

        if (-1. * PI64 < mean_anomaly && mean_anomaly < 0.) || (mean_anomaly > PI64) {
            e_naught = mean_anomaly - self.eccentricity;
        } else {
            e_naught = mean_anomaly + self.eccentricity;
        }

        let mut e_n = e_naught;
        let mut delta = eta + 1.;
        let mut count = 0;
        let mut e_np1 = 0.;

        while delta > eta {
            e_np1 = e_n
                + (mean_anomaly - e_n + self.eccentricity * e_n.sin())
                    / (1. - self.eccentricity * e_n.cos());
            delta = (e_np1 - e_n).abs();
            e_n = e_np1;
            count += 1;

            if count > 20 {
                println!("Something bad happened, couldn't converge for eccentric anomaly.");
                return 0. as f64;
            }
        }

        e_np1
    }

    pub fn true_anomaly(&self, eccentric_anomaly: f64) -> f64 {
        2. * (((1. + self.eccentricity) / (1. - self.eccentricity)).sqrt()
            * (eccentric_anomaly / 2.).tan())
        .atan()
    }

    pub fn distance(&self, eccentric_anomaly: f64) -> f64 {
        self.semimajor_axis * (1. - self.eccentricity * eccentric_anomaly.cos())
    }

    pub fn compute_orbit_lines(&self, num_lines: i32) -> Vec<Vec3> {
        let mut lines: Vec<Vec3> = Vec::<Vec3>::new();
        let period = 2. * PI64 * (self.semimajor_axis.powf(3.) / self.grav_parameter).sqrt();

        let time_increment = period / (num_lines as f64);

        let mut t: f64 = 0.;

        while t <= period {
            let vec = REAL_TO_WORLD * self.position(t);
            lines.push(Vec3::new(-vec.x, -vec.z, vec.y));
            t = t + time_increment;
        }

        lines.push(lines[lines.len() - 1 as usize]);
        lines.push(lines[0 as usize]);

        lines
    }
}

pub fn rotate_earth(mut query: Query<&mut Transform, With<EarthBody>>, time: Res<Time>, time_scale: Res<TimeScale>) {
    for mut transform in &mut query {
        let val = ((time.delta_seconds() * time_scale.scale as f32)
                / 86400 as f32)
                * 2.
                * PI;
        transform.rotate_y(val);
    }
}

pub fn rotate_moon(
    mut query: Query<&mut Transform, With<MoonBody>>,
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    lunar_orbit: ResMut<LunarOrbit>,
) {
    for mut transform in &mut query {
        transform.rotate_z(
            ((time.delta_seconds() * time_scale.scale as f32)
                / lunar_orbit.orbit.rotational_period as f32)
                * 2.
                * PI,
        );
    }
}

pub fn moon_orbit(
    mut body_query: Query<&mut Transform, With<MoonBody>>,
    orbit: Res<LunarOrbit>,
    time: Res<Time>,
    time_scale: Res<TimeScale>,
) {
    let mut posn = orbit
        .orbit
        .position(time.elapsed_seconds_f64() * time_scale.scale);

    // Convert from physical coordinates to world/scene coordinates
    posn.x *= REAL_TO_WORLD;
    posn.y *= REAL_TO_WORLD;
    posn.z *= REAL_TO_WORLD;

    for mut transform in &mut body_query {
        transform.translation = Vec3::new(-posn.x, -posn.z, posn.y);
    }
}

pub fn draw_lunar_orbit_lines(
    orbit: Res<LunarOrbit>,
    mut commands: Commands,
    mut mesh_query: Query<Entity, With<lines::OrbitalLines>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Only despawn original lines if the orbit has changed.
    if orbit.is_changed() {
        let orbit_lines = orbit.orbit.compute_orbit_lines(1000);

        for entity in &mut mesh_query {
            commands.entity(entity).despawn_recursive();
        }

        // Draw moon lines
        // Spawn a line strip that goes from point to point
        commands.spawn((
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(lines::LineStrip {
                    points: orbit_lines,
                })),
                transform: Transform::from_xyz(0.5, 0.0, 0.0),
                material: materials.add(StandardMaterial{
                    base_color: Color::rgba(1., 0.0, 0.0, 1.),
                    emissive: Color::rgba(1., 0.,0.,1.),
                    ..default()
                }),
                ..default()
            },
            lines::OrbitalLines,
        ));
        println!("Orbit changed.");
    }
}
