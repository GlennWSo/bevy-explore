use bevy::prelude::*;

use crate::{health::Health, state::GameState};

/// used for marking entity to not be faraway removed
#[derive(Component)]
pub struct Keep;

const MAX_DISTANCE: f32 = 100.0;
pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, remove_far)
            .add_systems(OnEnter(GameState::GameOver), remove_all);
    }
}

fn remove_all(mut cmds: Commands, q: Query<Entity, With<Health>>) {
    for ent in q.iter() {
        cmds.entity(ent).despawn_recursive();
    }
}

fn remove_far(
    mut cmds: Commands,
    q: Query<(Entity, &Transform), Without<Keep>>,
    player_q: Query<(&Transform), With<Keep>>,
) {
    let Ok(player) = player_q.get_single() else {
        return;
    };
    for (ent, trans) in q.iter() {
        let distance = trans.translation.distance(player.translation);
        if distance > MAX_DISTANCE {
            cmds.entity(ent).despawn_recursive();
        }
    }
}
