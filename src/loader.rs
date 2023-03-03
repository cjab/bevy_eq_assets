use std::collections::HashMap;

use anyhow::Result;
use bevy_asset::{AssetLoader, AssetPath, BoxedFuture, Handle, LoadContext, LoadedAsset};
use bevy_ecs::prelude::World;
use bevy_hierarchy::BuildWorldChildren;
use bevy_math::Vec3;
use bevy_pbr::prelude::{PbrBundle, StandardMaterial};
use bevy_render::{
    mesh::{Indices, Mesh, VertexAttributeValues},
    prelude::{Image, SpatialBundle},
    render_resource::{
        AddressMode, Extent3d, PrimitiveTopology, SamplerDescriptor, TextureDescriptor,
        TextureDimension, TextureFormat, TextureUsages,
    },
    texture::ImageSampler,
};
use bevy_scene::Scene;
use bevy_transform::prelude::Transform;

use bevy_utils::default;
use image::ImageFormat;
use log::{debug, error, info};

use super::{EqArchive, EqMesh, EqPrimitive, EqWld};

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
    let mut named_sources = HashMap::new();
    let mut named_wlds = HashMap::new();
    for (name, asset) in eq_archive::load(bytes)
        .expect("Failed to load archive")
        .files()
    {
        match name.splitn(2, ".").last() {
            Some("bmp") => {
                let source = load_bmp(&name[..], &asset[..], load_context);
                named_sources.insert(name, source);
            }
            Some("wld") => {
                let wld = load_wld(&name[..], &asset[..], load_context);
                named_wlds.insert(name, wld);
            }
            Some(_) => {
                error!("Unknown file type, ignoring: {}", name);
            }
            None => {
                error!("No filetype found, ignoring: {}", name);
            }
        }
    }

    load_context.set_default_asset(LoadedAsset::new(EqArchive {
        named_sources,
        named_wlds,
    }));
}

fn load_bmp(name: &str, bytes: &[u8], load_context: &mut LoadContext) -> Handle<Image> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Bmp)
        .expect("Failed to load bitmap")
        .into_rgba8();
    let format = TextureFormat::Rgba8UnormSrgb;
    let size = Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    };
    let data = image.into_raw();
    let label = texture_label(name);

    load_context.set_labeled_asset(
        &label,
        LoadedAsset::new(Image {
            data,
            texture_descriptor: TextureDescriptor {
                size,
                format,
                dimension: TextureDimension::D2,
                label: None,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
            sampler_descriptor: ImageSampler::Descriptor(SamplerDescriptor {
                address_mode_u: AddressMode::MirrorRepeat,
                address_mode_v: AddressMode::MirrorRepeat,
                ..Default::default()
            }),
            ..Default::default()
        }),
    );
    load_context.get_handle(AssetPath::new_ref(load_context.path(), Some(&label)))
}

