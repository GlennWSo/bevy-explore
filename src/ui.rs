use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};

use sickle_ui::{input_extension::KeyCodeToStringExt, prelude::*, SickleUiPlugin};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, synthetic_keyboard);
    }
}
// const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const GREEN_TINT: Srgba = Srgba::new(0., 0.5, 0., 0.1);
const RED_TINT: Srgba = Srgba::new(0.5, 0., 0., 0.1);

#[derive(Component)]
struct ForwardBtn;

#[derive(Component, Clone)]
struct SyntheticKey {
    key_code: KeyCode,
    logical_key: Key,
}

fn synthetic_keyboard(
    interaction_q: Query<(&Interaction, &SyntheticKey), Changed<Interaction>>,
    mut writer: EventWriter<KeyboardInput>,
    window_q: Query<Entity, With<Window>>,
) {
    let window = window_q.single();
    for (
        btn,
        SyntheticKey {
            key_code,
            logical_key,
        },
    ) in interaction_q.iter()
    {
        let state = match *btn {
            Interaction::Pressed => ButtonState::Pressed,
            _ => ButtonState::Released,
        };
        let key_event = KeyboardInput {
            key_code: *key_code,
            logical_key: logical_key.clone(),
            state,
            window,
        };
        println!("{:?}", btn);
        println!("key event:{:?}", key_event);
        writer.send(key_event);
    }
}

fn setup(mut commands: Commands) {
    commands
        .ui_builder(UiRoot)
        .row(|root_row| {
            root_row.column(|left_col| {
                left_col
                    .style()
                    .min_width(Val::Percent(3.))
                    .background_color(GREEN_TINT.into())
                    .height(Val::Percent(100.))
                    .justify_content(JustifyContent::Center);
                left_col.spawn(button());
            });
            root_row.column(|right_col| {
                right_col
                    .style()
                    .min_width(Val::Percent(3.))
                    .background_color(GREEN_TINT.into())
                    .height(Val::Percent(100.))
                    .justify_content(JustifyContent::Center);
                right_col.row(|top_row| {
                    top_row
                        .spawn((
                            button(),
                            SyntheticKey {
                                key_code: KeyCode::ArrowUp,
                                logical_key: Key::ArrowUp,
                            },
                        ))
                        .spawn(TextBundle::from_section("^^", btn_txt_style()));
                    top_row.style().justify_content(JustifyContent::Center);
                });
                right_col.row(|mid_row| {
                    mid_row
                        .spawn((
                            button(),
                            SyntheticKey {
                                key_code: KeyCode::ArrowLeft,
                                logical_key: Key::ArrowLeft,
                            },
                        ))
                        .spawn(TextBundle::from_section("<<", btn_txt_style()));
                    mid_row
                        .spawn((
                            button(),
                            SyntheticKey {
                                key_code: KeyCode::ArrowRight,
                                logical_key: Key::ArrowRight,
                            },
                        ))
                        .spawn(TextBundle::from_section(">>", btn_txt_style()));
                    mid_row.style().justify_content(JustifyContent::Center);
                });
                right_col.row(|inner_row| {
                    inner_row
                        .spawn((
                            button(),
                            SyntheticKey {
                                key_code: KeyCode::ArrowDown,
                                logical_key: Key::ArrowDown,
                            },
                        ))
                        .spawn(TextBundle::from_section("VV", btn_txt_style()));
                    inner_row.style().justify_content(JustifyContent::Center);
                });
            });
        })
        .style()
        // .background_color(green_tint.into())
        .height(Val::Percent(100.))
        // .justify_items(JustifyItems::Center);
        .justify_content(JustifyContent::SpaceBetween);
}

fn btn_txt_style() -> TextStyle {
    let txt_style = TextStyle {
        font_size: 40.0,
        color: Color::srgb(0.9, 0.9, 0.9),
        ..default()
    };
    txt_style
}

fn button() -> ButtonBundle {
    let btn = ButtonBundle {
        style: Style {
            width: Val::Vw(12.0),
            height: Val::Vh(16.0),
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
    btn
}
