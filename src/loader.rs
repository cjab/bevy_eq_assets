use anyhow::Result;
use bevy_asset::{AssetIoError, AssetLoader, AssetPath, Handle, LoadContext, LoadedAsset};
use bevy_ecs::{bevy_utils::BoxedFuture, World, WorldBuilderSource};
use bevy_math::Mat4;
use bevy_pbr::prelude::{PbrBundle, StandardMaterial};
use bevy_render::{
    camera::{
        Camera, CameraProjection, OrthographicProjection, PerspectiveProjection, VisibleEntities,
    },
    mesh::{Indices, Mesh, VertexAttributeValues},
    pipeline::PrimitiveTopology,
    prelude::{Color, Texture},
    render_graph::base,
    texture::{
        AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat,
    },
};
use bevy_scene::Scene;
use bevy_transform::{
    hierarchy::{BuildWorldChildren, WorldChildBuilder},
    prelude::{GlobalTransform, Transform},
};

use image::ImageFormat;
use log::{error, info};

#[derive(Default)]
pub struct EqAssetsLoader;

impl AssetLoader for EqAssetsLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { Ok(load_eq_archive(bytes, load_context)) })
    }

    fn extensions(&self) -> &[&str] {
        &["s3d", "eqg"]
    }
}

fn load_eq_archive(bytes: &[u8], load_context: &mut LoadContext) {
    for (name, asset) in eq_archive::load(bytes)
        .expect("Failed to load archive")
        .files()
    {
        match name.splitn(2, ".").last() {
            Some("bmp") => {
                load_bmp(&name[..], &asset[..], load_context);
            }
            Some("wld") => {
                load_wld(&name[..], &asset[..], load_context);
            }
            Some(_) => {
                error!("Unknown file type, ignoring: {}", name);
            }
            None => {
                error!("No filetype found, ignoring: {}", name);
            }
        }
    }
}

fn load_bmp(name: &str, bytes: &[u8], load_context: &mut LoadContext) {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Bmp)
        .expect("Failed to load bitmap")
        .into_rgba8();
    let format = TextureFormat::Rgba8UnormSrgb;
    let size = Extent3d::new(image.width(), image.height(), 1);
    let data = image.into_raw();
    let label = texture_label(name);

    load_context.set_labeled_asset(
        &label,
        LoadedAsset::new(Texture {
            data,
            size,
            format,
            dimension: TextureDimension::D2,
            sampler: SamplerDescriptor {
                address_mode_u: AddressMode::MirrorRepeat,
                address_mode_v: AddressMode::MirrorRepeat,
                ..SamplerDescriptor::default()
            },
            ..Texture::default()
        }),
    );
    println!("Loaded {}", label);
}

fn load_wld(name: &str, bytes: &[u8], load_context: &mut LoadContext) {
    info!("Loading wld file: {}", name);
    let wld = eq_wld::load(bytes).expect(&format!("Failed to load wld: {}", name));

    // Load meshes
    for mesh in wld.meshes() {
        let label = mesh_label(mesh.name().unwrap_or(""));
        let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // Set vertex positions
        bevy_mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float3(mesh.positions()),
        );

        // Set normals
        bevy_mesh.set_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float3(mesh.normals()),
        );

        let texture_coordinates = mesh.texture_coordinates();
        if texture_coordinates.len() > 0 {
            // Set texture coordinates
            bevy_mesh.set_attribute(
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float2(texture_coordinates),
            );
        }

        // Set vertex indices
        bevy_mesh.set_indices(Some(Indices::U32(mesh.indices())));

        println!("Loaded {}", label);
        load_context.set_labeled_asset(&label, LoadedAsset::new(bevy_mesh));
    }

    // Load materials
    for material in wld.materials() {
        let label = material_label(material.name().unwrap_or(""));

        let texture = match material.base_color_texture() {
            Some(t) => t,
            None => {
                println!("{} has no color texture!", label);
                continue;
            }
        };
        let texture_name = match texture.source() {
            Some(t) => t,
            None => {
                println!("{} has no texture source!", label);
                continue;
            }
        };

        load_material(&label, texture_name, load_context);
    }
}

fn load_material(label: &str, texture_name: String, load_context: &mut LoadContext) {
    let texture_label = texture_label(&texture_name);
    let path = AssetPath::new_ref(load_context.path(), Some(&texture_label));
    let texture_handle = load_context.get_handle(path);

    load_context.set_labeled_asset(
        &label,
        LoadedAsset::new(StandardMaterial {
            albedo_texture: Some(texture_handle),
            shaded: false,
            ..Default::default()
        }),
    );
    println!("Loaded {}", label);
}

fn material_label(name: &str) -> String {
    format!("Material[{}]", name)
}

fn texture_label(name: &str) -> String {
    format!("Texture[{}]", name)
}

fn mesh_label(name: &str) -> String {
    format!("Mesh[{}]", name)
}

fn primitive_label(mesh_name: &str, primitive_index: u16) -> String {
    format!("Mesh[{}]/Primitive[{}]", mesh_name, primitive_index)
}
