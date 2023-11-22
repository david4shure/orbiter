use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use chrono::{prelude::*, Duration, DurationRound};
use chrono::offset::LocalResult;

use crate::orbit::{EarthBody, MoonBody, LunarOrbit};

#[derive(Reflect, Component, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct PhysicsTime {
    pub mode: PhysicsTimeMode,
    pub tick_interval_seconds: f64, // when in StopTickMode, and we tick forward time, this determines the interval we wish to tick forward.
    pub delta_seconds: f64, // how many physics seconds have 
    pub clock_seconds: f64, // starts at clock_seconds = 0
    pub scale: f64, // ratio of physics seconds to 1 bevy second, can be negative to turn back time.
}

#[derive(Reflect, PartialEq)]
pub enum PhysicsTimeMode {
    Elapsing,
    StopTick,
}

#[derive(Component)]
pub struct TimeLabel;

impl Default for PhysicsTime {
    fn default() -> Self {
        return PhysicsTime { scale: 1., clock_seconds: 0., delta_seconds: 0., mode: PhysicsTimeMode::Elapsing, tick_interval_seconds: 86400. };
    }
}

pub struct PhysicsTimePlugin;

impl Plugin for PhysicsTimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_physics_clock)
            .add_systems(Update, stop_tick_mode_input_moon)
            .add_systems(Update, stop_tick_mode_input_earth)
            .add_systems(Update, stop_tick)
            .add_systems(Update, draw_date)
            .add_systems(Startup, setup)
            .register_type::<PhysicsTime>();
    }
}

pub fn setup(
    mut commands: Commands, ass: Res<AssetServer>,
) {
    commands.spawn(
        PhysicsTime{
            ..default()
        }
    ).insert(Name::new("Physics Time"));

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        }),
        TimeLabel,
    ));
}

pub fn sync_physics_clock(
    mut physics_time_q: Query<&mut PhysicsTime>,
    mut bevy_time: Res<Time>,
) {
    let mut physics_time = physics_time_q.get_single_mut().unwrap();

    if physics_time.mode == PhysicsTimeMode::StopTick {
        return;
    }

    let bevy_time = bevy_time.delta_seconds_f64();

    physics_time.delta_seconds = physics_time.scale * bevy_time;
    physics_time.clock_seconds += physics_time.delta_seconds;
}

pub fn stop_tick(
    keys: Res<Input<KeyCode>>,
    mut physics_time_q: Query<&mut PhysicsTime>,
) {
    let mut physics_time = physics_time_q.single_mut();

    if physics_time.mode == PhysicsTimeMode::Elapsing {
        return;
    }

    if keys.just_pressed(KeyCode::Right) {
        physics_time.clock_seconds += physics_time.tick_interval_seconds;
    }

    if keys.just_pressed(KeyCode::Left) {
        physics_time.clock_seconds -= physics_time.tick_interval_seconds;
    }
}

pub fn stop_tick_mode_input_moon(
    keys: Res<Input<KeyCode>>,
    mut physics_time_q: Query<&mut PhysicsTime>,
    mut moon_query: Query<&mut Transform, With<MoonBody>>,
    lunar_orbit: ResMut<LunarOrbit>,
) {
    let mut moon_trans = moon_query.single_mut();
    let mut physics_time = physics_time_q.single_mut();

    if physics_time.mode == PhysicsTimeMode::Elapsing {
        return;
    }

    if keys.just_pressed(KeyCode::Right) {
        moon_trans.rotate_y(
            ((physics_time.tick_interval_seconds
                / lunar_orbit.orbit.rotational_period)
                * 2.
                * std::f64::consts::PI) as f32,
        );
    }

    if keys.just_pressed(KeyCode::Left) {
        moon_trans.rotate_y(
            ((-physics_time.tick_interval_seconds
                / lunar_orbit.orbit.rotational_period)
                * 2.
                * std::f64::consts::PI) as f32,
        );
    }
}

pub fn stop_tick_mode_input_earth(
    keys: Res<Input<KeyCode>>,
    mut physics_time_q: Query<&mut PhysicsTime>,
    mut earth_query: Query<&mut Transform, With<EarthBody>>,
) {
    let mut earth_trans = earth_query.single_mut();
    let mut physics_time = physics_time_q.single_mut();

    if physics_time.mode == PhysicsTimeMode::Elapsing {
        return;
    }

    if keys.just_pressed(KeyCode::Right) {
        println!("Right");
        let val = (physics_time.tick_interval_seconds / 86400.0) * 2. * std::f64::consts::PI;
        println!("Rotating for {} radians", val);
        earth_trans.rotate_y(val as f32);
    }

    if keys.just_pressed(KeyCode::Left) {
        println!("Left");
        let val = (physics_time.tick_interval_seconds / 86400.0) * 2. * std::f64::consts::PI;
        println!("Rotating for {} radians", -val);
        earth_trans.rotate_y(-val as f32);
    }
}

pub fn draw_date(
    mut physics_time_q: Query<&mut PhysicsTime>,
    mut text_query: Query<&mut Text, With<TimeLabel>>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    let physics_time = physics_time_q.get_single_mut().unwrap();

    let seconds_since_j2000 = physics_time.clock_seconds;

    println!("Clock seconds {}", seconds_since_j2000);

    let reference_date = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(); // `2014-07-08T09:10:11Z`

    let duration = Duration::milliseconds((seconds_since_j2000 * 1000.) as i64);

    let date = reference_date + duration;

    let date_string = date.to_rfc2822();

    text.sections[0].value = format!(
        "{}",
        date_string
    );
}