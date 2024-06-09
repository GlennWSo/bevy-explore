use bevy::prelude::*;

#[derive(Component, Debug, Default)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug, Default)]
struct Velocity {
    x: f32,
    y: f32,
}

fn spawn_spaceship(mut cmds: Commands) {
    cmds.spawn((Position::default(), Velocity { x: 1.0, y: 1.0 }));
}

fn update_position(mut q: Query<(&Velocity, &mut Position)>) {
    for (velocity, mut position) in q.iter_mut() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn print_position(q: Query<(Entity, &Position)>) {
    for (entity, position) in q.iter() {
        info!("Enity {:?} is at position {:?}", entity, position)
    }
}

fn main() {
    App::new()
        .add_systems(Startup, spawn_spaceship)
        .add_systems(Update, (update_position, print_position))
        .add_plugins(DefaultPlugins)
        .run();
}
