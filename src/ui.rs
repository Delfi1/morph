use bevy::{
    asset::embedded_asset,
    prelude::*,
};
use bevy_cobweb_ui::prelude::*;

#[derive(Default)]
pub struct MorphUiPlugin;
impl Plugin for MorphUiPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "main.cob");

        app.add_plugins(CobwebUiPlugin)
            .load("embedded://morph/main.cob");
            //.add_systems(OnEnter(LoadState::Done), build_ui);
    }
}

pub fn on_click() {
    info!("Click!");
}

pub fn on_entered() {
    info!("Entered!");
}

pub fn _build_ui(
    mut commands: Commands,
    mut ui: SceneBuilder,
) {
    commands.spawn((
        Camera2d::default(),
    ));

    // Main cobweb file root
    let root = "embedded://morph/main.cob";
    commands.ui_root()
        .spawn_scene((root, "menu"), &mut ui, |handle| {
            handle.get("test_button")
                .on_pressed(on_click)
                .on_pointer_enter(on_entered);
        });
}
