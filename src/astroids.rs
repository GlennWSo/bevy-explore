use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Range;

use bevy::prelude::*;

use rand::Rng;

use crate::assets::Assets;
use crate::collide::Collider;
use crate::collide::CollisionDamage;
use crate::health::Health;
use crate::movement::Acc;
use crate::movement::MovingObj;
use crate::movement::Velocity;
use crate::schedule::InGameSet;

const SPAWN_RANGE_X: Range<f32> = -25.0..25.;
const SPAWN_RANGE_Y: Range<f32> = 0.0..25.;

const VELOCITY_SCALAR: f32 = 5.0;
const ACC_SCALAR: f32 = 1.0;
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
            .add_systems(Update, split_dead.in_set(InGameSet::Spawn))
            .add_systems(
                Update,
                (rotate_astriods, spawn_astriod).in_set(InGameSet::EntityUpdate),
            );
    }
}

#[derive(Component, Debug)]
pub struct Astroid;

impl Astroid {
    const ROTATION_SPEED: f32 = 1.0;
    const RADIUS: f32 = 2.5;
    const HEALTH: i32 = 40;
    const DAMAGE: i32 = 20;
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

#[derive(Component)]
struct Shard;

fn split_dead(
    mut cmds: Commands,
    q: Query<(&Health, &Transform, &Velocity), (With<Astroid>, Without<Shard>)>,
    assets: Res<Assets>,
) {
    for (health, &transform, &velocity) in q.iter() {
        if **health > 0 {
            continue;
        }
        let objs = random_shards(&assets.astriod, transform, velocity);
        let rocks = objs.map(|obj| {
            (
                obj,
                Shard,
                Astroid,
                Health(Astroid::HEALTH),
                CollisionDamage(Astroid::DAMAGE),
            )
        });

        for rock in rocks {
            cmds.spawn(rock);
        }
    }
}

fn random_shards(
    asset: &Handle<Scene>,
    mut transform: Transform,
    origin_velocity: Velocity,
) -> [MovingObj; 3] {
    let mut rng = rand::thread_rng();
    let speed: f32 = rng.gen_range(2.5..10.);

    let v1 = random_unit_vec(&mut rng) * speed;
    let rot = Quat::from_rotation_y(120.0f32.to_radians());
    let v2 = rot.mul_vec3(v1);
    let v3 = rot.mul_vec3(v2);

    let explosion = [v1, v2, v3];
    let factor = 0.7;
    transform.scale *= Vec3::ONE * factor;

    explosion.map(|v| {
        let model = SceneBundle {
            scene: asset.clone(),
            transform,
            ..Default::default()
        };
        let collider = Collider::new(Astroid::RADIUS);
        let velocity = (v + *origin_velocity).into();
        let acc = Acc::default();

        MovingObj {
            model,
            velocity,
            acc,
            collider,
        }
    })
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
    let obj = MovingObj {
        model,
        velocity,
        acc,
        collider,
    };

    let rock = (
        obj,
        Astroid,
        Health(Astroid::HEALTH),
        CollisionDamage(Astroid::DAMAGE),
    );
    cmd.spawn(rock);
}
