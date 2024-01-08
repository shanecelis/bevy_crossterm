use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowFocused;
use bevy_crossterm::prelude::*;

// Use crossterm focus events to pause an animation

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Focus example");

    App::new()
        .insert_resource(settings)
        .add_plugins(bevy_app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(16),
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
        .insert_resource(Countdown(
            Timer::new(std::time::Duration::from_millis(100), TimerMode::Repeating),
            true,
        ))
        .add_systems(Startup, startup_system)
        .add_systems(Update, update)
        .run();
}

#[derive(Resource)]
struct Countdown(Timer, bool);

#[derive(Component)]
struct Tag;

fn startup_system(
    mut commands: Commands,
    mut cursor: ResMut<Cursor>,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    cursor.hidden = true;

    let text = sprites.add(Sprite::new("If the terminal loses focus, the animation will pause."));
    let text2 = sprites.add(Sprite::new("If the terminal regains focus, the animation will continue."));
    let plain = stylemaps.add(StyleMap::default());

    // Static entity
    commands.spawn(SpriteBundle {
        sprite: text,
        stylemap: plain.clone(),
        ..Default::default()
    });
    // Moving entity that will stop if we loose focus
    commands.spawn((SpriteBundle {
        sprite: text2,
        stylemap: plain.clone(),
        position: Position::with_y(2),
        ..Default::default()
    }, Tag,));
}

fn update(
    time: Res<Time>,
    window: Query<&CrosstermWindow>,
    mut timer: ResMut<Countdown>,
    mut query: Query<(&Tag, &mut Position)>,
    mut focus: EventReader<WindowFocused>,
) {
    let window = window.single();

    for focus in focus.read() {
        timer.1 = focus.focused;
    }

    if !timer.1 {
        return;
    }

    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        let (_, mut pos) = query.iter_mut().next().unwrap();
        pos.x += 1;

        if pos.x > (window.width() as i32 / 3) {
            pos.x = 0
        }
    }
}
