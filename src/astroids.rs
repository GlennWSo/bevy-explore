use std::ops::Add;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Range;

use bevy::prelude::*;

use bevy::utils::HashSet;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::assets::Assets;
use crate::collide::Collider;
use crate::collide::CollisionDamage;
use crate::health::Health;
use crate::movement::Acc;
use crate::movement::MovingObj;
use crate::movement::Velocity;
use crate::schedule::InGameSet;
use crate::ship::SpaceShip;

// const VELOCITY_SCALAR: f32 = 5.0;
const VELOCITY_SCALAR: f32 = 0.0;
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
            .init_resource::<Zones>()
            .add_systems(Update, check_player_zone.in_set(InGameSet::Spawn))
            .add_systems(Startup, init_rocks)
            .add_systems(Update, split_dead.in_set(InGameSet::Spawn))
            .add_systems(Update, rotate_astriods.in_set(InGameSet::EntityUpdate));
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
        let objs: [_; 5] = random_shards(&assets.astriod, transform, velocity);
        let rocks = objs.map(|obj| {
            (
                obj,
                Shard,
                Astroid,
                Health {
                    life: Astroid::HEALTH,
                    ..Default::default()
                },
                CollisionDamage(Astroid::DAMAGE),
            )
        });

        for rock in rocks {
            cmds.spawn(rock);
        }
    }
}

/// create N objects moving away from input
fn random_shards<const N: usize>(
    asset: &Handle<Scene>,
    mut transform: Transform,
    origin_velocity: Velocity,
) -> [MovingObj; N] {
    let mut rng = rand::thread_rng();
    let base_speed: f32 = rng.gen_range(2.5..10.);

    let v1 = random_unit_vec(&mut rng) * base_speed;
    let angle = 360.0 / N as f32;
    // let rot = Quat::from_rotation_y(angle.to_radians());

    let mut explosion = [v1; N];
    for (lead, prev) in (1..N).zip(0..N) {
        let prev = explosion[prev];
        let speed_mod = rng.gen_range(0.8..1.25);
        let angle_mod = rng.gen_range(0.8..1.25);
        let rot = Quat::from_rotation_y(angle.to_radians() * angle_mod);
        explosion[lead] = rot.mul_vec3(prev) * speed_mod;
    }

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

#[derive(Resource, Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
struct SpawnZone {
    row: i32,
    col: i32,
}

impl From<Vec3> for SpawnZone {
    fn from(position: Vec3) -> Self {
        let col = (position[0] / Self::SIZE / 2.).round() as i32;
        let row = (position[2] / Self::SIZE / 2.).round() as i32;
        Self { col, row }
    }
}

impl From<[i32; 2]> for SpawnZone {
    fn from(value: [i32; 2]) -> Self {
        Self {
            row: value[0],
            col: value[1],
        }
    }
}

impl Add<&SpawnZone> for SpawnZone {
    type Output = SpawnZone;

    fn add(self, rhs: &SpawnZone) -> Self::Output {
        let row = self.row + rhs.row;
        let col = self.col + rhs.col;
        Self { row, col }
    }
}

impl SpawnZone {
    /// halfsize of square
    const SIZE: f32 = 100.0;
    fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }

    fn center(&self) -> Vec3 {
        let x = self.col as f32 * Self::SIZE * 2.;
        let z = self.row as f32 * Self::SIZE * 2.;
        let y = 0.0;
        Vec3 { x, y, z }
    }

    fn distance(&self, rhs: Vec3) -> f32 {
        self.center().distance(rhs)
    }

    fn min_x(&self) -> f32 {
        self.center()[0] - Self::SIZE
    }
    fn max_x(&self) -> f32 {
        self.center()[0] + Self::SIZE
    }
    fn min_z(&self) -> f32 {
        self.center()[2] - Self::SIZE
    }
    fn max_z(&self) -> f32 {
        self.center()[2] + Self::SIZE
    }

    const ADJECENT: [[i32; 2]; 9] = [
        [-1, -1],
        [-1, 0],
        [-1, 1],
        [0, -1],
        [0, 1],
        [1, -1],
        [1, 0],
        [1, 1],
        [0, 0],
    ];
    fn neighbors(&self) -> [Self; 9] {
        Self::ADJECENT.map(|rc| Into::<Self>::into(rc) + self)
    }

    pub fn rand_coordinates(&self) -> impl Iterator<Item = Vec3> {
        let rng = rand::thread_rng();
        let x_range = rand::distributions::Uniform::new(self.min_x(), self.max_x());
        let x_iter = rng.sample_iter(x_range);

        let rng = rand::thread_rng();
        let z_range = rand::distributions::Uniform::new(self.min_z(), self.max_z());
        let z_iter = rng.sample_iter(z_range);

        x_iter.zip(z_iter).map(|(x, z)| Vec3 { x, y: 0., z })
    }

    fn spawn(&self, cmds: &mut Commands, assets: &Res<Assets>) {
        for coord in self.rand_coordinates().take(40) {
            spawn_astriod(cmds, assets, coord)
        }
    }
}

#[derive(Resource, Default)]
struct Zones {
    // active: SpawnZone,
    visits: HashSet<SpawnZone>,
}

fn check_player_zone(
    mut cmds: Commands,
    q: Query<&Transform, With<SpaceShip>>,
    mut zones: ResMut<Zones>,
    assets: Res<Assets>,
) {
    let Ok(player) = q.get_single() else {
        return;
    };

    let zone: SpawnZone = player.translation.into();
    for zone in zone.neighbors() {
        if zones.visits.insert(zone) {
            zone.spawn(&mut cmds, &assets);
        }
    }
}

fn init_rocks(mut cmds: Commands, assets: Res<Assets>) {
    let start_rect = SpawnZone::new(0, 0);
    for coord in start_rect.rand_coordinates().take(40) {
        spawn_astriod(&mut cmds, &assets, coord)
    }
}

fn spawn_astriod(cmds: &mut Commands, assets: &Res<Assets>, translation: Vec3) {
    let mut rng = rand::thread_rng();
    let transform = Transform::from_translation(translation);
    let velocity = (random_unit_vec(&mut rng) * VELOCITY_SCALAR).into();
    let acc = Vec3::ZERO.into();

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
        Health {
            life: Astroid::HEALTH,
            ..Default::default()
        },
        CollisionDamage(Astroid::DAMAGE),
    );
    cmds.spawn(rock);
}
