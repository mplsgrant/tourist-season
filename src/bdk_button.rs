#![allow(clippy::type_complexity)]

use crate::{
    bdk_zone::get_segwit_challenge, constants::PopupBase, popup::PickedItem,
    tilemaptest::GameMapEvent,
};
use bevy::{color::palettes::basic::*, prelude::*};

pub struct BDKButton;

impl Plugin for BDKButton {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, button_system);
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub enum ButtonAction {
    Save,
    TogglePopup,
}

fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &ButtonAction,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut popup_q: Query<&mut Node, With<PopupBase>>,
    mut picked_q: Query<(Entity, &PickedItem)>,
    mut tilemap_e: EventWriter<GameMapEvent>,
) {
    for (interaction, mut color, mut border_color, children, button_action) in
        &mut interaction_query
    {
        match button_action {
            ButtonAction::Save => match *interaction {
                Interaction::Pressed => {
                    let _ = tilemap_e.write(GameMapEvent::Save);
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                    border_color.0 = Color::WHITE;
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                    border_color.0 = Color::BLACK;
                }
            },
            ButtonAction::TogglePopup => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                match *interaction {
                    Interaction::Pressed => {
                        // Despawn any PickedItem
                        for (entity, _) in picked_q.iter_mut() {
                            commands.entity(entity).despawn();
                            info!("Despawned: {entity:?}");
                        }

                        let z = get_segwit_challenge();
                        println!("z: {z:?}");
                        **text = "Press".to_string();
                        *color = PRESSED_BUTTON.into();
                        border_color.0 = RED.into();

                        // Toggle PopupBase
                        for mut node in &mut popup_q {
                            node.display = match node.display {
                                Display::None => Display::Flex,
                                _ => Display::None,
                            };
                            info!("Toggled PopupBase");
                        }
                    }
                    Interaction::Hovered => {
                        **text = "Hover".to_string();
                        *color = HOVERED_BUTTON.into();
                        border_color.0 = Color::WHITE;
                    }
                    Interaction::None => {
                        **text = "Button".to_string();
                        *color = NORMAL_BUTTON.into();
                        border_color.0 = Color::BLACK;
                    }
                }
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                column_gap: Val::Percent(1.0),
                ..default()
            },
            GlobalZIndex(4),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                    ButtonAction::TogglePopup,
                ))
                .with_child((
                    Text::new("Button"),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                    ZIndex(1),
                    ButtonAction::Save,
                ))
                .with_child((
                    Text::new("Save"),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
        });
}
