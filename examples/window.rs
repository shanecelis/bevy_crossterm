use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use bevy::log::LogPlugin;
use std::default::Default;

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Window example");

    // We set some options to make our program a little less resource intensive - it's just a terminal game
    // no need to try and go nuts
    // 1. Use only 1 thread
    // 2. Limit FPS: 20 fps should be more than enough for a scene that never changes

    App::new()
        // Add our window settings
        .insert_resource(settings)
        // Limit FPS: The Crossterm runner respects the schedulerunnersettings
        .add_plugins(bevy_app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(50),
        ))
        // Add the DefaultPlugins before the CrosstermPlugin. The crossterm plugin needs bevy's asset server, and if it's
        // not available you'll trigger an assert
        .add_plugins(
            DefaultPlugins
                // Limit threads
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(1),
                })
                // Disable logging, which would otherwise appear randomly.
                .set(LogPlugin {
                    filter: "off".into(),
                    level: bevy::log::Level::ERROR,
                }),
        )
        .add_plugins(CrosstermPlugin)
        .add_systems(Startup, startup_system)
        .run();
}

fn startup_system(
    mut commands: Commands,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    // Create our resources - two sprites and the default colors that we'll use for both
    let text = sprites.add(Sprite::new("This is an example which creates a crossterm window,\nsets a title, and displays some text."));
    let ctrlc = sprites.add(Sprite::new("Press Control-C to quit"));
    let color = stylemaps.add(StyleMap::default());

    // Spawn two sprites into the world
    commands.spawn(SpriteBundle {
        sprite: text,
        stylemap: color.clone(),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        sprite: ctrlc,
        position: Position { x: 0, y: 3, z: 0 },
        stylemap: color,
        ..Default::default()
    });
}
