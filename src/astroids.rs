use std::borrow::BorrowMut;
use std::ops::Add;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Range;

use bevy::prelude::*;

use bevy::utils::HashMap;
use bevy::utils::HashSet;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use crate::assets::Assets;
use crate::collide::Collider;
use crate::collide::CollisionDamage;
use crate::despawn;
use crate::health::Health;
use crate::movement::Acc;
use crate::movement::MovingObj;
use crate::movement::Velocity;
use crate::schedule::InGameSet;
use crate::ship::Player;
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
            .add_event::<DespawnEvent>()
            .add_systems(Startup, init_zone)
            .add_systems(
                Update,
                (despawn_oob_zones, despawn_zone)
                    .chain()
                    .in_set(InGameSet::Despawn),
            )
            .add_systems(Update, spawn_zones.in_set(InGameSet::Spawn))
            .add_systems(Update, split_dead.in_set(InGameSet::Spawn))
            // .add_systems(Update, despawn_astroid.in_set(InGameSet::Despawn))
            // .add_systems(
            //     Update,
            //     despawn::despawn_far::<Astroid>.in_set(InGameSet::Despawn),
            // )
            .add_systems(Update, rotate_astriods.in_set(InGameSet::EntityUpdate));
    }
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Astroid {
    bulk: u8,
}

impl Astroid {
    const ROTATION_SPEED: f32 = 1.0;
    const RADIUS: f32 = 2.5;
    const LIFE: i32 = 40;
    const HEALTH: Health = Health {
        life: Self::LIFE,
        death_cry: crate::health::DeathCry::None,
    };

    const DAMAGE: CollisionDamage = CollisionDamage(20);
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
    q: Query<(&Health, &Transform, &Velocity, &Astroid), Without<Shard>>,
    assets: Res<Assets>,
) {
    for (health, &transform, &velocity, Astroid { bulk }) in q.iter() {
        if **health > 0 {
            continue;
        }
        let objs = random_shards(&assets.astriod, transform, velocity, 5);

        for obj in objs {
            let rock = (
                obj,
                Shard,
                Astroid { bulk: 1 },
                Health {
                    life: 20,
                    ..Default::default()
                },
                Astroid::DAMAGE,
            );
            cmds.spawn(rock);
        }
    }
}

/// create N objects moving away from input
fn random_shards(
    asset: &Handle<Scene>,
    mut transform: Transform,
    origin_velocity: Velocity,
    n: usize,
) -> Box<[MovingObj]> {
    let mut rng = rand::thread_rng();
    let base_speed: f32 = rng.gen_range(2.5..10.);

    let mut v = random_unit_vec(&mut rng) * base_speed;
    let section_angle = 360.0 / n as f32;
    // let rot = Quat::from_rotation_y(angle.to_radians());

    let explosion: Box<[Vec3]> = (0..n)
        .map(|_| {
            let speed_mod = rng.gen_range(0.8..1.25);
            let angle_mod = rng.gen_range(0.8..1.25);
            let rot = Quat::from_rotation_y(section_angle.to_radians() * angle_mod);
            v = rot.mul_vec3(v) * speed_mod;
            v
        })
        .collect();

    let factor = 0.7;
    transform.scale *= Vec3::ONE * factor;

    explosion
        .into_iter()
        .map(|v| {
            let model = SceneBundle {
                scene: asset.clone(),
                transform,
                ..Default::default()
            };
            let collider = Collider::new(Astroid::RADIUS);
            let velocity = (*v + *origin_velocity).into();
            let acc = Acc::default();

            MovingObj {
                model,
                velocity,
                acc,
                collider,
            }
        })
        .collect()
}

#[derive(Component, Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
struct Zone {
    row: i32,
    col: i32,
}

impl From<Vec3> for Zone {
    fn from(position: Vec3) -> Self {
        let col = (position[0] / Self::SIZE / 2.).round() as i32;
        let row = (position[2] / Self::SIZE / 2.).round() as i32;
        Self { col, row }
    }
}

impl From<[i32; 2]> for Zone {
    fn from(value: [i32; 2]) -> Self {
        Self {
            row: value[0],
            col: value[1],
        }
    }
}

