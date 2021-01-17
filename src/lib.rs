//! # A Bevy asset loader for Everquest files
//!
//! # Example
//! ```rust
//! use bevy::prelude::*;
//! use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
//!
//! use bevy_eq_assets::EqAssetsPlugin;
//!
//! fn main() {
//!     App::build()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(EqAssetsPlugin)
//!         .add_startup_system(setup.system())
//!         .add_plugin(FlyCameraPlugin)
//!         .run();
//! }
//!
//! fn setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
//!     let map = asset_server.load("gfaydark.s3d#Wld[gfaydark.wld]/Map");
//!
//!     commands
//!         .spawn(Camera3dBundle {
//!             transform: Transform {
//!                 translation: Vec3::new(0.0, 0.0, 10.0),
//!                 ..Transform::default()
//!             }
//!             .looking_at(Vec3::default(), Vec3::unit_y()),
//!             ..Default::default()
//!         })
//!         .with(FlyCamera {
//!             speed: 5.0,
//!             max_speed: 5.0,
//!             key_forward: KeyCode::E,
//!             key_backward: KeyCode::D,
//!             key_left: KeyCode::S,
//!             key_right: KeyCode::F,
//!             key_up: KeyCode::A,
//!             key_down: KeyCode::Z,
//!             ..FlyCamera::default()
//!         })
//!         .spawn(LightBundle::default())
//!         .spawn((Transform::default(), GlobalTransform::default()))
//!         .with_children(|parent| {
//!             parent.spawn_scene(scene.clone());
//!         });
//! }
//! ```

mod loader;

use std::collections::HashMap;

use bevy_app::prelude::*;
use bevy_asset::{AddAsset, Handle};
use bevy_pbr::prelude::StandardMaterial;
use bevy_reflect::TypeUuid;
use bevy_render::{mesh::Mesh, prelude::Texture};

pub use loader::*;

/// Adds support for Everquest file loading to Apps
#[derive(Default)]
pub struct EqAssetsPlugin;

impl Plugin for EqAssetsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_asset_loader::<EqAssetsLoader>()
            .add_asset::<EqArchive>()
            .add_asset::<EqWld>()
            .add_asset::<EqNode>()
            .add_asset::<EqPrimitive>()
            .add_asset::<EqMesh>();
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "716e12d8-7f53-421c-baee-f4da9053c52b"]
pub struct EqArchive {
    pub named_sources: HashMap<String, Handle<Texture>>,
    pub named_wlds: HashMap<String, Handle<EqWld>>,
}

#[derive(Debug, TypeUuid)]
#[uuid = "899f49a7-e6e6-4661-897f-6c0f8ba07284"]
pub struct EqWld {
    pub meshes: Vec<Handle<EqMesh>>,
    pub named_meshes: HashMap<String, Handle<EqMesh>>,
    pub materials: Vec<Handle<StandardMaterial>>,
    pub named_materials: HashMap<String, Handle<StandardMaterial>>,
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "0a1bcf27-47df-4358-b95f-7ea917f7b5ba"]
pub struct EqNode {
    pub children: Vec<EqNode>,
    pub mesh: Option<Handle<EqMesh>>,
    pub transform: bevy_transform::prelude::Transform,
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "46556bd6-3155-4281-b2f7-adce8452d97f"]
pub struct EqMesh {
    pub primitives: Vec<EqPrimitive>,
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f44eea06-7f17-4511-a294-959edcbd27b6"]
pub struct EqPrimitive {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}
