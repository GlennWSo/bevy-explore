use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    assets::MyAssets,
    collide_dmg::CollisionDamage,
    health::{cry_dead, DeathCry, Health},
    schedule::{InGameSet, InitStages},
    stage::Stage,
};

pub struct SentryPlugin;

impl Plugin for SentryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_dbg_sentry.in_set(InitStages::Spawn));
        app.add_systems(Update, cry_dead::<Sentry>.in_set(InGameSet::Spawn));
    }
}

fn init_dbg_sentry(mut cmds: Commands, assets: Res<MyAssets>) {
    let transform = Transform::from_xyz(0., 30., 0.);
    let bundle = Sentry.stage(&assets, transform);
    cmds.spawn(bundle);
}

#[derive(Component)]
struct Sentry;

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
            model2d,
            RigidBody::Dynamic,
            ColliderDensity(6.),
            Collider::circle(2.5),
            CollisionDamage(1),
            Health { life: 50 },
        )
    }
}
