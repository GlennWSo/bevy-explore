use std::io::Repeat;
use std::iter;
use std::ops::Add;
use std::ops::Deref;
use std::ops::DerefMut;

use bevy::prelude::*;

use bevy::utils::HashMap;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use crate::assets::Assets;
use crate::collide::Collider;
use crate::collide::CollisionDamage;
use crate::health::Health;
use crate::movement::MovingObj;
use crate::movement::Velocity;
use crate::schedule::InGameSet;
use crate::ship::Player;
use crate::ship::SpaceShip;

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
            .register_type::<Zones>()
            .register_type::<Zone>()
            .register_type::<ZoneState>()
            .register_type::<Population>()
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
            //     despawn::despawn_far::<Astroid, 1000>.in_set(InGameSet::Despawn),
            // )
            .add_systems(Update, despawn_out_of_zone.in_set(InGameSet::Despawn))
            .add_systems(Update, rotate_astriods.in_set(InGameSet::EntityUpdate));
    }
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
pub struct Astroid {
    bulk: u8,
}

impl Astroid {
    const ROTATION_SPEED: f32 = 1.0;
    const RADIUS_MOD: f32 = 2.5;
    /// hit point scaling with size
    const LIFE_MOD: i32 = 10;
    /// hit point scaling with size
    const DAMAGE_MOD: i32 = 5;

    fn spawn(
        &self,
        assets: &Res<Assets>,
        particles: impl Iterator<Item = (Vec2, Velocity)>,
        cmds: &mut Commands,
        bundle: impl Bundle + Copy,
    ) {
        let batch: Box<[_]> = particles
            .map(|(particle)| {
                let transform = Transform::from_translation(particle.0.extend(0.0));
                self.bundle(assets, transform, particle.1)
            })
            .collect();
        cmds.spawn_batch(batch);
    }

    fn damage(&self) -> CollisionDamage {
        CollisionDamage(self.bulk as i32 * Self::DAMAGE_MOD)
    }

    fn health(&self) -> Health {
        Health {
            life: self.bulk as i32 * Self::LIFE_MOD,
            ..Default::default()
        }
    }

    fn collider(&self) -> Collider {
        Collider {
            radius: self.radius(),
            ..Default::default()
        }
    }

    fn radius(&self) -> f32 {
        (self.bulk as f32).sqrt() * Self::RADIUS_MOD
    }

    const SPEED_MOD: f32 = 2.0;
    fn random_velocity() -> Velocity {
        let mut rng = rand::thread_rng();

        let v_unit = random_unit_vec(&mut rng);
        let factor: f32 = rng.gen_range(0.0..Self::SPEED_MOD);
        Velocity(v_unit * factor)
    }

    fn scale(&self) -> Vec3 {
        Vec3::splat(self.radius() / 2.)
    }
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

fn random_unit_vec(rng: &mut impl Rng) -> Vec2 {
    let x = rng.gen_range(-1.0..1.0);
    let y = rng.gen_range(-1.0..1.0);
    Vec2::new(x, y).normalize_or_zero()
}

fn rotate_astriods(mut q: Query<&mut Transform, With<Astroid>>, time: Res<Time>) {
    let rot = Astroid::ROTATION_SPEED * time.delta_seconds();
    for mut trans in q.iter_mut() {
        trans.rotate_local_z(rot);
    }
}

// #[derive(Component, Clone, Copy)]
// struct Shard;

fn split_dead(
    mut cmds: Commands,
    q: Query<(&Health, &Transform, &Velocity, &Astroid)>,
    assets: Res<Assets>,
) {
    for (health, &transform, &velocity, Astroid { bulk }) in q.iter() {
        if **health > 0 {
            continue;
        }
        let velicities = explode_veclocity(velocity, *bulk as usize - 1);
        println!("explosion: {:#?}", velicities);
        let particles = velicities
            .into_iter()
            .map(|v| (transform.translation.truncate(), v));
        let astroid = Astroid { bulk: 1 };
        astroid.spawn(&assets, particles, &mut cmds, ());
    }
}

/// create vectors moving away from vector
fn explode_veclocity(origin_velocity: Velocity, n: usize) -> Vec<Velocity> {
    let mut rng = rand::thread_rng();
    let base_speed: f32 = rng.gen_range(2.5..10.);

    let mut v = random_unit_vec(&mut rng) * base_speed;
    let section_angle = 360.0 / n as f32;
    // let rot = Quat::from_rotation_y(angle.to_radians());

    (0..n)
        .map(|_| {
            let speed_mod = rng.gen_range(0.8..1.25);
            let angle_mod = rng.gen_range(0.8..1.25);
            let rot = Quat::from_rotation_z(section_angle.to_radians() * angle_mod);
            v = rot.mul_vec3(v.extend(0.0)).truncate() * speed_mod + *origin_velocity;
            Velocity(v)
        })
        .collect()
}

#[derive(Component, Reflect, Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
struct Zone {
    row: i32,
    col: i32,
}

impl From<Vec2> for Zone {
    fn from(position: Vec2) -> Self {
        let col = (position[0] / Self::SIZE / 2.).round() as i32;
        let row = (position[1] / Self::SIZE / 2.).round() as i32;
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
    const SIZE: f32 = 200.0;
    #[allow(dead_code)]
    fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }

