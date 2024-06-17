use bevy::prelude::*;

use crate::{
    astroids::Astroid,
    health::Health,
    ship::{Player, SpaceShip},
    state::GameState,
};

/// used for marking entity to not be faraway removed
#[derive(Component)]
pub struct Keep;

const MAX_DISTANCE: f32 = 1000.0;
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

pub fn remove_far<T: Component>(
    mut cmds: Commands,
    q: Query<(Entity, &Transform), With<T>>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(player) = player_q.get_single() else {
        return;
    };
    for (ent, trans) in q.iter() {
        let distance = trans.translation.distance(player.translation);
        if distance > MAX_DISTANCE {
            info!("despawning {:?}", ent);
            cmds.entity(ent).despawn_recursive();
        }
    }
}
