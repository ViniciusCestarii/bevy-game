//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/pull/14223).

use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};

use super::audio::sfx::Sfx;
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

    // Update facing based on controls.
    app.add_systems(Update, update_facing.in_set(AppSet::Update));

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
pub struct MovementController(pub Vec2);

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize so that diagonal movement has the same speed as
    // horizontal and vertical movement.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.0 = intent;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Movement {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    /// Note that physics engines may use different unit/pixel ratios.
    pub speed: f32,
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &Movement, &mut Transform)>,
) {
    for (controller, movement, mut transform) in &mut movement_query {
        let velocity = movement.speed * controller.0;
        transform.translation += velocity.extend(0.0) * time.delta_seconds();
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

fn update_facing(mut player_query: Query<(&MovementController, &mut Sprite)>) {
    for (controller, mut sprite) in &mut player_query {
        let dx = controller.0.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }
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
        if step.cooldown_timer.finished() && controller.0 != Vec2::ZERO {
            step.cooldown_timer.reset();
            commands.trigger(Sfx::Step);
        }
    }
}
