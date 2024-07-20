use bevy::{log::tracing_subscriber::fmt::time::Uptime, prelude::*};
use bevy_xpbd_2d::prelude::*;

use crate::{astroids::Astroid, collide::CollisionDamage, health::Health};

pub struct CollideDamagePlugin;

impl Plugin for CollideDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collision_damage);
    }
}

fn collision_damage(
    mut collision_event_reader: EventReader<CollisionStarted>,
    mut health_q: Query<&mut Health>,
    damage_q: Query<&CollisionDamage>,
) {
    for CollisionStarted(ent1, ent2) in collision_event_reader.read() {
        let ent1 = *ent1;
        let ent2 = *ent2;
        if let (Ok(dmg), Ok(mut health)) = (damage_q.get(ent1), health_q.get_mut(ent2)) {
            **health -= **dmg
        }
        if let (Ok(dmg), Ok(mut health)) = (damage_q.get(ent2), health_q.get_mut(ent1)) {
            **health -= **dmg
        }

        // println!(
        //     "Entities {:?} and {:?} are colliding",
        //     contacts.entity1, contacts.entity2,
        // );
    }
}