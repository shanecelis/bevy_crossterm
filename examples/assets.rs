use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use bevy_asset::LoadedUntypedAsset;
use std::default::Default;
use std::time::Duration;

#[derive(Clone, States, Default, Eq, PartialEq, Hash, Debug)]
enum GameState {
    #[default]
    Loading,
    Running,
}

// PRO TIP: _technically_ since Sprite's are just created using strings, an easier way to load them from an external
// file is just:
//static TITLE_TEXT: &str = include_str!("assets/demo/title.txt");
// then just:
//sprites.add(Sprite::new(TITLE_TEXT));
// and boom, you have yourself a sprite in the asset system.
// That's nice and easy - don't have to worry about async, don't need to distribute files alongside your exe.
// But then you can't take advantage of hot reloading, and plus it only works for sprites. StyleMaps have to go through
// the AssetServer if you want to load them from an external file.

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Assets example");

    App::new()
        // Add our window settings
        .insert_resource(settings)
        // Set some options in bevy to make our program a little less resource intensive - it's just a terminal game
        // no need to try and go nuts
        // .insert_resource(TaskPoolOptions::with_num_threads(1))
        // The Crossterm runner respects the schedulerunnersettings. No need to run as fast as humanly
        // possible - 20 fps should be more than enough for a scene that never changes
        .add_plugins(bevy_app::ScheduleRunnerPlugin::run_loop(
            Duration::from_millis(50),
        ))
        .add_state::<GameState>()
        .add_systems(OnEnter(GameState::Loading), default_settings)
        .add_systems(OnEnter(GameState::Loading), load_assets)
        .add_systems(Update, check_for_loaded)
        .add_systems(OnEnter(GameState::Running), create_entities)
        .add_plugins(DefaultPlugins)
        .add_plugins(CrosstermPlugin)
        .run();
}

static ASSETS: &[&str] = &["demo/title.txt", "demo/title.stylemap"];

#[derive(Resource)]
struct CrosstermAssets(Vec<Handle<LoadedUntypedAsset>>);

fn default_settings(mut cursor: ResMut<Cursor>) {
    cursor.hidden = true;
}

// This is a simple system that loads assets from the filesystem
fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the assets we want
    let mut handles = Vec::new();
    for asset in ASSETS {
        handles.push(asset_server.load_untyped(*asset));
    }

    commands.insert_resource(CrosstermAssets(handles));
}

// This function exists solely because bevy's asset loading is async.
// We need to wait until all assets are loaded before we do anything with them.
fn check_for_loaded(
    asset_server: Res<AssetServer>,
    handles: Res<CrosstermAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut all_done = true;
    for handle in handles.0.iter() {
        let data = asset_server.load_state(handle);

        match data {
            bevy::asset::LoadState::NotLoaded | bevy::asset::LoadState::Loading => {
                all_done = false;
                break;
            }
            bevy::asset::LoadState::Loaded => {}
            bevy::asset::LoadState::Failed => {
                panic!("This is an example and should not fail")
            }
        }
    }

    if all_done {
        next_state.set(GameState::Running);
    }
}

// Now that we're sure the assets are loaded, spawn a new sprite into the world
fn create_entities(
    mut commands: Commands,
    window: Query<&CrosstermWindow>,
    asset_server: Res<AssetServer>,
    sprites: Res<Assets<Sprite>>,
) {
    // I want to center the title, so i needed to wait until it was loaded before I could actually access
    // the underlying data to see how wide the sprite is and do the math
    let title_handle = asset_server.get_handle("demo/title.txt").unwrap();
    let title_sprite = sprites
        .get(&title_handle)
        .expect("We waited for asset loading");

    let window = window.single();
    let center_x = window.x_center() as i32 - title_sprite.x_center() as i32;
    let center_y = window.y_center() as i32 - title_sprite.y_center() as i32;

    commands.spawn(SpriteBundle {
        sprite: title_handle.clone(),
        position: Position::with_xy(center_x, center_y),
        stylemap: asset_server.get_handle("demo/title.stylemap").unwrap(),
        ..Default::default()
    });
}
