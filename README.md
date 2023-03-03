[![Crates.io](https://img.shields.io/crates/v/eq_wld.svg)](https://crates.io/crates/eq_wld)
[![Docs.rs](https://docs.rs/eq_wld/badge.svg)](https://docs.rs/eq_wld)

# bevy_eq_assets

## A Bevy asset loader for Everquest files

## Example
```rust
use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

use bevy_eq_assets::EqAssetsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EqAssetsPlugin)
        .add_startup_system(setup)
        .add_plugin(FlyCameraPlugin)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map = asset_server.load("gfaydark.s3d#Wld[gfaydark.wld]/Map");

    commands
        .spawn(Camera3dBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                ..Transform::default()
            }
            .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCamera {
            accel: 5.0,
            max_speed: 5.0,
            key_forward: KeyCode::E,
            key_backward: KeyCode::D,
            key_left: KeyCode::S,
            key_right: KeyCode::F,
            key_up: KeyCode::A,
            key_down: KeyCode::Z,
            ..FlyCamera::default()
        });
    commands
        .spawn(PointLightBundle::default());
    commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn(SceneBundle {
                scene: map,
                ..default()
            });
        });
}
```
