use bevy::prelude::*;

use crate::{
    camera::CameraFollow,
    ldtk_map::LdtkMap,
    map::CurrentLevel,
    markers::Markers,
    movement::{Gravity, MovementStages, OnGround, Position, Speed},
    GameAssets, MainLdtk, GRID_SIZE,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player)
            .add_system(check_player_intent)
            .add_system_to_stage(MovementStages::Movement, move_player);
    }
}

#[derive(Debug, Default)]
pub struct Player;

#[derive(Default, Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub transform: Transform,
    pub camera_follow: CameraFollow,
    pub global_transform: GlobalTransform,
    pub speed: Speed,
    pub position: Position,
    pub gravity: Gravity,
    pub intent: PlayerIntent,
    pub markers: Markers,
}

fn spawn_player(
    mut commands: Commands,
    mut ldtk_map_asset_events: EventReader<AssetEvent<MainLdtk>>,
    ldtk_assets: Res<Assets<MainLdtk>>,
    game_assets: Res<GameAssets>,
    player_query: Query<Entity, With<Player>>,
    levels_query: Query<&CurrentLevel>,
) {
    let last_event = ldtk_map_asset_events.iter().last();

    let event = if let Some(event) = last_event {
        event
    } else {
        return;
    };

    for entity in player_query.iter() {
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
                        let player = if let Some(player) = entities.all_player.first() {
                            player
                        } else {
                            return;
                        };

                        let pos = player.position_cell.as_f32()
                            + level.world_position_px.as_f32() / GRID_SIZE as f32
                            + player.pivot * player.dimensions_px.as_f32() / GRID_SIZE as f32;

                        info!(
                            "Spawning player at: {} ({:?}) ({} + {})",
                            pos,
                            Position::from(pos),
                            player.position_cell.as_f32(),
                            level.world_position_px.as_f32()
                        );

                        commands
                            .spawn_bundle(PlayerBundle {
                                position: Position::from(pos),
                                transform: Transform::from_translation(
                                    (pos * crate::GRID_SIZE as f32).round().extend(1.5),
                                ),
                                gravity: Gravity(Vec2::new(0., -0.02)),
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(SpriteSheetBundle {
                                    texture_atlas: game_assets.texture_atlas_handle.clone(),
                                    sprite: TextureAtlasSprite {
                                        index: 145,
                                        ..Default::default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.,
                                        (player.pivot.y + 0.5) * player.dimensions_px.y as f32,
                                        0.,
                                    ),
                                    ..Default::default()
                                });

                                // let transform = Transform {
                                //     scale: Vec3::splat(0.2),
                                //     translation: Vec3::new(0., 0., 10.),
                                //     ..Default::default()
                                // };

                                // parent.spawn_bundle(SpriteSheetBundle {
                                //     texture_atlas: game_assets.texture_atlas_handle.clone(),
                                //     sprite: TextureAtlasSprite {
                                //         index: 156,
                                //         color: Color::rgba(1., 1., 1., 0.2),
                                //         ..Default::default()
                                //     },
                                //     visible: Visible {
                                //         is_transparent: true,
                                //         ..Default::default()
                                //     },
                                //     transform,
                                //     ..Default::default()
                                // });
                            });
                    }
                    _ => (),
                }
            }
        }
        AssetEvent::Removed { handle: _ } => (),
    }
}

#[derive(Debug)]
pub enum PlayerDirection {
    Left,
    Right,
}

#[derive(Debug, Default)]
pub struct PlayerIntent {
    pub direction: Option<PlayerDirection>,
    pub jump: bool,
}

impl PlayerIntent {
    fn reset(&mut self) {
        *self = Default::default();
    }
}

fn check_player_intent(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<&mut PlayerIntent, With<Player>>,
) {
    for mut player_intent in player_query.iter_mut() {
        if keyboard.pressed(KeyCode::A) {
            player_intent.direction = Some(PlayerDirection::Left);
        }
        if keyboard.pressed(KeyCode::D) {
            player_intent.direction = Some(PlayerDirection::Right);
        }

        if keyboard.just_pressed(KeyCode::Space) {
            player_intent.jump = true;
        }
    }
}

fn move_player(mut player_query: Query<(&mut Speed, &mut PlayerIntent, &Markers), With<Player>>) {
    for (mut speed, mut player_intent, markers) in player_query.iter_mut() {
        match &player_intent.direction {
            Some(direction) => match direction {
                PlayerDirection::Left => speed.speed.x -= 0.06,
                PlayerDirection::Right => speed.speed.x += 0.06,
            },
            None => (),
        }

        if player_intent.jump && markers.contains::<OnGround>() {
            speed.speed.y += 0.7;
        }

        player_intent.reset();
    }
}
