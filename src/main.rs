mod camera;
mod ldtk_map;
mod map;
mod markers;
mod movement;
mod objects;
mod player;
mod ui;

use std::{path::PathBuf, time::Duration};

use benimator::SpriteSheetAnimation;
use bevy::{prelude::*, render::pass::ClearColor};
use bevy_spicy_ldtk::ldtk;
use camera::ViewCamera;
use ldtk_map::LdtkMap;

ldtk!(pub ldtk, "assets/levels/world.ldtk");

pub type MainLdtk = LdtkMap<ldtk::Project>;

pub const GRID_SIZE: i32 = 18;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(245, 255, 232)))
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_simple_tilemap::prelude::SimpleTileMapPlugin)
        .add_plugin(ldtk_map::LdtkPlugin::<ldtk::Project>::default())
        .add_plugin(benimator::AnimationPlugin)
        .add_plugin(markers::MarkerPlugin)
        .add_plugin(map::MapPlugin)
        .add_plugin(movement::MovementPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(objects::ObjectPlugin)
        .add_plugin(ui::UiPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_game_assets)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_game_counters)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let scale = 1. / 4.;
    let mut cam = OrthographicCameraBundle::new_2d();
    cam.transform.scale = Vec3::splat(scale);
    cam.orthographic_projection.far = 1000. / scale;
    commands.spawn_bundle(cam).insert(ViewCamera);
}

pub struct CoinCount(u32);

pub struct GameAssets {
    texture_atlas_handle: Handle<TextureAtlas>,

    ldtk_map_handle: Handle<MainLdtk>,

    coin_texture_handle: Handle<Texture>,
    coin_animation_handle: Handle<SpriteSheetAnimation>,

    coin_pickup_handle: Handle<TextureAtlas>,
    coin_pickup_animation: Handle<SpriteSheetAnimation>,

    text_font_handle: Handle<Font>,
}

fn setup_game_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
    mut animation_assets: ResMut<Assets<SpriteSheetAnimation>>,
) {
    asset_server.watch_for_changes().unwrap();

    let path = PathBuf::from(ldtk::FILEPATH);
    let ldtk_map_handle: Handle<MainLdtk> = asset_server.load(path.strip_prefix("assets").unwrap());

    let texture_handle = asset_server.load("art/tiles.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::splat(GRID_SIZE as f32),
        20,
        9,
        Vec2::splat(2.),
    );
    let texture_atlas_handle = texture_atlas_assets.add(texture_atlas);

    let texture_handle = asset_server.load("art/coin_pickup.png");
    let coin_pickup_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::splat(100.),
        4,
        4,
        Vec2::splat(0.),
    );
    let coin_pickup_handle = texture_atlas_assets.add(coin_pickup_atlas);

    let coin_pickup_animation = animation_assets
        .add(SpriteSheetAnimation::from_range(0..=16, Duration::from_millis(16)).once());

    let text_font_handle = asset_server.load("fonts/Chewy-Regular.ttf");

    let coin_texture_handle = asset_server.load("art/coin.png");

    let coin_animation_handle = animation_assets.add(SpriteSheetAnimation::from_range(
        151..=152,
        Duration::from_millis(750),
    ));

    commands.insert_resource(GameAssets {
        texture_atlas_handle,
        ldtk_map_handle,
        coin_texture_handle,
        coin_animation_handle,
        coin_pickup_handle,
        coin_pickup_animation,
        text_font_handle,
    })
}

fn setup_game_counters(mut commands: Commands) {
    commands.insert_resource(CoinCount(0));
}
