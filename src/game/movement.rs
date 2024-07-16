//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/pull/14223).

use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};

use super::{audio::sfx::Sfx, spawn::player::Player};
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.add_systems(
        Update,
        record_movement_controller.in_set(AppSet::RecordInput),
    );

    // Apply movement based on controls.
    app.register_type::<(Movement, WrapWithinWindow)>();
    app.add_systems(
        Update,
        (apply_movement, wrap_within_window)
            .chain()
            .in_set(AppSet::Update),
    );

    // Trigger step sound effects based on controls.
    app.register_type::<StepSfx>();
    app.add_systems(
        Update,
        (
            tick_step_sfx.in_set(AppSet::TickTimers),
            trigger_step_sfx.in_set(AppSet::Update),
        ),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController {
    movement_intent: f32,
    rotation_intent: f32,
}

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut movement_intent = 0.0;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        movement_intent += 1.0;
    }

    let mut rotation_intent = 0.0;
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        rotation_intent += 1.0;
    }

    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        rotation_intent -= 1.0;
    }

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.movement_intent = movement_intent;
        controller.rotation_intent = rotation_intent;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Movement {
    pub movement_speed: f32,
    pub rotation_speed: f32,
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &Movement, &mut Transform)>,
) {
    for (controller, movement, mut transform) in &mut movement_query {
        // update the ship rotation around the Z axis (perpendicular to the 2D plane of the screen)
        transform
            .rotate_z(controller.rotation_intent * movement.rotation_speed * time.delta_seconds());

        // get the ship's forward vector by applying the current rotation to the ships initial facing
        // vector
        let movement_direction = transform.rotation * Vec3::Y;
        // get the distance the ship will move based on direction, the ship's movement speed and delta
        // time
        let movement_distance =
            controller.movement_intent * movement.movement_speed * time.delta_seconds();
        // create the change in translation using the new movement direction and distance
        let translation_delta = movement_direction * movement_distance;
        // update the ship translation with our new translation delta
        transform.translation += translation_delta;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WrapWithinWindow;

fn wrap_within_window(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<WrapWithinWindow>>,
) {
    let size = window_query.single().size() + 50.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}

/// Time between walk sound effects.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct StepSfx {
    pub cooldown_timer: Timer,
}

impl StepSfx {
    pub fn new(cooldown: Duration) -> Self {
        let mut cooldown_timer = Timer::new(cooldown, TimerMode::Once);
        cooldown_timer.set_elapsed(cooldown);
        Self { cooldown_timer }
    }
}

fn tick_step_sfx(time: Res<Time>, mut step_query: Query<&mut StepSfx>) {
    for mut step in &mut step_query {
        step.cooldown_timer.tick(time.delta());
    }
}

/// If the player is moving, play a step sound effect.
fn trigger_step_sfx(
    mut commands: Commands,
    mut step_query: Query<(&MovementController, &mut StepSfx)>,
) {
    for (controller, mut step) in &mut step_query {
        if step.cooldown_timer.finished() && controller.movement_intent != 0.0 {
            step.cooldown_timer.reset();
            commands.trigger(Sfx::Step);
        }
    }
}
