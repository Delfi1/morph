use bevy::{
    asset::*,
    prelude::*
};
use bevy_cobweb_ui::prelude::*;

#[derive(Default)]
pub struct MorphUiPlugin;
impl Plugin for MorphUiPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "main.cob");
    }
}

pub fn _build_ui(
    mut commands: Commands,
    mut ui: SceneBuilder
) {
    commands.spawn((
        Camera2d::default(),
    ));

    let path = ("embedded://morph/main.cob", "scene");
    commands.ui_root()
        .spawn_scene_simple(path, &mut ui);
}
