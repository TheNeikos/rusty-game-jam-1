use bevy::prelude::*;
use bevy_simple_tilemap::{prelude::TileMapBundle, Tile, TileMap};

use crate::{ldtk, ldtk_map::LdtkMap, movement::LevelBounds, GameAssets, MainLdtk, GRID_SIZE};

#[derive(Debug, Default)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_map).add_system(update_map);
    }
}

pub struct CurrentMap(pub Handle<MainLdtk>);

fn load_map(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.insert_resource(CurrentMap(game_assets.ldtk_map_handle.clone()));

    commands
        .spawn_bundle(TileMapBundle {
            texture_atlas: game_assets.texture_atlas_handle.clone(),
            ..Default::default()
        })
        .insert(CurrentLevel("Level_0".to_string()));
}

pub struct CurrentLevel(pub String);

fn update_map(
    mut commands: Commands,
    mut ldtk_map_asset_events: EventReader<AssetEvent<MainLdtk>>,
    ldtk_assets: Res<Assets<MainLdtk>>,
    current_map: Res<CurrentMap>,
    mut ldtk_map_query: Query<(&mut TileMap, &mut Transform, &CurrentLevel)>,
) {
    let mut to_be_updated = vec![];

    for event in ldtk_map_asset_events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                to_be_updated.push(handle.clone());
            }
            AssetEvent::Removed { handle: _ } => (),
        }
    }

    let ldtk_handle = &current_map.0;

    for handle in to_be_updated {
        for (mut map, mut transform, current_level) in ldtk_map_query.single_mut() {
            if &handle != ldtk_handle {
                continue;
            }

            let ldtk = if let Some(LdtkMap { ldtk }) = ldtk_assets.get(ldtk_handle) {
                ldtk
            } else {
                continue;
            };

            map.clear();

            let level = ldtk
                .levels
                .iter()
                .find(|level| level.identifier == current_level.0)
                .unwrap();

            transform.translation = level.world_position_px.extend(0).as_f32();

            info!("Map position: {}", level.world_position_px);

            add_layer(&level.layers.background, 0, &mut map);
            add_layer(&level.layers.background_details, 1, &mut map);
            add_layer(&level.layers.foreground, 2, &mut map);

            commands.insert_resource(LevelBounds(level.dimensions_px / GRID_SIZE));
        }
    }
}

fn add_layer(
    layer: &bevy_spicy_ldtk::Layer<ldtk::ProjectEntities>,
    height: i32,
    map: &mut TileMap,
) {
    match &layer.special {
        bevy_spicy_ldtk::SpecialValues::IntGrid {
            auto_layer: tiles, ..
        }
        | bevy_spicy_ldtk::SpecialValues::Tiles { tiles, .. }
        | bevy_spicy_ldtk::SpecialValues::AutoLayer { auto_layer: tiles } => {
            for tile in tiles {
                let pos = tile.position_px / layer.grid_size as i32;
                // info!("Spawning at {}", pos);
                let sprite_index = tile.id as _;
                map.set_tile(
                    pos.extend(height),
                    Some(Tile {
                        sprite_index,
                        ..Default::default()
                    }),
                );
            }
        }
        bevy_spicy_ldtk::SpecialValues::Entities(_) => (),
    }
}
