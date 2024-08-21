use crate::astroids::Rock;
use crate::schedule::InitStages;
use crate::ship::Player;
use crate::ship::SpaceShip;
use crate::stage::IntoMovingBundle;
use std::ops::Add;

use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::prelude::{Rng, SliceRandom};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use crate::{assets::MyAssets, astroids::Astroid, schedule::InGameSet};

pub struct ZonePlugin;

impl Plugin for ZonePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Zones>()
            .register_type::<Zones>()
            .register_type::<Zone>()
            .register_type::<ZoneState>()
            .register_type::<Population>()
            .add_event::<DespawnEvent>()
            .add_systems(Startup, init_zone.in_set(InitStages::Spawn))
            .add_systems(
                Update,
                (despawn_oob_zones, despawn_zone)
                    .chain()
                    .in_set(InGameSet::Despawn),
            )
            .add_systems(Update, despawn_out_of_zone.in_set(InGameSet::Despawn))
            .add_systems(Update, spawn_zones.in_set(InGameSet::Spawn));
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Reflect, Clone, Copy)]
pub enum Seed {
    Rock(Astroid),
}
#[derive(Default, Debug, Reflect)]
#[reflect(Default)]
struct Population {
    map: HashMap<Seed, u32>,
}

impl Population {
    #[allow(dead_code)]
    fn size(&self) -> u32 {
        self.map.iter().map(|(_k, v)| v).sum()
    }
    fn spawn_at(
        &self,
        cmds: &mut Commands,
        assets: &Res<MyAssets>,
        mut coords: impl Iterator<Item = Vec2>,
    ) {
        for (seed, count) in &self.map {
            let coords = coords.by_ref().take(*count as usize);
            let bundles: Box<[_]> = coords
                .map(|coord| match seed {
                    Seed::Rock(astriod) => {
                        // let velocity = Velocity::default();
                        let velocity = Astroid::random_velocity();
                        let transform = Transform::from_translation(coord.extend(0.0));
                        astriod.bundle(&assets, transform, velocity)
                    }
                })
                .collect();
            cmds.spawn_batch(bundles);
        }
    }

    fn spawn(&self, cmds: &mut Commands, assets: &Res<MyAssets>, zone: Zone) {
        let coords = zone.rand_coordinates();
        self.spawn_at(cmds, assets, coords);
    }
}

impl From<Zone> for Population {
    fn from(zone: Zone) -> Self {
        let mut rng: Pcg64 = Seeder::from(zone).make_rng();
        let n: u8 = rng.gen_range(10..100);
        let size_dist = rand_distr::Binomial::new(15, 0.1).unwrap();
        let kind_dist = rand_distr::Standard;
        let mut kind_gen = rng.clone().sample_iter(kind_dist);
        let astriods = rng.sample_iter(size_dist).map(|rand| {
            let bulk = ((rand + 1).pow(2)) as u8;
            Astroid {
                bulk,
                kind: kind_gen.next().unwrap(),
            }
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
    const SIZE: f32 = 300.0;
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

    /// TODO change signature
    pub fn grid_coordinates(&self) -> Vec<Vec2> {
        let n = 15;

        let cell_size = Self::SIZE * 2.0 / n as f32;
        let origo = self.center() - Self::SIZE + cell_size / 2.0;
        let coords: Vec<_> = (0..n)
            .flat_map(|r| {
                (0..n).map(move |c| {
                    let x = origo.x + cell_size * c as f32;
                    let y = origo.y + cell_size * r as f32;
                    Vec2 { x, y }
                })
            })
            .collect();
        // coords
        coords
    }

    pub fn rand_coordinates(&self) -> impl Iterator<Item = Vec2> {
        let mut rng: Pcg64 = Seeder::from(self).make_rng();
        // let x_range = rand::distributions::Uniform::new(self.min_x(), self.max_x());
        // let x_iter = rng.sample_iter(x_range);

        // let offset = Zone {
        //     row: 1337,
        //     col: 7331,
        // };
        // let zone2 = *self + &offset;
        // let rng: Pcg64 = Seeder::from(zone2).make_rng();
        // let y_range = rand::distributions::Uniform::new(self.min_y(), self.max_y());
        // let y_iter = rng.sample_iter(y_range);

        // x_iter.zip(y_iter).map(|(x, y)| Vec2 { x, y })
        let mut coords = self.grid_coordinates();
        coords.shuffle(&mut rng);
        coords.into_iter().cycle()
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

fn init_zone(mut cmds: Commands, mut zones: ResMut<Zones>, assets: Res<MyAssets>) {
    let zone: Zone = [0, 0].into();
    let pop: Population = zone.into();
    let coords = zone
        .rand_coordinates()
        .filter(|coord| coord.distance(Vec2::ZERO) > 30.0);
    pop.spawn_at(&mut cmds, &assets, coords);
    zones
        .state
        .insert_unique_unchecked(zone, ZoneState::Spawned);
}
fn spawn_zones(
    mut cmds: Commands,
    q: Query<&Transform, With<SpaceShip>>,
    mut zones: ResMut<Zones>,
    assets: Res<MyAssets>,
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
