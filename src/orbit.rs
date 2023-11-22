use crate::lines;
use crate::time::{PhysicsTime, PhysicsTimeMode};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
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

#[derive(Reflect, Resource, InspectorOptions, Component)]
#[reflect(Resource, InspectorOptions)]
pub struct LunarOrbit {
    pub orbit: OrbitalParameters,
}


// EC= 6.476694128611285E-02 QR= 3.565283199467715E+05 IN= 5.240010829674768E+00
// OM= 1.239837028145578E+02 W = 3.081359034620368E+02 Tp=  2451533.965359285008
// N = 1.546268358955514E-04 MA= 1.407402571142365E+02 TA= 1.451550311169052E+02
// A = 3.812186883524646E+05 AD= 4.059090567581577E+05 PR= 2.328185776517964E+06

// Symbol meaning:

// JDTDB    Julian Day Number, Barycentric Dynamical Time
//   EC     Eccentricity, e
//   QR     Periapsis distance, q (km)
//   IN     Inclination w.r.t X-Y plane, i (degrees)
//   OM     Longitude of Ascending Node, OMEGA, (degrees)
//   W      Argument of Perifocus, w (degrees)
//   Tp     Time of periapsis (Julian Day Number)
//   N      Mean motion, n (degrees/sec)
//   MA     Mean anomaly, M (degrees)
//   TA     True anomaly, nu (degrees)
//   A      Semi-major axis, a (km)
//   AD     Apoapsis distance (km)
//   PR     Sidereal orbit period (sec)

impl Default for LunarOrbit {
    fn default() -> Self {
        return LunarOrbit {
            orbit: OrbitalParameters::new(
                2.45638088,
                3.812186883524646E+05, 
                6.476694128611285E-02, 
                0.3361502582,
                5.37798606, 
                2.16392383, 
                5.9722e+24,
                2360584.6848,
                2360592.,
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
    pub semimajor_axis: f64,     // KM
    pub longitude_asc_node: f64, // Radians
    pub arg_of_periapsis: f64,   // Radians
    pub inclination: f64,        // Radians
    pub eccentricity: f64,       // Unitless
    pub mass_of_parent: f64,     // KG
    pub grav_parameter: f64,     // KM^3s^-2
    pub period: f64,             // Seconds
    pub rotational_period: f64,  // Seconds
    pub mean_anomaly_at_epoch: f64, // Angles
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
            mean_anomaly_at_epoch: 0.,
        }
    }
}

impl OrbitalParameters {
    pub fn new(
        mean_anomaly_at_epoch: f64,
        semimajor_axis: f64,
        eccentricity: f64,
        inclination: f64,
        arg_of_periapsis: f64,
        longitude_asc_node: f64,
        mass_of_parent: f64,
        period: f64,
        rotational_period: f64,
    ) -> OrbitalParameters {
        let mu = G * mass_of_parent;
        OrbitalParameters {
            mean_anomaly_at_epoch,
            semimajor_axis,
            eccentricity,
            inclination,
            arg_of_periapsis,
            longitude_asc_node,
            mass_of_parent,
            grav_parameter: mu,
            period: period,
            rotational_period,
        }
    }