    fn center(&self) -> Vec2 {
        let x = self.col as f32 * Self::SIZE * 2.;
        let y = self.row as f32 * Self::SIZE * 2.;
        Vec2 { x, y }
    }

    #[allow(dead_code)]
    fn distance(&self, rhs: Vec2) -> f32 {
        self.center().distance(rhs)
    }

    fn min_x(&self) -> f32 {
        self.center()[0] - Self::SIZE
    }
    fn max_x(&self) -> f32 {
        self.center()[0] + Self::SIZE
    }
    fn min_y(&self) -> f32 {
        self.center()[1] - Self::SIZE
    }
    fn max_y(&self) -> f32 {
        self.center()[1] + Self::SIZE
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

    pub fn rand_coordinates(&self) -> impl Iterator<Item = Vec2> {
        let rng: Pcg64 = Seeder::from(self).make_rng();
        let x_range = rand::distributions::Uniform::new(self.min_x(), self.max_x());
        let x_iter = rng.sample_iter(x_range);

        let offset = Zone {
            row: 1337,
            col: 7331,
        };
        let zone2 = *self + &offset;
        let rng: Pcg64 = Seeder::from(zone2).make_rng();
        let y_range = rand::distributions::Uniform::new(self.min_y(), self.max_y());
        let y_iter = rng.sample_iter(y_range);

        x_iter.zip(y_iter).map(|(x, y)| Vec2 { x, y })
    }

    fn inside(&self, v: Vec2) -> bool {
        if v.x < self.min_x() {
            return false;
        }
        if v.x >= self.max_x() {
            return false;
        }
        if v.y < self.min_y() {
            return false;
        }
        if v.y >= self.max_y() {
            return false;
        }
        true
    }
}

trait Extra {
    type Extras: Bundle + Sized;
    fn extra(self) -> Self::Extras;
}

trait Stage {
    fn stage(&self, assets: &Res<Assets>, transform: Transform) -> SceneBundle;
}
trait IntoMovingBundle {
    type Extras: Bundle + Sized;
    fn moving_obj(
        &self,
        assets: &Res<Assets>,
        transform: Transform,
        velocity: Velocity,
    ) -> MovingObj;
    fn bundle(
        self,
        assets: &Res<Assets>,
        transform: Transform,
        velocity: Velocity,
    ) -> (MovingObj, Self::Extras);
}

impl<C, T> IntoMovingBundle for T
where
    C: Bundle + Sized,
    T: Extra<Extras = C> + Stage,
{
    type Extras = C;

    fn moving_obj(
        &self,
        assets: &Res<Assets>,
        transform: Transform,
        velocity: Velocity,
    ) -> MovingObj {
        let obj = MovingObj {
            velocity,
            model: self.stage(assets, transform),
            ..Default::default()
        };
        obj
    }

    fn bundle(
        self,
        assets: &Res<Assets>,
        transform: Transform,
        velocity: Velocity,
    ) -> (MovingObj, Self::Extras) {
        (self.moving_obj(assets, transform, velocity), self.extra())
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Reflect, Clone, Copy)]
enum Seed {
    Rock(Astroid),
}

impl Extra for Astroid {
    type Extras = (Astroid, Health, Collider, CollisionDamage, Name);

