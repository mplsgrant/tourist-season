use bevy::{color::palettes::basic::*, prelude::*};

use crate::constants::PopupBase;

pub struct Popup;

impl Plugin for Popup {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, button_system);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let base_node = (
        Node {
            margin: UiRect::all(Val::Px(25.0)),
            align_self: AlignSelf::Stretch,
            justify_self: JustifySelf::Stretch,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
    );

    let root = commands.spawn(base_node).id();

    // labels for the different border edges
    let building_labels = ["Bottom right"];

    // all the different combinations of border edges
    // these correspond to the labels above
    let lower_right = asset_server.load("tiles-test/tile_0038.png");
    let building_images = [ImageNode::new(lower_right)];

    for (label, building_image) in building_labels.into_iter().zip(building_images) {
        let building_node = commands
            .spawn((
                building_image,
                Button,
                Outline {
                    width: Val::Px(2.),
                    offset: Val::Px(0.),
                    color: Color::WHITE,
                },
            ))
            .id();
        let label_node = commands
            .spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
            ))
            .id();
        let container = commands
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Percent(12.0),
                ..default()
            })
            .add_children(&[building_node, label_node])
            .id();
        commands.entity(root).add_child(container);
    }

    let popup_label = commands
        .spawn((
            Node {
                margin: UiRect {
                    left: Val::Px(25.0),
                    right: Val::Px(25.0),
                    top: Val::Px(25.0),
                    bottom: Val::Px(0.0),
                },
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new("Buildings"),
                TextFont {
                    font_size: 20.0,
                    ..Default::default()
                },
            ));
        })
        .id();

    commands
        .spawn((
            Node {
                margin: UiRect::all(Val::Px(25.0)),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Stretch,
                justify_self: JustifySelf::Stretch,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            PopupBase,
        ))
        .add_child(popup_label)
        .add_child(root);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut Outline),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut outline) in &mut interaction_query {
        info!("HELLO");
        match *interaction {
            Interaction::Pressed => {
                outline.color = RED.into();
            }
            Interaction::Hovered => {
                outline.color = Color::WHITE;
            }
            Interaction::None => {
                outline.color = Color::BLACK;
            }
        }
    }
}
