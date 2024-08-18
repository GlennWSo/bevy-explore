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
    let green_tint = Srgba::new(0., 0.5, 0., 0.1);
    let red_tint = Srgba::new(0.5, 0., 0., 0.1);
    commands
        .ui_builder(UiRoot)
        .row(|root_row| {
            root_row.column(|root_col| {
                root_col
                    .style()
                    .background_color(red_tint.into())
                    .height(Val::Percent(100.))
                    .justify_self(JustifySelf::Center)
                    .justify_content(JustifyContent::Center);
                root_col.row(|cell| {
                    cell.spawn(forward_btn);
                    cell.style()
                        .background_color(green_tint.into())
                        // .width(Val::Percent(25.))
                        .height(Val::Percent(25.));
                });
            });
        })
        .style()
        .background_color(green_tint.into())
        .height(Val::Percent(100.))
        // .justify_items(JustifyItems::Center);
        .justify_content(JustifyContent::Center);
}
