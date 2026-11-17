//! the board's blocks and effects as sprite entities, between the back and front paint layers

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::board::*;
use super::*;
use crate::effects::{BoardFx, Impact, Particle, Trail};
use crate::game::board::{Board, Mode};
use crate::gfx::{Color, IVec2, Rect, Vec2};
use crate::playfield::SlotState;
use crate::render::BOARD_Z;
use crate::style::{BlockStyle, Style};

const SHAPE_TEXTURE: u32 = 128; // px, rounded shape textures
const LAYER_DEPTH: f32 = 0.001; // z between the layers of a look
                                // z of each kind of look, later draws on top
const BLOCK_Z: f32 = 1.0;
const GHOST_Z: f32 = 2.0;
const FLASH_Z: f32 = 3.0;
const TRAIL_Z: f32 = 4.0;
const IMPACT_Z: f32 = 5.0;
const PARTICLE_Z: f32 = 6.0;

/// whether the board is drawn, and its shake in view units
#[derive(Resource, Default)]
pub struct BoardShown {
    pub shown: bool,
    pub shake: Option<Vec2>,
}

/// parent of every board sprite, carries the shake
#[derive(Component)]
pub struct BoardRoot;

/// rects drawn as child sprites, bottom first, in view units
#[derive(Component, Clone, PartialEq, Default)]
pub struct Look {
    pub layers: Vec<BlockLayer>,
    pub z: f32,
}

/// a pooled look showing board blocks
#[derive(Component)]
struct BoardBlock;

/// one rect of a look
#[derive(Component)]
struct LookLayer;

#[derive(Resource)]
struct ShapeImages {
    rounded: Handle<Image>,
    outline: Handle<Image>,
}

/// lays out the board sprites, set BoardShown before it
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoardSprites;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        // in Update: bevy turns changed sprites into meshes early in PostUpdate,
        // a sprite changed after that shows its old look for a frame
        app.init_resource::<BoardShown>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    place_root,
                    adopt_effects,
                    (sync_blocks, effect_looks),
                    layout_looks,
                )
                    .chain()
                    .in_set(BoardSprites),
            );
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let radius = JELLY_RADIUS / BLOCK_SIZE;
    let width = GHOST_OUTLINE_WIDTH / BLOCK_SIZE;
    let rounded = shape_image(|p| rounded_coverage(p, 0.0, radius));
    let outline = shape_image(|p| {
        rounded_coverage(p, 0.0, radius) - rounded_coverage(p, width, radius - width)
    });
    commands.insert_resource(ShapeImages {
        rounded: images.add(rounded),
        outline: images.add(outline),
    });
    commands.spawn((
        BoardRoot,
        Transform::from_xyz(0.0, 0.0, BOARD_Z),
        Visibility::Hidden,
    ));
}

/// coverage of a unit rounded square inset by `inset`, at p in 0..1
fn rounded_coverage(p: Vec2, inset: f32, radius: f32) -> f32 {
    let half = 0.5 - inset;
    let q = (p - Vec2::splat(0.5)).abs() - Vec2::splat(half - radius);
    let distance = q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - radius;
    // a pixel wide edge
    (0.5 - distance * SHAPE_TEXTURE as f32).clamp(0.0, 1.0)
}

