use crate::{
    CrosstermKeyEventWrapper, CrosstermMouseEventWrapper, CrosstermWindow, CrosstermWindowSettings,
};
use std::io::Write;

use bevy::window::{PrimaryWindow, WindowCreated, WindowResized};
use bevy_app::{App, AppExit};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Events;
use crossterm::{queue, ExecutableCommand, QueueableCommand};

impl CrosstermWindow {
    /// Creates a new CrosstermWindow and prepares crossterm for rendering.
    fn new(settings: &CrosstermWindowSettings) -> Self {
        crossterm::terminal::enable_raw_mode().expect("Could not enable crossterm raw mode");

        let mut term = std::io::stdout();
        queue!(
            term,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture,
            crossterm::event::EnableFocusChange,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All,),
        )
        .expect("Could not queue commands");

        let title = if let Some(title) = &settings.title {
            term.queue(crossterm::terminal::SetTitle(title))
                .expect("Could not set terminal title");
            Some(title.clone())
        } else {
            None
        };

        let colors = settings.colors;
        term.queue(crossterm::style::SetColors(colors.to_crossterm()))
            .expect("Could not set window colors");

        term.flush().expect("Could not initialize terminal");

        let (width, height) =
            crossterm::terminal::size().expect("Could not read current terminal size");

        Self {
            height,
            width,
            colors,
            title,
        }
    }
}

// Ensure teardown even if we encounter a panic
impl Drop for CrosstermWindow {
    fn drop(&mut self) {
        let mut term = std::io::stdout();
        queue!(
            term,
            crossterm::event::DisableMouseCapture,
            crossterm::event::DisableFocusChange,
            crossterm::cursor::Show,
        )
        .expect("Could not queue commands");
        term.flush().expect("Could not reset terminal");

        crossterm::terminal::disable_raw_mode().expect("Could not disable raw mode");
    }
}

pub fn crossterm_runner(mut app: App) {
    let bevy_window = setup_window(&mut app);

    // There should only be one ScheduleRunnerPlugin, but if there isn't, add one
    // (also there might be a better way to do this)
    let settings = app.get_added_plugins::<bevy_app::ScheduleRunnerPlugin>();
    let settings = if !settings.is_empty() {
        settings[0]
    } else {
        app.add_plugins(bevy_app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(50),
        ));
        app.get_added_plugins::<bevy_app::ScheduleRunnerPlugin>()[0]
    };

    match settings.run_mode {
        bevy::app::RunMode::Once => {
            app.update();
        }
        bevy::app::RunMode::Loop { wait } => {
            // Run the main loop, and delay if we need to
            let mut start_time = std::time::Instant::now();
            while tick(&mut app, bevy_window).is_ok() {
                let end_time = std::time::Instant::now();

                if let Some(wait) = wait {
                    let exe_time = end_time - start_time;
                    if exe_time < wait {
                        let delay = wait - exe_time;
                        // dbg!(delay);
                        std::thread::sleep(delay);
                    }
                }

                start_time = end_time;
            }

            // Cleanup and teardown
            // Most teardown is done by the drop implementation of CrosstermWindow, which will run even if we encounter
            // a panic (provided we do not run in panic="abort" mode)
            // We do __NOT__ want to leave the alternate screen after a panic, because that would wipe out the panic
            // message
            let mut term = std::io::stdout();
            term.execute(crossterm::terminal::LeaveAlternateScreen)
                .expect("Could not leave alternate terminal");
        }
    }
}

/// Setup the crossterm window, so it is available to the rest of the app
fn setup_window(app: &mut App) -> Entity {
    app.init_resource::<CrosstermWindowSettings>();

    let window_settings = app.world.resource::<CrosstermWindowSettings>();
    let window = CrosstermWindow::new(window_settings);

    // Insert our window entity so that other parts of our app can use them
    let bevy_window = app.world.spawn(window).insert(PrimaryWindow).id();

    // Publish to the app that a terminal window has been created
    app.world.send_event(WindowCreated {
        window: bevy_window,
    });

    bevy_window
}

/// A single game update
fn tick(app: &mut App, bevy_window: Entity) -> Result<(), AppExit> {
    crossterm_events(&mut app.world, bevy_window);

    // Yield execution to the rest of bevy and it's scheduler
    app.update();

    // After all the other systems have updated, check if there are any AppExit events and
    // handle them
    {
        let app_exit_events = app.world.resource::<Events<AppExit>>();
        let mut app_exit_reader = app_exit_events.get_reader();
        if app_exit_reader.read(app_exit_events).next().is_some() {
            // We're breaking out, the app requested an exit
            return Err(AppExit);
        };
    }

    Ok(())
}

/// Check if any events are immediately available and if so, read them and republish
fn crossterm_events(world: &mut bevy_ecs::world::World, bevy_window: Entity) {
    while let Ok(available) = crossterm::event::poll(std::time::Duration::from_secs(0)) {
        if available {
            match crossterm::event::read().unwrap() {
                // Republish keyboard events in bevy
                crossterm::event::Event::Key(key_event) => {
                    // If the key event is for C-c, submit a AppExit event so the application
                    // can be killed
                    use crossterm::event::{KeyCode, KeyModifiers};
                    if key_event.code == KeyCode::Char('c')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        world.send_event(AppExit);
                    }

                    world.send_event(CrosstermKeyEventWrapper(key_event));
                }

                // Republish mouse events in bevy
                crossterm::event::Event::Mouse(mouse_event) => {
                    world.send_event(CrosstermMouseEventWrapper(mouse_event));
                }

                // Send a bevy window resized event if the terminal is resized, and also change the persisted window state
                crossterm::event::Event::Resize(width, height) => {
                    // Update the window resource and publish an event for the window being resized
                    world.send_event(WindowResized {
                        window: bevy_window,
                        width: width as f32,
                        height: height as f32,
                    });

                    let mut window_component =
                        world.get_mut::<CrosstermWindow>(bevy_window).unwrap();

                    window_component.height = height;
                    window_component.width = width;
                }

                // Send a bevy window focused event
                crossterm::event::Event::FocusGained => {
                    world.send_event(bevy::window::WindowFocused {
                        window: bevy_window,
                        focused: true,
                    })
                }
                crossterm::event::Event::FocusLost => {
                    world.send_event(bevy::window::WindowFocused {
                        window: bevy_window,
                        focused: false,
                    })
                }

                // Ignore bracketed paste. It's not well supported on windows.
                // If it's ever required it should be easy to add a wrapper for it.
                crossterm::event::Event::Paste(_) => {}
            }
        } else {
            break;
        }
    }
}
