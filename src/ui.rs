use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

use sickle_ui::{prelude::*, SickleUiPlugin};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, synthetic_keyboard);
    }
}
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct ForwardBtn;

fn synthetic_keyboard(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ForwardBtn>)>,
    mut writer: EventWriter<KeyboardInput>,
    window_q: Query<Entity, With<Window>>,
) {
    let window = window_q.single();
    for btn in interaction_q.iter() {
        let state = match *btn {
            Interaction::Pressed => ButtonState::Pressed,
            _ => ButtonState::Released,
        };
        let key_event = KeyboardInput {
            key_code: KeyCode::ArrowUp,
            logical_key: bevy::input::keyboard::Key::ArrowUp,
            state,
            window,
        };
        println!("{:?}", btn);
        println!("key event:{:?}", key_event);
        writer.send(key_event);
    }
}

fn setup(mut commands: Commands) {
    // ui camera
    commands
        .ui_builder(UiRoot)
        .column(|column| {
            let btn = ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(5.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                border_radius: BorderRadius::MAX,
                // background_color: NORMAL_BUTTON.into(),
                ..default()
            };
            let forward_btn = (ForwardBtn, btn.clone());
            column.spawn(forward_btn);
        })
        .style()
        .justify_content(JustifyContent::Center);
}
