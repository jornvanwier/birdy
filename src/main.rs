mod audio;
mod big_bird;
mod camera;
mod debug;
pub mod environment;
mod input;
mod ship;
mod ui;

use avian3d::prelude::*;
use bevy::audio::{AudioPlugin, Volume};
use bevy::prelude::*;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_hanabi::HanabiPlugin;
use bevy_rand::prelude::*;
use big_space::prelude::*;
use shadow_rs::shadow;

shadow!(build_info);

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(120.0))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault,
            },
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Birdy Flight Sim".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),

                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume::new(Volume::Linear(0.1)),
                    ..default()
                })
                .disable::<TransformPlugin>(),
            BigSpaceDefaultPlugins,
            PhysicsPlugins::default(),
            big_bird::BigSpaceAvianSyncPlugin,
            big_bird::BigSpaceHanabiSyncPlugin,
            EntropyPlugin::<WyRand>::default(),
            HanabiPlugin,
            environment::EnvironmentPlugin,
            audio::ProceduralAudioPlugin,
            input::InputPlugin,
            ship::ShipPlugin,
            ui::HudPlugin,
            camera::ChaseCameraPlugin,
            debug::DebugPlugin,
        ))
        .add_systems(PreStartup, set_global_default_font)
        .run();
}

/// Override default font with JetBrains Mono
fn set_global_default_font(mut fonts: ResMut<Assets<Font>>) {
    const FONT_DATA: &[u8] = include_bytes!("../assets/game/fonts/JetBrainsMono-Regular.ttf");

    let font = Font::from_bytes(FONT_DATA.to_vec());

    // Overwrite Bevy's default font handle
    fonts
        .insert(&Handle::default(), font)
        .expect("Failed to insert font");
}
