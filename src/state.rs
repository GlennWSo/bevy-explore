use bevy::prelude::*;

#[derive(States, Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Play,
    Paused,
    GameOver,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Update, restart_game.run_if(in_state(GameState::GameOver)))
            .add_systems(Update, toggle_game_state);
    }
}

fn toggle_game_state(
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if key_input.just_pressed(KeyCode::Escape) {
        println!("key pressed");
        match dbg!(state.get()) {
            GameState::Play => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Play),
            GameState::GameOver => todo!(),
        }
    }
}

fn restart_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Play);
}