    pub fn position(mut self, t: f64) -> Vec3 {
        self.period = 2. * PI64 * (self.semimajor_axis.powf(3.) / self.grav_parameter).sqrt();

        let mean_anomaly = self.mean_anomaly(t % self.period);
        let eccentric_anomaly = self.eccentric_anomaly(mean_anomaly);
        let true_anomaly = self.true_anomaly(eccentric_anomaly);
        let distance = self.distance(eccentric_anomaly);

        let x = distance * true_anomaly.cos();
        let y = distance * true_anomaly.sin();
        let z = 0.;

        let coords = arr1(&[x, y, z]);

        // cos Ω cos ω − sin Ω sin ω cos i 
        let i_1_1 = self.longitude_asc_node.cos() * self.arg_of_periapsis.cos() - self.longitude_asc_node.sin() * self.arg_of_periapsis.sin() * self.inclination.cos();
        // − cos Ω sin ω − sin Ω cos ω cos i
        let i_1_2  = - self.longitude_asc_node.cos() * self.arg_of_periapsis.sin() - self.longitude_asc_node.sin() * self.arg_of_periapsis.cos() * self.inclination.cos();
        // sin Ω sin i
        let i_1_3 = self.longitude_asc_node.sin() * self.inclination.sin();

        // sin Ω cos ω + cos Ω sin ω cos i
        let i_2_1 = self.longitude_asc_node.sin() * self.arg_of_periapsis.cos() + self.longitude_asc_node.cos() * self.arg_of_periapsis.sin() * self.inclination.cos();
        // − sin Ω sin ω + cos Ω cos ω cos i
        let i_2_2 = - self.longitude_asc_node.sin() * self.arg_of_periapsis.sin() + self.longitude_asc_node.cos() * self.arg_of_periapsis.cos() * self.inclination.cos();
        // − cos Ω sin i
        let i_2_3 = - self.longitude_asc_node.cos() * self.inclination.sin();

        // sin ω sin i
        let i_3_1 = self.arg_of_periapsis.sin() * self.inclination.sin();
        // cos ω sin i
        let i_3_2 = self.arg_of_periapsis.cos() * self.inclination.sin();
        // cos i
        let i_3_3 = self.inclination.cos();

        let trans = arr2(&[
            [i_1_1, i_1_2, i_1_3],
            [i_2_1, i_2_2, i_2_3],
            [i_3_1, i_3_2, i_3_3],
        ]);

        let final_coords = trans.dot(&coords);

        return Vec3::new(
            final_coords[0] as f32,
            final_coords[1] as f32,
            final_coords[2] as f32,
        );
    }

    pub fn mean_anomaly(&self, t: f64) -> f64 {
        // println!("Expected mean anomaly = {}",2.45638088);
        // println!("Actual mean anomaly = {}", self.mean_anomaly_at_epoch + self.mean_motion() * t);
        (self.mean_anomaly_at_epoch + self.mean_motion() * t) % (2. * std::f64::consts::PI) 
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

    pub fn mean_motion(&self) -> f64 {
        (2. * std::f64::consts::PI) / self.period
    }

    pub fn true_anomaly(&self, eccentric_anomaly: f64) -> f64 {
        let t_a = 2. * (((1. + self.eccentricity) / (1. - self.eccentricity)).sqrt()
            * (eccentric_anomaly / 2.).tan())
        .atan();

        // println!("True anomaly = {}, expected = {}", t_a, 2.53343322);
        t_a
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

pub fn rotate_earth(
    mut query: Query<&mut Transform, With<EarthBody>>,
    physics_time_q: Query<&PhysicsTime>,
) {
    let physics_time = physics_time_q.single();

    if physics_time.mode == PhysicsTimeMode::StopTick {
        return;
    }

    for mut transform in &mut query {
        let val = (physics_time.delta_seconds / 86400.0) * 2. * std::f64::consts::PI;
        transform.rotate_y(val as f32);
    }
}

pub fn rotate_moon(
    mut query: Query<&mut Transform, With<MoonBody>>,
    physics_time_q: Query<&mut PhysicsTime>,
    lunar_orbit: ResMut<LunarOrbit>,
) {
    let physics_time = physics_time_q.single();

    if physics_time.mode == PhysicsTimeMode::StopTick {
        return;
    }

    for mut transform in &mut query {
        transform.rotate_y(
            ((physics_time.delta_seconds
                / lunar_orbit.orbit.rotational_period)
                * 2.
                * std::f64::consts::PI) as f32,
        );
    }
}

pub fn moon_orbit(
    mut body_query: Query<&mut Transform, With<MoonBody>>,
    orbit: Res<LunarOrbit>,
    physics_time_q: Query<&PhysicsTime>,
) {
    let physics_time = physics_time_q.single();

    let mut posn = orbit
        .orbit
        .position(physics_time.clock_seconds);

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
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(1., 0.0, 0.0, 1.),
                    emissive: Color::rgba(1., 0., 0., 1.),
                    unlit: true,
                    ..default()
                }),
                ..default()
            },
            lines::OrbitalLines,
            NotShadowCaster,
            NotShadowReceiver,
        ));
        println!("Orbit changed.");
    }
}
