use bevy::prelude::*;

use crate::{health::Health, state::GameState, Player};

/// used for marking entity to not be faraway removed
#[derive(Component)]
pub struct Keep;

pub const MAX_DISTANCE: f32 = 1000.0;
pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), remove_all);
    }
}

fn remove_all(mut cmds: Commands, q: Query<Entity, With<Health>>) {
    for ent in q.iter() {
        cmds.entity(ent).despawn_recursive();
    }
}

pub fn despawn_far<T: Component, const DIST: i32>(
    mut cmds: Commands,
    q: Query<(Entity, &Transform), With<T>>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(player) = player_q.get_single() else {
        return;
    };
    for (ent, trans) in q.iter() {
        let distance = trans.translation.distance(player.translation);
        if distance > DIST as f32 {
            cmds.entity(ent).despawn_recursive();
        }
    }
}
