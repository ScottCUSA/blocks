//! draws the frame's DrawList with bevy: one 2d mesh per vertex run, stacked in draw order

use bevy::asset::RenderAssetUsages;
use bevy::camera::ScalingMode;
use bevy::image::ImageSampler;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d,
    RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, ColorMaterialUniform, Material2d, Material2dKey, Material2dPlugin,
};

use crate::gfx::{Atlas, BlendMode, Depth, DrawList, Run};

const RUN_DEPTH: f32 = 0.001; // z between runs, later runs draw on top
pub const BOARD_Z: f32 = 100.0; // board sprites, over the back layer
const FRONT_Z: f32 = 200.0; // front layer, over the board

/// the frame being drawn, filled before PostUpdate's upload
#[derive(Resource, Default)]
pub struct Paint(pub DrawList);

/// the camera everything is drawn with
#[derive(Component)]
pub struct PaintCamera;

/// where the draw list goes to the gpu, fill Paint before it
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaintUpload;

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<PaintMaterial>::default())
            .init_resource::<Paint>()
            .add_systems(Startup, setup)
            .add_systems(PostUpdate, upload.in_set(PaintUpload));
    }
}

/// vertex colors times the atlas, blended over or added to what's below
#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(PaintKey)]
pub struct PaintMaterial {
    // the color material's shader and layout
    #[uniform(0)]
    params: ColorMaterialUniform,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
    additive: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct PaintKey {
    additive: bool,
}

impl From<&PaintMaterial> for PaintKey {
    fn from(material: &PaintMaterial) -> Self {
        PaintKey {
            additive: material.additive,
        }
    }
}

impl Material2d for PaintMaterial {
    fn fragment_shader() -> ShaderRef {
        <ColorMaterial as Material2d>::fragment_shader()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &bevy::sprite_render::Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.additive {
            let add = BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            };
            let state = BlendState {
                color: add,
                alpha: add,
            };
            if let Some(fragment) = &mut descriptor.fragment {
                for target in fragment.targets.iter_mut().flatten() {
                    target.blend = Some(state);
                }
            }
        }
        Ok(())
    }
}

/// gpu side of the draw list
#[derive(Resource)]
struct Gpu {
    atlas: Handle<Image>,
    alpha: Handle<PaintMaterial>,
    add: Handle<PaintMaterial>,
    runs: Vec<(Entity, Handle<Mesh>)>, // pooled, one per run
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<PaintMaterial>>,
) {
    let mut image = Image::new(
        Extent3d {
            width: Atlas::SIZE,
            height: Atlas::SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; (Atlas::SIZE * Atlas::SIZE * 4) as usize],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    let atlas = images.add(image);
    let params = ColorMaterialUniform {
        color: Vec4::ONE,
        uv_transform: Mat3::IDENTITY,
        // texture bit and blend alpha mode, as in the color material shader
        flags: 1 | (2 << 30),
        alpha_cutoff: 0.5,
    };
    let mut material = |additive| {
        materials.add(PaintMaterial {
            params: params.clone(),
            texture: atlas.clone(),
            additive,
        })
    };
    let gpu = Gpu {
        alpha: material(false),
        add: material(true),
        atlas,
        runs: Vec::new(),
    };
    commands.insert_resource(gpu);
}

fn run_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
}

/// hand this frame's runs to their meshes
fn upload(
    mut commands: Commands,
    mut paint: ResMut<Paint>,
    mut gpu: ResMut<Gpu>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut clear: ResMut<ClearColor>,
    mut cameras: Query<&mut Projection, With<PaintCamera>>,
    mut entities: Query<(
        &mut Visibility,
        &mut MeshMaterial2d<PaintMaterial>,
        &mut Transform,
    )>,
) {
    let list = &mut paint.0;
    let c = list.clear;
    clear.0 = Color::srgba(c.r, c.g, c.b, c.a);

    // whole physical pixels across the target, y down from the top left
    let size = list.size;
    for mut projection in &mut cameras {
        let Projection::Orthographic(ortho) = &*projection else {
            continue;
        };
        let current = matches!(ortho.scaling_mode,
            ScalingMode::Fixed { width, height } if width == size.x && height == size.y);
        if !current {
            if let Projection::Orthographic(ortho) = &mut *projection {
                ortho.scaling_mode = ScalingMode::Fixed {
                    width: size.x,
                    height: size.y,
                };
            }
        }
    }

    if list.atlas.dirty {
        if let Some(mut image) = images.get_mut(&gpu.atlas) {
            image.data = Some(list.atlas.pixels.clone());
        }
        list.atlas.dirty = false;
    }

    while gpu.runs.len() < list.used {
        let mesh = meshes.add(run_mesh());
        let entity = commands
            .spawn((Mesh2d(mesh.clone()), MeshMaterial2d(gpu.alpha.clone())))
            .id();
        gpu.runs.push((entity, mesh));
    }

    let half = size / 2.0;
    for (i, (entity, handle)) in gpu.runs.iter().enumerate() {
        let shown = i < list.used;
        let run = &list.runs[i];
        let material = match run.blend {
            Some(BlendMode::Add) => &gpu.add,
            _ => &gpu.alpha,
        };
        let z = match run.layer {
            Depth::Back => 0.0,
            Depth::Front => FRONT_Z,
        } + i as f32 * RUN_DEPTH;
        // spawned this frame, set up through commands
        let Ok((mut vis, mut current, mut transform)) = entities.get_mut(*entity) else {
            if !shown {
                continue;
            }
            commands.entity(*entity).insert((
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(0.0, 0.0, z),
            ));
            update_mesh(&mut meshes, handle, run, half);
            continue;
        };
        let wanted = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        vis.set_if_neq(wanted);
        if !shown {
            continue;
        }
        if current.0 != *material {
            current.0 = material.clone();
        }
        if transform.translation.z != z {
            transform.translation.z = z;
        }
        update_mesh(&mut meshes, handle, run, half);
    }
}

/// a run's vertices, centered with y up
fn update_mesh(meshes: &mut Assets<Mesh>, handle: &Handle<Mesh>, run: &Run, half: Vec2) {
    let Some(mut mesh) = meshes.get_mut(handle) else {
        return;
    };
    let positions: Vec<[f32; 3]> = run
        .vertices
        .iter()
        .map(|v| [v.position[0] - half.x, half.y - v.position[1], 0.0])
        .collect();
    let uvs: Vec<[f32; 2]> = run.vertices.iter().map(|v| v.uv).collect();
    let colors: Vec<[f32; 4]> = run.vertices.iter().map(|v| v.color).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(run.indices.clone()));
}