fn load_wld(wld_name: &str, bytes: &[u8], load_context: &mut LoadContext) -> Handle<EqWld> {
    info!("Loading wld file: {}", wld_name);
    let wld = eq_wld::load(bytes).expect(&format!("Failed to load wld: {}", wld_name));

    // Load materials
    let mut materials = vec![];
    let mut named_materials: HashMap<String, Handle<_>> = HashMap::new();
    for material in wld.materials() {
        let label = material_label(wld_name, material.name().unwrap_or(""));

        let texture = match material.base_color_texture() {
            Some(t) => t,
            None => {
                debug!("{} has no color texture!", label);
                continue;
            }
        };
        let texture_name = match texture.source() {
            Some(t) => t,
            None => {
                debug!("{} has no texture source!", label);
                continue;
            }
        };

        let material_handle = load_material(&label, texture_name, load_context);
        if let Some(name) = material.name() {
            materials.push(material_handle.clone());
            named_materials.insert(name.to_string(), material_handle.clone());
        }
    }

    // Load meshes
    let mut meshes = vec![];
    let mut named_meshes = HashMap::new();
    let mut world = World::default();

    world
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            for mesh in wld.meshes() {
                let mut primitives = vec![];
                let (x, y, z) = mesh.center();
                parent
                    .spawn(SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(x, y, z)),
                        ..default()
                    })
                    .with_children(|parent| {
                        for primitive in mesh.primitives() {
                            let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);

                            // Set vertex positions
                            bevy_mesh.insert_attribute(
                                Mesh::ATTRIBUTE_POSITION,
                                VertexAttributeValues::Float32x3(primitive.positions()),
                            );

                            // Set normals
                            bevy_mesh.insert_attribute(
                                Mesh::ATTRIBUTE_NORMAL,
                                VertexAttributeValues::Float32x3(primitive.normals()),
                            );

                            // Set texture coordinates
                            let texture_coordinates = primitive.texture_coordinates();
                            if texture_coordinates.len() > 0 {
                                bevy_mesh.insert_attribute(
                                    Mesh::ATTRIBUTE_UV_0,
                                    VertexAttributeValues::Float32x2(texture_coordinates),
                                );
                            }

                            // Set vertex indices
                            bevy_mesh.set_indices(Some(Indices::U32(primitive.indices())));

                            let label = primitive_label(
                                wld_name,
                                mesh.name().unwrap_or(""),
                                primitive.index(),
                            );
                            load_context.set_labeled_asset(&label, LoadedAsset::new(bevy_mesh));
                            let mesh_handle: Handle<Mesh> = load_context
                                .get_handle(AssetPath::new_ref(load_context.path(), Some(&label)));
                            let material_handle = match named_materials
                                .get(primitive.material().name().unwrap())
                                .cloned()
                            {
                                Some(material) => material,
                                None => {
                                    debug!("Could not find {:?}", primitive.material().name());
                                    continue;
                                }
                            };

                            parent.spawn(PbrBundle {
                                mesh: mesh_handle.clone(),
                                material: material_handle.clone(),
                                ..Default::default()
                            });

                            primitives.push(EqPrimitive {
                                mesh: mesh_handle.clone(),
                                material: material_handle.clone(),
                            })
                        }
                    });

                let label = mesh_label(wld_name, mesh.name().unwrap_or(""));
                load_context.set_labeled_asset(&label, LoadedAsset::new(EqMesh { primitives }));
                let eq_mesh_handle: Handle<EqMesh> =
                    load_context.get_handle(AssetPath::new_ref(load_context.path(), Some(&label)));
                named_meshes.insert(label, eq_mesh_handle.clone());
                meshes.push(eq_mesh_handle.clone());
            }
        });

    let label = wld_label(wld_name);
    load_context.set_labeled_asset(
        &format!("{}/Map", label),
        LoadedAsset::new(Scene::new(world)),
    );
    load_context.set_labeled_asset(
        &label,
        LoadedAsset::new(EqWld {
            meshes,
            named_meshes,
            materials,
            named_materials,
        }),
    );
    info!("Loaded: {}", label);
    load_context.get_handle(AssetPath::new_ref(load_context.path(), Some(&label)))
}

fn load_material(
    label: &str,
    texture_name: String,
    load_context: &mut LoadContext,
) -> Handle<StandardMaterial> {
    let texture_label = texture_label(&texture_name);
    let texture_handle = load_context.get_handle(AssetPath::new_ref(
        load_context.path(),
        Some(&texture_label),
    ));

    load_context.set_labeled_asset(
        &label,
        LoadedAsset::new(StandardMaterial {
            base_color_texture: Some(texture_handle),
            unlit: true,
            ..Default::default()
        }),
    );
    load_context.get_handle(AssetPath::new_ref(load_context.path(), Some(&label)))
}

fn wld_label(wld_name: &str) -> String {
    format!("Wld[{}]", wld_name)
}

fn material_label(wld_name: &str, name: &str) -> String {
    format!("{}/Material[{}]", wld_label(wld_name), name)
}

fn texture_label(name: &str) -> String {
    format!("Texture[{}]", name)
}

fn mesh_label(wld_name: &str, name: &str) -> String {
    format!("{}/Mesh[{}]", wld_label(wld_name), name)
}

fn primitive_label(wld_name: &str, mesh_name: &str, primitive_index: usize) -> String {
    format!(
        "{}/Primitive[{}]",
        mesh_label(wld_name, mesh_name),
        primitive_index
    )
}