fn shape_image(coverage: impl Fn(Vec2) -> f32) -> Image {
    let size = SHAPE_TEXTURE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) / size as f32;
            let alpha = (coverage(p).clamp(0.0, 1.0) * 255.0).round() as u8;
            data.extend([255, 255, 255, alpha]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

/// show the board while its screen is up, offset by the shake
fn place_root(
    shown: Res<BoardShown>,
    view: Res<ViewSettings>,
    mut roots: Query<(&mut Transform, &mut Visibility), With<BoardRoot>>,
) {
    for (mut transform, mut visibility) in &mut roots {
        visibility.set_if_neq(if shown.shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        // view units, y down, to pixels, y up
        let shake = shown.shake.unwrap_or_default() * view.scale;
        let translation = Vec3::new(shake.x, -shake.y, BOARD_Z);
        if transform.translation != translation {
            transform.translation = translation;
        }
    }
}

/// effects spawned by the effects plugin go under the board
fn adopt_effects(
    mut commands: Commands,
    roots: Query<Entity, With<BoardRoot>>,
    added: Query<Entity, Or<(Added<Particle>, Added<Trail>, Added<Impact>)>>,
) {
    let Ok(root) = roots.single() else {
        return;
    };
    for entity in &added {
        commands
            .entity(entity)
            // reset effects can despawn it first
            .try_insert((ChildOf(root), Look::default(), Transform::default()));
    }
}

/// looks for the board's blocks: locked, active, ghost, previews and the flash
fn sync_blocks(
    mut commands: Commands,
    board: Res<Board>,
    style: Res<Style>,
    mode: Res<Mode>,
    fx: Res<BoardFx>,
    view: Res<ViewSettings>,
    shown: Res<BoardShown>,
    roots: Query<Entity, With<BoardRoot>>,
    mut pool: Query<&mut Look, With<BoardBlock>>,
    mut entities: Local<Vec<Entity>>,
) {
    let Ok(root) = roots.single() else {
        return;
    };
    let looks = if shown.shown {
        board_looks(&board, style.block, mode.assists(), &fx, &view)
    } else {
        Vec::new()
    };
    while entities.len() < looks.len() {
        let entity = commands
            .spawn((
                BoardBlock,
                Look::default(),
                Transform::default(),
                Visibility::default(),
                ChildOf(root),
            ))
            .id();
        entities.push(entity);
    }
    for (i, entity) in entities.iter().enumerate() {
        let look = looks.get(i).cloned().unwrap_or_default();
        match pool.get_mut(*entity) {
            Ok(mut current) => {
                current.set_if_neq(look);
            }
            // spawned this frame
            Err(_) => {
                commands.entity(*entity).insert(look);
            }
        }
    }
}

fn board_looks(
    board: &Board,
    style: BlockStyle,
    assists: bool,
    fx: &BoardFx,
    view: &ViewSettings,
) -> Vec<Look> {
    let playfield = &board.playfield;
    let game_over = fx.game_over.as_ref();
    let (staging_rect, playfield_rect) = (&view.staging_rect, &view.playfield_rect);
    let mut looks = Vec::new();
    let block = |rect: Rect, color: Color| Look {
        layers: block_layers(rect, color, style),
        z: BLOCK_Z,
    };

    // locked blocks
    for (y, row) in playfield.slots.iter().enumerate() {
        let clearing = playfield
            .line_clear
            .as_ref()
            .filter(|line_clear| line_clear.lines.contains(&y));
        for (x, slot) in row.iter().enumerate() {
            let SlotState::Locked(rtype) = slot else {
                continue;
            };
            let mut color = block_color(*rtype, game_over, y as i32);
            if let Some(line_clear) = clearing {
                let progress = line_clear.progress() as f32;
                let flash = (1.0 - progress * 4.0).max(0.0);
                let base = if line_clear.lines.len() >= 4 {
                    // glint riding the wipe outward
                    let bottom = line_clear.lines.iter().min().copied().unwrap_or(y);
                    let peak = line_clear.wipe_start(x) as f32
                        + QUAD_GLINT_LEAD
                        + (y - bottom) as f32 * QUAD_GLINT_ROW;
                    let glint = (1.0 - (progress - peak).abs() / QUAD_GLINT_WIDTH).max(0.0);
                    mix_color(LINE_CLEAR_COLOR, Color::WHITE, glint * 0.8)
                } else {
                    LINE_CLEAR_COLOR
                };
                color = mix_color(base, Color::WHITE, flash);
                color.a *= line_clear_alpha(line_clear, x);
            }
            let rect = playfield_block_rect([x as i32, y as i32], staging_rect, playfield_rect);
            looks.push(block(rect, color));
        }
    }

    // the active piece
    if let Some(active) = &playfield.active_rustomino {
        for slot in active.playfield_slots() {
            let rect = playfield_block_rect([slot.x, slot.y], staging_rect, playfield_rect);
            looks.push(block(rect, block_color(active.rtype, game_over, slot.y)));
        }
    }

    // previews
    let mut preview = |rustomino: &Option<_>, panel: &Rect| {
        for (rect, color) in preview_blocks(rustomino, panel, game_over) {
            looks.push(block(rect, color));
        }
    };
    if assists {
        preview(&board.held, &view.hold_rect);
    }
    preview(&board.next, &view.preview_rect);

    // the ghost, over the blocks
    if let Some(ghost) = playfield.ghost_rustomino.as_ref().filter(|_| assists) {
        let color = block_color(ghost.rtype, game_over, PREVIEW_ROW);
        let slots = ghost.playfield_slots();
        let (fill, outline) = (
            with_alpha(color, GHOST_FILL_ALPHA),
            with_alpha(color, GHOST_OUTLINE_ALPHA),
        );
        let mut layers = Vec::new();
        for slot in &slots {
            let rect = playfield_block_rect([slot.x, slot.y], staging_rect, playfield_rect);
            if style == BlockStyle::Jelly {
                // rounded cells like the blocks
                layers.push((rect, fill, Shape::Rounded));
                layers.push((rect, outline, Shape::RoundedOutline));
            } else {
                layers.push((rect, fill, Shape::Square));
            }
        }
        if style != BlockStyle::Jelly {
            layers.extend(
                ghost_outline(&slots, staging_rect, playfield_rect)
                    .into_iter()
                    .map(|rect| (rect, outline, Shape::Square)),
            );
        }
        looks.push(Look { layers, z: GHOST_Z });
    }

    // the playfield flash, over everything on the board
    let flash = fx.flash();
    if flash > 0.0 {
        looks.push(Look {
            layers: vec![(
                view.playfield_rect,
                with_alpha(Color::WHITE, flash),
                Shape::Square,
            )],
            z: FLASH_Z,
        });
    }
    looks
}

/// looks for the trails, impacts and particles
fn effect_looks(
    view: Res<ViewSettings>,
    mut trails: Query<(&Trail, &mut Look), Without<Impact>>,
    mut impacts: Query<(&Impact, &mut Look), (Without<Trail>, Without<Particle>)>,
    mut particles: Query<(&Particle, &mut Look), (Without<Trail>, Without<Impact>)>,
) {
    let cell = BLOCK_SIZE + BLOCK_PADDING;
    let left = view.staging_rect.x + 1.0;
    let bottom = view.playfield_rect.y + view.playfield_rect.h - 1.0;
    for (trail, mut look) in &mut trails {
        let fade = trail.fade();
        let x = left + trail.x * cell;
        let length = trail.from - trail.to;
        let layers = (0..TRAIL_SEGMENTS)
            .map(|i| {
                let t = i as f32 / TRAIL_SEGMENTS as f32;
                let y0 = trail.to + length * t;
                let h = length / TRAIL_SEGMENTS as f32 * cell;
                let rect = snap_edges(Rect::new(x, bottom - y0 * cell - h, cell, h));
                let alpha = TRAIL_ALPHA * trail.strength * fade * (1.0 - t);
                (rect, with_alpha(trail.color, alpha), Shape::Square)
            })
            .collect();
        look.set_if_neq(Look { layers, z: TRAIL_Z });
    }
    for (impact, mut look) in &mut impacts {
        let color = with_alpha(Color::WHITE, IMPACT_ALPHA * impact.strength * impact.fade());
        let layers = impact
            .blocks
            .iter()
            .map(|block| {
                let rect = playfield_block_rect(
                    [block.x, block.y],
                    &view.staging_rect,
                    &view.playfield_rect,
                );
                (rect, color, Shape::Square)
            })
            .collect();
        look.set_if_neq(Look {
            layers,
            z: IMPACT_Z,
        });
    }
    for (particle, mut look) in &mut particles {
        let layers = if particle.visible() {
            let fade = particle.fade();
            let size = BLOCK_SIZE * particle.size * (0.4 + 0.6 * fade);
            let center = Vec2::new(left + particle.pos.x * cell, bottom - particle.pos.y * cell);
            let rect = Rect::new(center.x - size / 2.0, center.y - size / 2.0, size, size);
            vec![(
                snap_rect(rect),
                with_alpha(particle.color, fade),
                Shape::Square,
            )]
        } else {
            Vec::new()
        };
        look.set_if_neq(Look {
            layers,
            z: PARTICLE_Z,
        });
    }
}

/// child sprites for changed looks, or all of them when the view changes
fn layout_looks(
    mut commands: Commands,
    view: Res<ViewSettings>,
    images: Res<ShapeImages>,
    looks: Query<(Entity, Ref<Look>, Option<&Children>)>,
    mut layers: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<LookLayer>>,
) {
    let size = Vec2::new(view.view_rect.w, view.view_rect.h) * view.scale;
    for (entity, look, children) in &looks {
        if !look.is_changed() && !view.is_changed() {
            continue;
        }
        let children: Vec<Entity> = children
            .map(|c| c.iter().filter(|e| layers.contains(*e)).collect())
            .unwrap_or_default();
        for (i, &(rect, color, shape)) in look.layers.iter().enumerate() {
            let sprite = Sprite {
                image: match shape {
                    Shape::Square => Handle::default(),
                    Shape::Rounded => images.rounded.clone(),
                    Shape::RoundedOutline => images.outline.clone(),
                },
                color: bevy::color::Color::srgba(color.r, color.g, color.b, color.a),
                custom_size: Some(Vec2::new(rect.w, rect.h) * view.scale),
                ..default()
            };
            // view units, y down from the top left, to pixels, y up from the center
            let center = rect.center() * view.scale;
            let translation = Vec3::new(
                center.x - size.x / 2.0,
                size.y / 2.0 - center.y,
                look.z + i as f32 * LAYER_DEPTH,
            );
            let visible = rect.w > 0.0 && rect.h > 0.0 && color.a > 0.0;
            let visibility = if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            match children
                .get(i)
                .and_then(|&child| layers.get_mut(child).ok())
            {
                Some((mut current, mut transform, mut vis)) => {
                    *current = sprite;
                    transform.translation = translation;
                    vis.set_if_neq(visibility);
                }
                None => {
                    commands.spawn((
                        LookLayer,
                        sprite,
                        Transform::from_translation(translation),
                        visibility,
                        ChildOf(entity),
                    ));
                }
            }
        }
        for &child in children.iter().skip(look.layers.len()) {
            if let Ok((_, _, mut vis)) = layers.get_mut(child) {
                vis.set_if_neq(Visibility::Hidden);
            }
        }
    }
}

/// blocks of a piece centered in a preview panel
fn preview_blocks(
    rustomino: &Option<crate::rustomino::Rustomino>,
    panel: &Rect,
    game_over: Option<&crate::effects::GameOver>,
) -> Vec<(Rect, Color)> {
    let Some(rustomino) = rustomino else {
        return Vec::new();
    };
    // center the piece's bounding box in the panel
    let cell = BLOCK_SIZE + BLOCK_PADDING;
    let min = rustomino
        .blocks
        .iter()
        .fold(rustomino.blocks[0], |a, b| a.min(*b));
    let max = rustomino
        .blocks
        .iter()
        .fold(rustomino.blocks[0], |a, b| a.max(*b));
    let width = (max.x - min.x + 1) as f32 * cell - BLOCK_PADDING;
    let height = (max.y - min.y + 1) as f32 * cell - BLOCK_PADDING;
    let left = snap(panel.x + (panel.w - width) / 2.0);
    let top = snap(panel.y + (panel.h - height) / 2.0);
    let color = block_color(rustomino.rtype, game_over, PREVIEW_ROW);
    rustomino
        .blocks
        .iter()
        .map(|block: &IVec2| {
            let rect = Rect::new(
                left + (block.x - min.x) as f32 * cell,
                top + (max.y - block.y) as f32 * cell,
                block_size(),
                block_size(),
            );
            (rect, color)
        })
        .collect()
}
