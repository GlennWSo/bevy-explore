use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::{prelude::*, tasks::ThreadExecutor};

use crate::{
    assets::MyAssets,
    collide_dmg::CollisionDamage,
    guns::{GunFireEvent, PlasmaGun},
    health::{cry_dead, DeathCry, Health},
    schedule::{InGameSet, InitStages},
    ship::SpaceShip,
    stage::Stage,
};

pub struct SentryPlugin;

impl Plugin for SentryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_dbg_sentry.in_set(InitStages::Spawn));
        app.add_systems(Update, cry_dead::<Sentry>.in_set(InGameSet::Spawn));
        app.add_systems(Update, detect_threat::<SpaceShip>);
        app.add_systems(Update, rotate_sentry);
        app.add_event::<ThreatEvent>();
        app.add_systems(Update, fire_ctrl);
    }
}

#[derive(Component)]
struct Sentry;

#[derive(Component)]
struct Detector<T: Component> {
    phantom: PhantomData<T>,
}

impl<T: Component> Detector<T> {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

fn init_dbg_sentry(mut cmds: Commands, assets: Res<MyAssets>) {
    let transform = Transform::from_xyz(0., 30., 0.);
    let radius = 100.0;
    let collider = Collider::circle(radius);
    let detector = Detector::<SpaceShip>::new();
    let gun = PlasmaGun::default();
    cmds.spawn((Sentry.stage(&assets, transform), gun))
        .with_children(|parrent| {
            parrent.spawn((detector, collider, Sensor));
        });
}

#[derive(Clone, Copy, Debug)]
struct Threat {
    distance: f32,
    radians: f32,
}

#[derive(Event)]
struct ThreatEvent {
    sentry: Entity,
    threats: Box<[Threat]>,
}

fn fire_ctrl(
    mut fire_reporter: EventWriter<GunFireEvent<PlasmaGun>>,
    mut threat_reader: EventReader<ThreatEvent>,
    sentry_q: Query<(&Rotation, &Position), With<Sentry>>,
) {
    for ThreatEvent { sentry, threats } in threat_reader.read() {
        let entity = *sentry;
        let Ok((rot, pos)) = sentry_q.get(entity) else {
            continue;
        };
        let Position(Vec2 { x, y }) = pos;
        let mut transform = Transform::from_xyz(*x, *y, 0.0);
        transform.rotate_z(rot.as_radians() + 90.0_f32.to_radians());
        transform.translation += transform.down() * 6.0;
        let fire_event = GunFireEvent {
            phantom: PhantomData,
            entity,
            transform,
        };
        fire_reporter.send(fire_event);
    }
}

fn rotate_sentry(
    mut sentry_q: Query<&mut Transform, With<Sentry>>,
    mut threat_reader: EventReader<ThreatEvent>,
    time: Res<Time>,
) {
    for ThreatEvent { sentry, threats } in threat_reader.read() {
        trace!("Sentry: {}, dected theats:{:?}", sentry, threats);
        let Ok(mut sentry_transform) = sentry_q.get_mut(*sentry) else {
            continue;
        };
        let threat1 =
            threats
                .iter()
                .copied()
                .reduce(|acc, e| if e.distance < acc.distance { e } else { acc });
        if let Some(Threat { radians, .. }) = threat1 {
            let v = 20.0_f32.to_radians();
            let action = radians.clamp(-v, v);
            sentry_transform.rotate_z(action * time.delta_seconds());
        }
    }
}

fn detect_threat<T: Component>(
    sensor_q: Query<(&Parent, &CollidingEntities, &Position, &Rotation), With<Detector<T>>>,
    threat_q: Query<&Transform, With<T>>,
    mut reporter: EventWriter<ThreatEvent>, // parrent_q: Query<&Transform>,
) {
    for (sentry, collisions, pos, &rot) in sensor_q.iter() {
        let threats: Box<_> = collisions
            .iter()
            .filter_map(|entity| {
                let Ok(threat_transform) = threat_q.get(*entity) else {
                    return None;
                };
                let linear_distance = threat_transform.translation.truncate() - **pos;
                let rads_to_east = linear_distance.y.atan2(linear_distance.x);
                let relative_angle = rads_to_east - rot.as_radians();
                let threat = Threat {
                    distance: linear_distance.length(),
                    radians: relative_angle,
                };
                Some(threat)
            })
            .collect();
        if threats.is_empty() {
            continue;
        }
        let event = ThreatEvent {
            sentry: **sentry,
            threats,
        };
        reporter.send(event);
    }
}

impl DeathCry for Sentry {
    fn cry(&self, assets: &MyAssets) -> AudioBundle {
        let sound = AudioBundle {
            source: assets.crack.clone(),
            settings: PlaybackSettings::DESPAWN,
        };
        sound
    }
}

impl Stage for Sentry {
    fn stage(self, assets: &Res<crate::assets::MyAssets>, transform: Transform) -> impl Bundle {
        let texture = assets.turret.clone();
        let model2d = SpriteBundle {
            transform,
            texture,
            sprite: Sprite {
                custom_size: Some(Vec2 { x: 5.0, y: 5.0 }),
                ..default()
            },
            ..default()
        };
        (
            Sentry,
            Name::new("Sentry"),
            model2d,
            RigidBody::Dynamic,
            ColliderDensity(6.),
            Collider::circle(2.5),
            CollisionDamage(1),
            Health { life: 50 },
        )
    }
}
