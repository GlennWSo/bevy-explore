use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::{guns::Plasma, health::Health};

pub struct CollideDamagePlugin;

impl Plugin for CollideDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, contact_damage::<(), ()>);
        // app.add_systems(Update, contact_damage::<With<Plasma>, Without<Plasma>>);
        // app.add_systems(Update, contact_damage::<Without<Plasma>, ()>);
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct CollisionDamage(pub i32);

fn contact_damage<DmgFilter, HealthFilter>(
    collision_event_reader: EventReader<CollisionStarted>,
    health_q: Query<&mut Health, HealthFilter>,
    damage_q: Query<&CollisionDamage, DmgFilter>,
) where
    DmgFilter: QueryFilter,
    HealthFilter: QueryFilter,
{
    {
        let mut collision_event_reader = collision_event_reader;
        let mut health_q = health_q;
        for CollisionStarted(ent1, ent2) in collision_event_reader.read() {
            let ent1 = *ent1;
            let ent2 = *ent2;
            if let (Ok(dmg), Ok(mut health)) = (damage_q.get(ent1), health_q.get_mut(ent2)) {
                **health -= **dmg
            }
            if let (Ok(dmg), Ok(mut health)) = (damage_q.get(ent2), health_q.get_mut(ent1)) {
                **health -= **dmg
            }
        }
    };
}