impl Add<&Zone> for Zone {
    type Output = Zone;

    fn add(self, rhs: &Zone) -> Self::Output {
        let row = self.row + rhs.row;
        let col = self.col + rhs.col;
        Self { row, col }
    }
}

// #[allow(dead_code)]
impl Zone {
    /// halfsize of square
    const SIZE: f32 = 80.0;
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
        let rng: Pcg64 = Seeder::from(self).make_rng();
        let x_range = rand::distributions::Uniform::new(self.min_x(), self.max_x());
        let x_iter = rng.sample_iter(x_range);

        let offset = Zone {
            row: 1337,
            col: 7331,
        };
        let zone2 = *self + &offset;
        let rng: Pcg64 = Seeder::from(zone2).make_rng();
        let z_range = rand::distributions::Uniform::new(self.min_z(), self.max_z());
        let z_iter = rng.sample_iter(z_range);

        x_iter.zip(z_iter).map(|(x, z)| Vec3 { x, y: 0., z })
    }

    fn inside(&self, v: Vec3) -> bool {
        if v.x < self.min_x() {
            return false;
        }
        if v.x >= self.max_x() {
            return false;
        }
        if v.z < self.min_z() {
            return false;
        }
        if v.z >= self.max_z() {
            return false;
        }
        true
    }
}

#[derive(Default, Debug)]
struct Population {
    rocks: HashMap<AstroidSeed, u32>,
}
impl Population {
    fn spawn(&self, cmds: &mut Commands, assets: &Res<Assets>, zone: Zone) {
        for (seed, count) in self.rocks.iter() {
            let n = *count as usize;
            seed.spawn(assets, zone.rand_coordinates().take(n), cmds);
        }
    }
}

impl From<Zone> for Population {
    fn from(zone: Zone) -> Self {
        let mut rng: Pcg64 = Seeder::from(zone).make_rng();
        let n: u8 = rng.gen_range(1..60);
        let health_dist = Uniform::from(10..40);
        let seeds = rng.sample_iter(health_dist).map(|life| {
            let health = Health {
                ..Default::default()
            };
            AstroidSeed(Astroid { bulk: 5 }, Astroid::DAMAGE, health)
        });
        let mut map = HashMap::new();
        map.reserve(10);
        for seed in seeds.take(n as usize) {
            let res = map.try_insert(seed, 1);
            match res {
                Ok(_) => (),
                Err(mut occupied) => {
                    let mut count = occupied.entry.into_mut();
                    *count += 1;
                }
            }
        }
        Population { rocks: map }
    }
}

type AstroidTuple = (Astroid, CollisionDamage, Health);

#[derive(Debug, Bundle, Hash, PartialEq, Eq, Clone, Copy)]
struct AstroidSeed(Astroid, CollisionDamage, Health);

impl AstroidSeed {
    fn spawn(&self, assets: &Res<Assets>, coords: impl Iterator<Item = Vec3>, cmds: &mut Commands) {
        let batch: Box<[_]> = coords
            .map(|coord| {
                error!("spawning at: {coord}");
                let transform = Transform {
                    translation: coord,
                    ..Default::default()
                };
                let model = SceneBundle {
                    scene: assets.astriod.clone(),
                    transform,
                    ..Default::default()
                };
                let collider = Collider {
                    radius: Astroid::RADIUS,
                    ..Default::default()
                };
                let obj = MovingObj {
                    model,
                    collider,
                    ..Default::default()
                };
                AstriodBundle(obj, self.clone())
            })
            .collect();
        cmds.spawn_batch(batch);
    }
}

impl From<AstriodBundle> for AstroidSeed {
    fn from(bundle: AstriodBundle) -> Self {
        bundle.1
    }
}

#[derive(Bundle)]
struct AstriodBundle(MovingObj, Astroid, CollisionDamage, Health);

