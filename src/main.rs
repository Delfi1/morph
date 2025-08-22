use bevy::{
    app::*,
    prelude::*
};

mod networking;
use networking::*;

// debug camera. WIP
mod camera;
use camera::*;

mod renderer;
use renderer::*;

mod utils;
mod stdb;

mod ui;
use ui::*;

// Default window 
fn setup_window() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Morph".into(),
            ..default()
        }),
        ..default()
    }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 36.0, 0.0),
        MainCamera::new()
    ));
}

plugin_group! {
    /// All Morph Project default plugins setup
    pub struct MorphPlugins {
        // Bevy plugins
        bevy::app:::PanicHandlerPlugin,
        bevy::log:::LogPlugin,
        bevy::app:::TaskPoolPlugin,
        bevy::diagnostic:::FrameCountPlugin,
        bevy::time:::TimePlugin,
        bevy::transform:::TransformPlugin,
        bevy::diagnostic:::DiagnosticsPlugin,
        bevy::input:::InputPlugin,
        bevy::window:::WindowPlugin,
        bevy::a11y:::AccessibilityPlugin,
        #[custom(cfg(any(unix, windows)))]
        bevy::app:::TerminalCtrlCHandlerPlugin,
        bevy::asset:::AssetPlugin,
        bevy::scene:::ScenePlugin,
        bevy::winit:::WinitPlugin,
        bevy::render:::RenderPlugin,
        bevy::render::texture:::ImagePlugin,
        bevy::render::pipelined_rendering:::PipelinedRenderingPlugin,
        bevy::core_pipeline:::CorePipelinePlugin,
        bevy::animation:::AnimationPlugin,
        bevy::state::app:::StatesPlugin,
        bevy::sprite:::SpritePlugin,
        bevy::audio:::AudioPlugin,
        bevy::text:::TextPlugin,
        bevy::pbr:::PbrPlugin,
        bevy::ui:::UiPlugin,

        // Main morph plugins
        :NetworkingPlugin,
        :RenderingPlugin,
        :MorphUiPlugin,
        :CameraPlugin,
    }
}

fn main() {
    App::new()
        .add_plugins(
            MorphPlugins
            .set(setup_window())
            .set(ImagePlugin { default_sampler: default_sampler() })
        )
        .add_plugins(bevy::picking::DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .run();
}
