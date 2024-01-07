#![feature(trivial_bounds)]
use bevy::prelude::*;
use bevy_app::App;

mod asset_loaders;
pub mod components;
pub mod prelude;
mod runner;
mod systems;

pub struct CrosstermPlugin;
impl Plugin for CrosstermPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cursor::default())
            .insert_resource(components::PreviousEntityDetails::default())
            .insert_resource(components::EntitiesToRedraw::default())
            .insert_resource(components::PreviousWindowColors::default())
            // Custom assets
            .register_asset_loader(asset_loaders::SpriteLoader)
            .init_asset::<components::Sprite>()
            .register_asset_loader(crate::asset_loaders::StyleMapLoader)
            .init_asset::<components::StyleMap>()
            // Crossterm events
            .add_event::<CrosstermKeyEventWrapper>()
            .add_event::<CrosstermMouseEventWrapper>()
            .set_runner(runner::crossterm_runner)
            // TODO check if asset events work correctly this way
            // Old comment:
            // This must be before LAST because change tracking is cleared during LAST, but AssetEvents are published
            // after POST_UPDATE. The timing for all these things is pretty delicate
            .add_systems(
                PostUpdate,
                (
                    systems::add_previous_position,
                    systems::calculate_entities_to_redraw,
                    systems::crossterm_render,
                    systems::update_previous_position,
                )
                    .chain(),
            );
    }
}

#[derive(Event)]
pub struct CrosstermKeyEventWrapper(pub crossterm::event::KeyEvent);
impl From<crossterm::event::KeyEvent> for CrosstermKeyEventWrapper {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        CrosstermKeyEventWrapper(event)
    }
}
#[derive(Event)]
pub struct CrosstermMouseEventWrapper(pub crossterm::event::MouseEvent);
impl From<crossterm::event::MouseEvent> for CrosstermMouseEventWrapper {
    fn from(event: crossterm::event::MouseEvent) -> Self {
        CrosstermMouseEventWrapper(event)
    }
}

#[derive(Clone, Eq, PartialEq, Resource)]
pub struct CrosstermWindowSettings {
    colors: components::Colors,
    title: Option<String>,
}

impl Default for CrosstermWindowSettings {
    fn default() -> Self {
        CrosstermWindowSettings {
            colors: components::Colors::term_colors(),
            title: None,
        }
    }
}

impl CrosstermWindowSettings {
    pub fn colors(&self) -> components::Colors {
        self.colors
    }

    pub fn title(&self) -> &Option<String> {
        &self.title
    }

    pub fn set_title<T: std::string::ToString>(&mut self, title: T) -> &mut Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn set_colors(&mut self, colors: components::Colors) -> &mut Self {
        self.colors = colors;
        self
    }
}

#[derive(Debug, Component)]
pub struct CrosstermWindow {
    height: u16,
    width: u16,
    colors: components::Colors,
    title: Option<String>,
}

impl Default for CrosstermWindow {
    fn default() -> Self {
        let (width, height) =
            crossterm::terminal::size().expect("Could not read current terminal size");

        let colors = components::Colors::term_colors();
        CrosstermWindow {
            height,
            width,
            colors,
            title: None,
        }
    }
}

impl CrosstermWindow {
    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn colors(&self) -> components::Colors {
        self.colors
    }

    pub fn set_colors(&mut self, new_colors: components::Colors) {
        self.colors = new_colors;
    }

    pub fn x_center(&self) -> u16 {
        self.width / 2
    }

    pub fn y_center(&self) -> u16 {
        self.height / 2
    }
}

#[derive(Debug, Default, Resource)]
pub struct Cursor {
    pub x: i32,
    pub y: i32,
    pub hidden: bool,
}

//TODO unsure how to handle schedules in bevy 0.11
// pub mod stage {
//     use bevy_ecs::schedule::ScheduleLabel;
//
//     pub const PRE_RENDER: &str = "pre_render";
//     pub const RENDER: &str = "render";
//     pub const POST_RENDER: &str = "post_render";
//
//     #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
//     pub struct PreRender;
//     #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
//     pub struct Render;
//     #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
//     pub struct PostRender;
// }
