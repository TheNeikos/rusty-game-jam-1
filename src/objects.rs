use std::time::Duration;

use benimator::Play;
use bevy::prelude::*;

use crate::{
    ldtk_map::LdtkMap,
    map::CurrentLevel,
    markers::Markers,
    movement::{Position, Speed},
    player::Player,
    CoinCount, GameAssets, MainLdtk, GRID_SIZE,
};

pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_objects)
            .add_system(interact_spring_with_player)
            .add_system(update_spring_tile)
            .add_system(interact_coin_with_player)
            .add_system(update_coin_pickup_tile);
    }
}

#[derive(Debug, Default)]
pub struct Spring {
    force: f32,
}

#[derive(Default, Bundle)]
pub struct SpringBundle {
    pub spring: Spring,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub position: Position,
    pub markers: Markers,
}

fn update_spring_tile(
    spring_query: Query<(&Markers, &Children), (Changed<Markers>, With<Spring>)>,
    mut atlas_sprite_query: Query<&mut TextureAtlasSprite>,
) {
    for (markers, children) in spring_query.iter() {
        let id = if markers.contains::<Sprung>() {
            108
        } else {
            107
        };

        for child in children.as_ref() {
            if let Ok(mut atlas_sprite) = atlas_sprite_query.get_mut(*child) {
                atlas_sprite.index = id;
            }
        }
    }
}
struct Sprung;

fn interact_spring_with_player(
    mut player_query: Query<(&mut Speed, &Position), (With<Player>, Without<Spring>)>,
    mut spring_query: Query<(&Position, &Spring, &mut Markers), With<Spring>>,
) {
    for (mut speed, position) in player_query.iter_mut() {
        for (spring_pos, spring, mut spring_markers) in spring_query.iter_mut() {
            if spring_pos.cell == position.cell && !spring_markers.contains::<Sprung>() {
                speed.speed.y += spring.force;
                spring_markers.add_marker_for::<Sprung>(Duration::from_millis(1000));
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Coin;

#[derive(Default, Bundle)]
pub struct CoinBundle {
    pub spring: Coin,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub position: Position,
    pub markers: Markers,
}

#[derive(Debug, Default)]
pub struct CoinPickup;

#[derive(Default, Bundle)]
pub struct CoinPickupBundle {
    pub coin_pickup: CoinPickup,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub position: Position,
}

fn update_coin_pickup_tile(mut commands: Commands, finished_coins: RemovedComponents<Play>) {
    for coin in finished_coins.iter() {
        commands.entity(coin).despawn_recursive();
    }
}

fn interact_coin_with_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut coin_count: ResMut<CoinCount>,
    mut player_query: Query<&Position, With<Player>>,
    mut coin_query: Query<(Entity, &Position), With<Coin>>,
) {
    for player_position in player_query.iter_mut() {
        for (entity, coin_pos) in coin_query.iter_mut() {
            if coin_pos.cell == player_position.cell {
                commands.entity(entity).despawn_recursive();
                commands
                    .spawn_bundle(CoinPickupBundle {
                        position: *coin_pos,
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        let transform = Transform {
                            translation: Vec3::new(0.5 * GRID_SIZE as f32, 4., 0.),
                            ..Default::default()
                        };

                        parent
                            .spawn_bundle(SpriteSheetBundle {
                                texture_atlas: game_assets.coin_pickup_handle.clone(),
                                sprite: TextureAtlasSprite {
                                    index: 0,
                                    ..Default::default()
                                },
                                transform,
                                ..Default::default()
                            })
                            .insert(game_assets.coin_pickup_animation.clone())
                            .insert(Play);
                    });

                coin_count.0 += 1;
            }
        }
    }
}

fn spawn_objects(
    mut commands: Commands,
    mut ldtk_map_asset_events: EventReader<AssetEvent<MainLdtk>>,
    ldtk_assets: Res<Assets<MainLdtk>>,
    game_assets: Res<GameAssets>,
    old_objects_query: Query<Entity, Or<(With<Spring>, With<Coin>)>>,
    levels_query: Query<&CurrentLevel>,
) {
    let last_event = ldtk_map_asset_events.iter().last();

    let event = if let Some(event) = last_event {
        event
    } else {
        return;
    };

    for entity in old_objects_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    match event {
        AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
            let ldtk = if let Some(LdtkMap { ldtk }) = ldtk_assets.get(handle) {
                ldtk
            } else {
                return;
            };

            for current_level in levels_query.iter() {
                let level = if let Some(level) = ldtk
                    .levels
                    .iter()
                    .find(|level| level.identifier == current_level.0)
                {
                    level
                } else {
                    error!("Could not find level: {}", current_level.0);
                    continue;
                };

                match &level.layers.entities.special {
                    bevy_spicy_ldtk::SpecialValues::Entities(entities) => {
                        for spring in &entities.all_spring {
                            let pos = spring.position_cell.as_f32()
                                + level.world_position_px.as_f32() / GRID_SIZE as f32
                                + spring.pivot * spring.dimensions_px.as_f32() / GRID_SIZE as f32;

                            info!(
                                "Spawning spring at: {} from {}",
                                pos,
                                spring.position_px.as_f32()
                            );

                            commands
                                .spawn_bundle(SpringBundle {
                                    spring: Spring {
                                        force: spring.fields.force as f32,
                                    },
                                    position: Position::from(pos),
                                    transform: Transform::from_xyz(0., 0., 1.5),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn_bundle(SpriteSheetBundle {
                                        texture_atlas: game_assets.texture_atlas_handle.clone(),
                                        sprite: TextureAtlasSprite {
                                            index: 107,
                                            ..Default::default()
                                        },
                                        transform: Transform::from_translation(Vec3::new(
                                            0.,
                                            (spring.pivot.y + 0.5) * spring.dimensions_px.y as f32,
                                            0.,
                                        )),
                                        ..Default::default()
                                    });
                                });
                        }

                        for coin in &entities.all_coin {
                            let pos = coin.position_cell.as_f32()
                                + level.world_position_px.as_f32() / GRID_SIZE as f32
                                + coin.pivot * coin.dimensions_px.as_f32() / GRID_SIZE as f32;

                            info!(
                                "Spawning spring at: {} from {}",
                                pos,
                                coin.position_px.as_f32()
                            );

                            commands
                                .spawn_bundle(CoinBundle {
                                    spring: Coin,
                                    position: Position::from(pos),
                                    transform: Transform::from_xyz(0., 0., 1.5),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn_bundle(SpriteSheetBundle {
                                            texture_atlas: game_assets.texture_atlas_handle.clone(),
                                            sprite: TextureAtlasSprite {
                                                index: 151,
                                                ..Default::default()
                                            },
                                            transform: Transform::from_translation(Vec3::new(
                                                0.,
                                                (coin.pivot.y + 0.5) * coin.dimensions_px.y as f32,
                                                0.,
                                            )),
                                            ..Default::default()
                                        })
                                        .insert(game_assets.coin_animation_handle.clone())
                                        .insert(Play);
                                });
                        }
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
}
