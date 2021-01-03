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
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(EqAssetsPlugin)
        .add_startup_system(setup.system())
        .add_plugin(FlyCameraPlugin)
        .run();
}

fn setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let material_handle = asset_server.load("gfaydark.s3d#Material[SGRASS_MDF]");
    let mdl_handle = asset_server.load("gfaydark.s3d#Mesh[R43_DMSPRITEDEF]");

    commands
        .spawn(Camera3dBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                ..Transform::default()
            }
            .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .with(FlyCamera {
            speed: 5.0,
            max_speed: 5.0,
            key_forward: KeyCode::E,
            key_backward: KeyCode::D,
            key_left: KeyCode::S,
            key_right: KeyCode::F,
            key_up: KeyCode::A,
            key_down: KeyCode::Z,
            ..FlyCamera::default()
        })
        .spawn(LightBundle::default())
        .spawn(PbrBundle {
            mesh: mdl_handle.clone(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, -50.0, 0.0),
                rotation: Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                ..Default::default()
            },
            ..Default::default()
        });
}
```