    fn extra(self) -> Self::Extras {
        (
            self,
            self.health(),
            self.collider(),
            self.damage(),
            Name::new("Astroid"),
        )
    }
}

impl Stage for Astroid {
    fn stage(&self, assets: &Res<Assets>, transform: Transform) -> SceneBundle {
        let transform = transform.with_scale(self.scale());
        SceneBundle {
            transform,
            scene: assets.astriod.clone(),
            ..Default::default()
        }
    }
}

#[derive(Default, Debug, Reflect)]
#[reflect(Default)]
struct Population {
    map: HashMap<Seed, u32>,
}

impl Population {
    fn spawn(&self, cmds: &mut Commands, assets: &Res<Assets>, zone: Zone) {
        let mut coords = zone.rand_coordinates();
        for (seed, count) in &self.map {
            let coords = coords.by_ref().take(*count as usize);
            let bundles: Box<[_]> = coords
                .map(|coord| match seed {
                    Seed::Rock(astriod) => {
                        let velocity = Astroid::random_velocity();
                        let transform = Transform::from_translation(coord.extend(0.0));
                        astriod.bundle(&assets, transform, velocity)
                    }
                })
                .collect();
            cmds.spawn_batch(bundles);
        }
        // for (seed, count) in self.map.iter() {
        //     let n = *count as usize;
        //     let particles = zone
        //         .rand_coordinates()
        //         .map(|coord| (coord, Astroid::random_velocity()))
        //         .take(n);
        //     match seed {
        //         Seed::Rock(astriod) => astriod.spawn(assets, particles, cmds, ()),
        //     };
        // }
    }
}

impl From<Zone> for Population {
    fn from(zone: Zone) -> Self {
        let mut rng: Pcg64 = Seeder::from(zone).make_rng();
        let n: u8 = rng.gen_range(1..200);
        let size_dist = rand_distr::Binomial::new(15, 0.1).unwrap();
        let astriods = rng.sample_iter(size_dist).map(|rand| {
            let bulk = ((rand + 1).pow(2) - 1) as u8;
            Astroid { bulk }
        });
        let mut map: HashMap<Seed, _> = HashMap::new();
        for astriod in astriods.take(n as usize) {
            let res = map.try_insert(Seed::Rock(astriod), 1);
            match res {
                Ok(_) => (),
                Err(occupied) => {
                    let count = occupied.entry.into_mut();
                    *count += 1;
                }
            }
        }
        Population { map }
    }
}

impl From<AstriodBundle> for Astroid {
    fn from(bundle: AstriodBundle) -> Self {
        bundle.astriod
    }
}

#[derive(Bundle)]
struct AstriodBundle {
    mover: MovingObj,
    astriod: Astroid,
    damage: CollisionDamage,
    health: Health,
    name: Name,
}

#[derive(Debug, Default, Reflect)]
enum ZoneState {
    #[default]
    Spawned,
    Despawned(Population),
}

impl From<Population> for ZoneState {
    fn from(pop: Population) -> Self {
        Self::Despawned(pop)
    }
}

impl ZoneState {
    fn insert(&mut self, seed: Seed) {
        match self {
            ZoneState::Spawned => {
                let mut map = HashMap::new();
                map.insert_unique_unchecked(seed, 1);
                *self = ZoneState::Despawned(Population { map })
            }
            ZoneState::Despawned(pop) => match pop.map.get_mut(&seed) {
                Some(count) => *count += 1,
                None => {
                    pop.map.insert_unique_unchecked(seed, 1);
                }
            },
        }
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct Zones {
    // active: SpawnZone,
    state: HashMap<Zone, ZoneState>,
}
impl Zones {
    fn insert(&mut self, zone: Zone, seed: Seed) {
        match self.state.get_mut(&zone) {
            Some(pop) => pop.insert(seed),
            None => {
                let pop: Population = zone.into();
                let mut depop: ZoneState = pop.into();
                depop.insert(seed);
                self.state.insert_unique_unchecked(zone, depop);
            }
        }
    }
}

fn init_zone(mut cmds: Commands, mut zones: ResMut<Zones>, assets: Res<Assets>) {
    let rock = Astroid { bulk: 5 };
    let mut map = HashMap::with_capacity(1);
    map.insert_unique_unchecked(Seed::Rock(rock), 4);
    let pop = Population { map };
    let zone = [0, 0].into();
    pop.spawn(&mut cmds, &assets, zone);
    zones
        .state
        .insert_unique_unchecked(zone, ZoneState::Spawned);
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

    let zone: Zone = player.translation.truncate().into();
    for zone in zone.neighbors() {
        match zones.state.get(&zone) {
            None => {
                let pop: Population = zone.into();
                pop.spawn(&mut cmds, &assets, zone);
                zones
                    .state
                    .insert_unique_unchecked(zone, ZoneState::Spawned);
            }
            Some(depop) => {
                if let ZoneState::Despawned(pop) = depop {
                    pop.spawn(&mut cmds, &assets, zone);
                    zones.state.insert(zone, ZoneState::Spawned);
                }
            }
        }
    }
}

const DESPAWN_DIST: f32 = Zone::SIZE * 5.;

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
            ZoneState::Spawned => true,
            ZoneState::Despawned(_) => false,
        })
        .filter_map(|(zone, _)| {
            (zone.center().distance(player.translation.truncate()) > DESPAWN_DIST).then(|| zone)
        })
    {
        writer.send(DespawnEvent { zone: *zone });
    }
}

fn despawn_out_of_zone(
    mut cmds: Commands,
    q: Query<(Entity, &Transform, &Astroid)>,
    player_q: Query<&Transform, With<Player>>,
    mut zones: ResMut<Zones>,
) {
    let dist = Zone::SIZE * 8.;
    let Ok(player) = player_q.get_single() else {
        return;
    };
    for (ent, trans, &astroid) in q.iter() {
        let distance = trans.translation.distance(player.translation);
        if distance > dist {
            let zone: Zone = trans.translation.truncate().into();
            cmds.entity(ent).despawn_recursive();
            zones.insert(zone, Seed::Rock(astroid));
        }
    }
}
fn despawn_zone(
    mut cmds: Commands,
    q: Query<(Entity, &Transform, &Astroid)>,
    mut reader: EventReader<DespawnEvent>,
    mut zones: ResMut<Zones>,
) {
    for event in reader.read() {
        q.iter().for_each(|(ent, transform, astriod)| {
            if event.zone.inside(transform.translation.truncate()) {
                let pop = zones.state.get_mut(&event.zone).unwrap();
                pop.insert(Seed::Rock(*astriod));
                cmds.entity(ent).despawn_recursive();
            }
        });
        match zones.state.get_mut(&event.zone).unwrap() {
            ZoneState::Despawned(_) => (),
            pop => {
                *pop = ZoneState::Despawned(Population::default());
            }
        }
    }
}
