use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use bevy::log::LogPlugin;
use std::default::Default;

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Transparency example");

    App::new()
        .insert_resource(settings)
        .add_plugins(bevy_app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(50),
        ))
        .add_plugins(
            DefaultPlugins
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(1),
                })
                .set(LogPlugin {
                    filter: "off".into(),
                    level: bevy::log::Level::ERROR,
                }),
        )
        .add_plugins(CrosstermPlugin)
        .add_systems(Startup, startup_system)
        .run();
}

// 5x5 box of spaces
static BIG_BOX: &str = "         \n         \n         \n         \n         ";
static SMALL_BOX: &str = r##"@@@@@
@ @ @
@   @
@ @ @
@@@@@"##;

fn startup_system(
    mut commands: Commands,
    window: Query<&CrosstermWindow>,
    mut cursor: ResMut<Cursor>,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    cursor.hidden = true;
    let window = window.single();

    // Create our resources
    let plain = stylemaps.add(StyleMap::default());
    let white_bg = stylemaps.add(StyleMap::with_bg(Color::White));

    // Spawn two sprites into the world
    commands.spawn(SpriteBundle {
        sprite: sprites.add(Sprite::new(BIG_BOX)),
        position: Position {
            x: window.x_center() as i32 - 3,
            y: window.y_center() as i32 - 1,
            z: 0,
        },
        stylemap: white_bg.clone(),
        ..Default::default()
    });
    // Entity on top, transparent, so we can see the entity below
    commands.spawn(SpriteBundle {
        sprite: sprites.add(Sprite::new(SMALL_BOX)),
        position: Position {
            x: window.x_center() as i32 - 1,
            y: window.y_center() as i32 - 1,
            z: 1,
        },
        stylemap: plain.clone(),
        visible: Visible::transparent(),
    });
}
