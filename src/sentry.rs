use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    assets::MyAssets,
    collide_dmg::CollisionDamage,
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
        app.add_systems(Update, detect_threat);
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
    cmds.spawn(Sentry.stage(&assets, transform))
        .with_children(|parrent| {
            parrent.spawn((detector, collider, Sensor));
        });
}

fn detect_threat(
    sensor_q: Query<(&CollidingEntities, &Parent), With<Detector<SpaceShip>>>,
    threat_q: Query<&Transform, With<SpaceShip>>,
    parrent_q: Query<&Transform>,
) {
    sensor_q.par_iter().for_each(|(threats, parrent)| {
        for &threat in threats.iter() {
            let Ok(transform) = threat_q.get(threat) else {
                continue;
            };
            let Ok(origin) = parrent_q.get(**parrent) else {
                continue;
            };
            let linear_distance = transform.translation - origin.translation;
            println!("Threat detected at {:?}", linear_distance);
        }
    });
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
