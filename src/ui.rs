use bevy::prelude::*;

use crate::{CoinCount, GameAssets};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui).add_system(update_coins);
    }
}

struct CoinCounter;

fn update_coins(coin_stats: Res<CoinCount>, mut text_query: Query<&mut Text, With<CoinCounter>>) {
    if coin_stats.is_changed() {
        for mut text in text_query.iter_mut() {
            text.sections[0].value = coin_stats.0.to_string();
        }
    }
}

fn setup_ui(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    let default_text_style = TextStyle {
        font: game_assets.text_font_handle.clone(),
        font_size: 50.,
        color: Color::BLACK,
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(25.),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Px(75.),
                },
                padding: Rect {
                    left: Val::Px(50.),
                    right: Val::Px(50.),
                    ..Default::default()
                },
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(ColorMaterial::color(Color::rgba(0., 0., 0., 0.))),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        padding: Rect::all(Val::Px(20.)),
                        ..Default::default()
                    },
                    material: materials.add(ColorMaterial::color(Color::rgb_u8(178, 200, 152))),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(50.), Val::Auto),
                            ..Default::default()
                        },
                        material: materials
                            .add(ColorMaterial::texture(game_assets.coin_texture_handle.clone())),
                        ..Default::default()
                    });
                    parent
                        .spawn_bundle(TextBundle {
                            style: Style {
                                margin: Rect {
                                    left: Val::Px(10.),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text::with_section(
                                "0",
                                TextStyle {
                                    color: Color::rgb_u8(102, 57, 49),
                                    ..default_text_style
                                },
                                TextAlignment::default(),
                            ),
                            ..Default::default()
                        })
                        .insert(CoinCounter);
                });
        });
}