#[derive(Debug)]
enum DePopulation {
    Spawned,
    Despawned(Population),
}
impl DePopulation {
    fn insert(&mut self, rock: AstroidSeed) {
        match self {
            DePopulation::Spawned => {
                let mut map = HashMap::new();
                map.insert_unique_unchecked(rock, 1);
                *self = DePopulation::Despawned(Population { rocks: map })
            }
            DePopulation::Despawned(pop) => match pop.rocks.get_mut(&rock) {
                Some(count) => *count += 1,
                None => {
                    pop.rocks.insert_unique_unchecked(rock, 1);
                }
            },
        }
    }
}

#[derive(Resource, Default)]
struct Zones {
    // active: SpawnZone,
    state: HashMap<Zone, DePopulation>,
}

fn init_zone(mut cmds: Commands, mut zones: ResMut<Zones>, assets: Res<Assets>) {
    let rock_type = AstroidSeed(Astroid { bulk: 5 }, Astroid::DAMAGE, Astroid::HEALTH);
    let mut rocks = HashMap::with_capacity(1);
    rocks.insert_unique_unchecked(rock_type, 4);
    let pop = Population { rocks };
    let zone = [0, 0].into();
    pop.spawn(&mut cmds, &assets, zone);
    zones
        .state
        .insert_unique_unchecked(zone, DePopulation::Spawned);
}
fn spawn_zones(
    mut cmds: Commands,
    q: Query<&Transform, With<SpaceShip>>,
    mut zones: ResMut<Zones>,
    assets: Res<Assets>,
) {
    let Ok(player) = q.get_single() else {
        return;
    };

    let zone: Zone = player.translation.into();
    for zone in zone.neighbors() {
        match zones.state.get(&zone) {
            None => {
                let pop: Population = zone.into();
                pop.spawn(&mut cmds, &assets, zone);
                error!("Spawining Zone:{:?} {:?}", zone, zone.center());
                zones
                    .state
                    .insert_unique_unchecked(zone, DePopulation::Spawned);
            }
            Some(depop) => {
                if let DePopulation::Despawned(pop) = depop {
                    pop.spawn(&mut cmds, &assets, zone);
                    zones.state.insert(zone, DePopulation::Spawned);
                }
            }
        }
    }
}

fn init_rocks(mut cmds: Commands, assets: Res<Assets>) {
    let start_rect = Zone::new(0, 0);
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
        Astroid { bulk: 5 },
        Health {
            life: Astroid::LIFE,
            ..Default::default()
        },
        Astroid::DAMAGE,
    );
    cmds.spawn(rock);
}

const DESPAWN_DIST: f32 = 1000.;

#[derive(Event)]
struct DespawnEvent {
    zone: Zone,
}

fn despawn_oob_zones(
    q: Query<&Transform, With<Player>>,
    mut writer: EventWriter<DespawnEvent>,
    zones: Res<Zones>,
) {
    let Ok(player) = q.get_single() else {
        return;
    };
    for zone in zones
        .state
        .iter()
        .filter(|(_, pop)| match pop {
            DePopulation::Spawned => true,
            DePopulation::Despawned(_) => false,
        })
        .filter_map(|(zone, _)| {
            (zone.center().distance(player.translation) > DESPAWN_DIST).then(|| zone)
        })
    {
        writer.send(DespawnEvent { zone: *zone });
    }
}

fn despawn_zone(
    mut cmds: Commands,
    q: Query<(Entity, &Transform, &Astroid, &Health, &CollisionDamage), Without<Shard>>,
    mut reader: EventReader<DespawnEvent>,
    mut zones: ResMut<Zones>,
) {
    for event in reader.read() {
        error!("despawning zone: {:?}", event.zone);
        q.iter()
            .for_each(|(ent, transform, _astriod, health, dmg)| {
                if event.zone.inside(transform.translation) {
                    let seed = AstroidSeed(Astroid { bulk: 5 }, *dmg, *health);
                    let pop = zones.state.get_mut(&event.zone).unwrap();
                    pop.insert(seed);
                    // error!("pop {:?}", pop);
                    cmds.entity(ent).despawn_recursive();
                }
            });
        match zones.state.get_mut(&event.zone).unwrap() {
            DePopulation::Despawned(_) => (),
            pop => {
                error!("zone was empty");
                *pop = DePopulation::Despawned(Population::default());
            }
        }
    }
}
