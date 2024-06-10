use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Range;

use bevy::prelude::*;

use rand::Rng;

use crate::assets::Assets;
use crate::collide::Collider;
use crate::movement::MovingObj;

const SPAWN_RANGE_X: Range<f32> = -25.0..25.;
const SPAWN_RANGE_Y: Range<f32> = 0.0..25.;

const VELOCITY_SCALAR: f32 = 5.0;
const ACC_SCALAR: f32 = 1.0;

#[derive(Component, Debug)]
struct Astroid;

impl Astroid {
    const ROTATION_SPEED: f32 = 1.0;
    const RADIUS: f32 = 2.5;
}

#[derive(Resource, Debug)]
struct SpawnTimer(Timer);

impl Deref for SpawnTimer {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpawnTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn random_unit_vec(rng: &mut impl Rng) -> Vec3 {
    let x = rng.gen_range(-1.0..1.0);
    let y = 0.0;
    let z = rng.gen_range(-1.0..1.0);
    Vec3::new(x, y, z).normalize_or_zero()
}

fn rotate_astriods(mut q: Query<&mut Transform, With<Astroid>>, time: Res<Time>) {
    let rot = Astroid::ROTATION_SPEED * time.delta_seconds();
    for mut trans in q.iter_mut() {
        trans.rotate_local_z(rot);
    }
}

fn spawn_astriod(
    mut cmd: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    assets: Res<Assets>,
) {
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    let x = rng.gen_range(SPAWN_RANGE_X);
    let y = 0.0;
    let z = rng.gen_range(SPAWN_RANGE_Y);

    let translation = Vec3::new(x, y, z);
    let transform = Transform::from_translation(translation);
    let velocity = (random_unit_vec(&mut rng) * VELOCITY_SCALAR).into();
    let acc = (random_unit_vec(&mut rng) * ACC_SCALAR).into();

    let model = SceneBundle {
        scene: assets.astriod.clone(),
        transform,
        ..Default::default()
    };
    let collider = Collider::new(Astroid::RADIUS);
    let rock = MovingObj {
        model,
        velocity,
        acc,
        collider,
    };

    cmd.spawn((rock, Astroid));
}

fn handle_astriod_collisions(mut cmds: Commands, q: Query<(Entity, &Collider), With<Astroid>>) {
    for (ent, collider) in q.iter() {
        for &collide_ent in collider.colliding_entities.iter() {
            if q.get(collide_ent).is_ok() {
                continue;
            }
            cmds.entity(ent).despawn_recursive();
        }
    }
}
pub struct AstriodPlug;

impl AstriodPlug {
    /// spawn interval in seconds
    const SPAWN_TIMER: f32 = 1.0;
}

impl Plugin for AstriodPlug {
    fn build(&self, app: &mut App) {
        let timer = Timer::from_seconds(Self::SPAWN_TIMER, TimerMode::Repeating);
        let timer = SpawnTimer(timer);
        app.insert_resource(timer)
            .add_systems(Update, spawn_astriod)
            .add_systems(Update, rotate_astriods)
            .add_systems(Update, handle_astriod_collisions);
    }
}
