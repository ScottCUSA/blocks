// title screen backdrops tried and set aside: a perspective tile floor lit on the beat, and a glitch
// not compiled; was src/draw/background/title.rs
// wiring it used:
// - draw_title_backdrop in place of draw_background on the title screens, clear color title_clear_color
// - draw_backdrop_title in place of draw_title on the main menu
// - time: a clock advanced by dt in Background::update
// - beat: title theme position in beats, audio::Track::position (player.get_pos()) * 150 bpm / 60

use ggez::glam::Vec2;
use ggez::graphics::{self, Color, Rect};
use ggez::{Context, GameResult};

use crate::rustomino::RustominoType;
use strum::IntoEnumIterator;

use super::*;
use crate::draw::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TitleBackdrop {
    Floor,
    Glitch,
    Game, // the in-game background, calmed
}

impl TitleBackdrop {
    pub const ALL: [TitleBackdrop; 3] = [
        TitleBackdrop::Floor,
        TitleBackdrop::Glitch,
        TitleBackdrop::Game,
    ];

    pub fn name(self) -> &'static str {
        match self {
            TitleBackdrop::Floor => "Floor",
            TitleBackdrop::Glitch => "Glitch",
            TitleBackdrop::Game => "Game",
        }
    }
}

pub(super) const TITLE_COLORS: (Color, Color) = (
    Color::new(0.01, 0.015, 0.04, 1.0),
    Color::new(0.05, 0.05, 0.12, 1.0),
);

// floor
const FLOOR_HORIZON: f32 = 0.55; // of the view height
const FLOOR_EYE: f32 = 3.5; // camera height, in tiles; tiles look square at this depth
const FLOOR_NEAR: f32 = 2.5; // depth at the bottom of the view, in tiles
const FLOOR_ROWS: usize = 24;
const FLOOR_SCROLL: f32 = 0.5; // rows per second
const FLOOR_GAP: f32 = 0.12; // of a tile
const FLOOR_DEPTH: f32 = 0.4; // a tile's depth, of its width
const FLOOR_LIT: f32 = 0.1; // chance a tile lights on a beat
const FLOOR_COLOR: Color = Color::new(0.12, 0.1, 0.24, 1.0);

// glitch
const GLITCH_RATE: f32 = 12.0; // noise steps per second
const GLITCH_CHANCE: f32 = 0.3; // chance a beat bursts
const GLITCH_BURST: f32 = 0.35; // of a beat
const GLITCH_SCANLINE: f32 = 4.0; // px
const GLITCH_BLOCKS: usize = 40;
const GLITCH_BANDS: usize = 10; // during a burst
const GLITCH_MAGENTA: Color = Color::new(1.0, 0.2, 0.75, 1.0);
const GLITCH_CYAN: Color = Color::new(0.2, 0.95, 1.0, 1.0);
const GLITCH_GLYPHS: [char; 7] = ['#', '%', '&', '0', '1', '/', '?'];

// titles
const FLOOR_HOP: f32 = 0.2; // of the title size
const GLITCH_SPLIT: f32 = 3.0; // px, rgb split at rest

fn palette() -> Vec<Color> {
    RustominoType::iter().map(|t| t.color()).collect()
}

fn gradient(canvas: &mut Painter, meshes: &Meshes, view_rect: &Rect) {
    let (top, bottom) = TITLE_COLORS;
    draw_rect(canvas, meshes, *view_rect, bottom);
    draw_mesh(
        canvas,
        &meshes.background,
        graphics::DrawParam::default()
            .dest_rect(*view_rect)
            .color(top),
    );
}

/// the backdrop behind the title screens, beat in beats of the title theme
pub fn draw_title_backdrop(
    ctx: &Context,
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    backdrop: TitleBackdrop,
    time: f32,
    beat: f32,
) {
    gradient(canvas, meshes, view_rect);
    match backdrop {
        TitleBackdrop::Game => {}
        TitleBackdrop::Floor => draw_floor(ctx, canvas, meshes, view_rect, time, beat),
        TitleBackdrop::Glitch => draw_glitch(canvas, meshes, view_rect, time, beat),
    }
}

/// a tile floor in perspective, tiles light up on the beat
fn draw_floor(
    ctx: &Context,
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    time: f32,
    beat: f32,
) {
    let colors = palette();
    let horizon = view_rect.y + view_rect.h * FLOOR_HORIZON;
    let bottom = view_rect.y + view_rect.h;
    let center = view_rect.x + view_rect.w / 2.0;
    let tile = (bottom - horizon) * FLOOR_NEAR / FLOOR_EYE; // a tile's width at depth 1
                                                            // horizon glow
    set_blend_mode(canvas, graphics::BlendMode::ADD);
    draw_mesh(
        canvas,
        &meshes.glow,
        graphics::DrawParam::default()
            .dest(Vec2::new(center, horizon))
            .scale(Vec2::new(view_rect.w * 0.7, view_rect.h * 0.12))
            .color(with_alpha(colors[1], 0.18)),
    );
    set_blend_mode(canvas, graphics::BlendMode::ALPHA);
    // floor point to screen, x across and z into the view, in tiles
    let project = |x: f32, z: f32| Vec2::new(center + x * tile / z, horizon + tile * FLOOR_EYE / z);
    let z_min = FLOOR_NEAR * 0.9; // just below the view
    let reach = FLOOR_ROWS as f32 * FLOOR_DEPTH;
    let fade = |z: f32| 1.0 - smoothstep(FLOOR_NEAR + reach * 0.3, FLOOR_NEAR + reach, z);
    let scroll = time * FLOOR_SCROLL;
    let beat_index = beat.floor().max(0.0) as usize;
    let pulse = (1.0 - beat.fract()).powi(3);
    let half_gap = FLOOR_GAP / 2.0;
    let gap_z = half_gap * FLOOR_DEPTH;
    let mut vertices = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let first = scroll.floor() as i32;
    for k in 0..FLOOR_ROWS as i32 {
        let row = first + k;
        let at = (row as f32 - scroll) * FLOOR_DEPTH + z_min;
        let near = (at - FLOOR_DEPTH + gap_z).max(z_min);
        let far = at - gap_z;
        if far <= near {
            continue;
        }
        // enough columns to reach the sides at the far edge
        let cols = (far * view_rect.w / 2.0 / tile).ceil() as i32 + 1;
        for col in -cols..cols {
            let id = (row as usize).wrapping_mul(997) + (col + 512) as usize;
            let lit = hash(id, beat_index as f32 * 0.37) < FLOOR_LIT;
            let color = if lit {
                let hue = colors[(hash(id, 3.0) * colors.len() as f32) as usize];
                mix_color(FLOOR_COLOR, hue, 0.25 + 0.6 * pulse)
            } else {
                // the whole floor breathes on the beat
                tint(FLOOR_COLOR, 0.04 * pulse)
            };
            let (x0, x1) = (col as f32 + half_gap, col as f32 + 1.0 - half_gap);
            let base = vertices.len() as u32;
            for (x, z) in [(x0, near), (x1, near), (x1, far), (x0, far)] {
                let at = project(x, z);
                let c = with_alpha(color, fade(z));
                vertices.push(graphics::Vertex {
                    position: [at.x, at.y],
                    uv: [0.0, 0.0],
                    color: [c.r, c.g, c.b, c.a],
                });
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    if indices.is_empty() {
        return;
    }
    let mesh = graphics::Mesh::from_data(
        ctx,
        graphics::MeshData {
            vertices: &vertices,
            indices: &indices,
        },
    );
    draw(canvas, &mesh, graphics::DrawParam::default());
}

/// glitch burst strength, 0.0 to 1.0, on some beats
fn glitch_burst(beat: f32) -> f32 {
    let index = beat.floor().max(0.0) as usize;
    if hash(index, 1700.0) >= GLITCH_CHANCE {
        return 0.0;
    }
    1.0 - smoothstep(0.0, GLITCH_BURST, beat.fract())
}

/// scanlines, corrupt blocks, and torn bands on the beat
fn draw_glitch(canvas: &mut Painter, meshes: &Meshes, view_rect: &Rect, time: f32, beat: f32) {
    let colors = palette();
    let burst = glitch_burst(beat);
    let step = (time * GLITCH_RATE).floor() as usize;
    let (w, h) = (view_rect.w, view_rect.h);
    let pick = |r: f32| match (r * 4.0) as usize {
        0 => GLITCH_MAGENTA,
        1 => GLITCH_CYAN,
        _ => colors[(r * 97.0) as usize % colors.len()],
    };
    // rolling bar
    let roll = (time * 0.07).fract();
    draw_rect(
        canvas,
        meshes,
        Rect::new(
            view_rect.x,
            view_rect.y + roll * (h + 120.0) - 120.0,
            w,
            120.0,
        ),
        with_alpha(GLITCH_CYAN, 0.025),
    );
    // corrupt blocks, snapped to an 8 px grid
    let count = GLITCH_BLOCKS + (burst * GLITCH_BLOCKS as f32 * 2.0) as usize;
    for i in 0..count {
        // each holds for a few steps
        let hold = 2 + i % 5;
        let id = i * 131 + step / hold;
        let n = |k: f32| hash(id, k + 1900.0);
        if n(1.0) < 0.4 - 0.4 * burst {
            continue;
        }
        let bw = 8.0 * (1.0 + (n(2.0) * 12.0).floor());
        let bh = 8.0 * (1.0 + (n(3.0) * 3.0).floor());
        let x = view_rect.x + ((n(4.0) * w) / 8.0).floor() * 8.0;
        let y = view_rect.y + ((n(5.0) * h) / 8.0).floor() * 8.0;
        let alpha = 0.04 + 0.05 * n(6.0) + 0.25 * burst;
        draw_rect(
            canvas,
            meshes,
            Rect::new(x, y, bw, bh),
            with_alpha(pick(n(7.0)), alpha),
        );
    }
    // torn bands during a burst
    if burst > 0.0 {
        for i in 0..GLITCH_BANDS {
            let n = |k: f32| hash(i * 17 + step, k + 2100.0);
            if n(1.0) > burst {
                continue;
            }
            let y = view_rect.y + n(2.0) * h;
            let bh = 2.0 + n(3.0) * 24.0;
            let shift = (n(4.0) - 0.5) * w * 0.3;
            let color = if n(5.0) < 0.5 {
                GLITCH_MAGENTA
            } else {
                GLITCH_CYAN
            };
            set_blend_mode(canvas, graphics::BlendMode::ADD);
            draw_rect(
                canvas,
                meshes,
                Rect::new(view_rect.x + shift, y, w, bh),
                with_alpha(color, 0.12 * burst),
            );
            set_blend_mode(canvas, graphics::BlendMode::ALPHA);
        }
    }
    // scanlines over everything
    let mut y = view_rect.y;
    while y < view_rect.y + h {
        draw_rect(
            canvas,
            meshes,
            Rect::new(view_rect.x, y, w, GLITCH_SCANLINE / 2.0),
            Color::new(0.0, 0.0, 0.0, 0.18),
        );
        y += GLITCH_SCANLINE;
    }
}

/// one letter of a title
struct Letter {
    offset: Vec2,
    color: Color,
    glyph: char,
    scale: f32,
}

/// a title drawn letter by letter, centered
fn draw_letters(
    ctx: &Context,
    canvas: &mut Painter,
    title: &str,
    size: f32,
    center: Vec2,
    shadow: bool,
    letter: impl Fn(usize, char) -> Letter,
) -> GameResult {
    let dims = ui_text(title, size).measure_ui(ctx)?;
    let left = center - dims / 2.0;
    let param = text_param();
    let drop = Vec2::splat((size / 16.0).max(2.0));
    for (i, (start, c)) in title.char_indices().enumerate() {
        let x = ui_text(&title[..start], size).measure_ui(ctx)?.x;
        let l = letter(i, c);
        let base = ui_text(&c.to_string(), size).measure_ui(ctx)?;
        let text = ui_text(&l.glyph.to_string(), size * l.scale);
        let actual = text.measure_ui(ctx)?;
        let pos = left + Vec2::new(x, 0.0) + (base - actual) / 2.0 + l.offset;
        if shadow {
            let shade = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * l.color.a);
            draw(canvas, &text, param.dest(pos + drop).color(shade));
        }
        draw(canvas, &text, param.dest(pos).color(premultiply(l.color)));
    }
    Ok(())
}

/// the main menu title, styled to match the backdrop
#[allow(clippy::too_many_arguments)]
pub fn draw_backdrop_title(
    ctx: &Context,
    canvas: &mut Painter,
    backdrop: TitleBackdrop,
    beat: f32,
    title: &str,
    center: Vec2,
    size: f32,
) -> GameResult {
    let colors: Vec<Color> = palette().into_iter().map(|c| tint(c, TITLE_TINT)).collect();
    let time = ctx.time.time_since_start().as_secs_f32();
    let plain = |i: usize, c: char| Letter {
        offset: Vec2::ZERO,
        color: colors[i % colors.len()],
        glyph: c,
        scale: 1.0,
    };
    let count = title.chars().count();
    match backdrop {
        TitleBackdrop::Game => draw_letters(ctx, canvas, title, size, center, true, plain),
        TitleBackdrop::Floor => {
            // a letter hops on each beat, the rest pulse
            let pulse = (1.0 - beat.fract()).powi(3);
            let hop = beat.floor().max(0.0) as usize % count;
            draw_letters(ctx, canvas, title, size, center, true, |i, c| {
                if i == hop {
                    let up = (beat.fract() * std::f32::consts::PI).sin();
                    Letter {
                        offset: Vec2::new(0.0, -up * size * FLOOR_HOP),
                        color: tint(colors[i % colors.len()], 0.3 * pulse),
                        ..plain(i, c)
                    }
                } else {
                    Letter {
                        scale: 1.0 + 0.04 * pulse,
                        ..plain(i, c)
                    }
                }
            })
        }
        TitleBackdrop::Glitch => {
            let burst = glitch_burst(beat);
            let step = (time * GLITCH_RATE).floor() as usize;
            let glitched = |i: usize| {
                let n = |k: f32| hash(i * 31 + step, k + 2300.0);
                (n(1.0) < 0.03 + 0.5 * burst).then(|| {
                    (
                        Vec2::new((n(2.0) - 0.5) * size * 0.3, (n(3.0) - 0.5) * size * 0.1),
                        (n(4.0) < 0.5).then(|| GLITCH_GLYPHS[(n(5.0) * 7.0) as usize % 7]),
                    )
                })
            };
            let letter = |i: usize, c: char| match glitched(i) {
                Some((offset, glyph)) => Letter {
                    offset,
                    glyph: glyph.unwrap_or(c),
                    ..plain(i, c)
                },
                None => plain(i, c),
            };
            // rgb split, wider in a burst
            let split = GLITCH_SPLIT + 10.0 * burst;
            set_blend_mode(canvas, graphics::BlendMode::ADD);
            for (side, color) in [(-1.0, GLITCH_MAGENTA), (1.0, GLITCH_CYAN)] {
                let at = center + Vec2::new(side * split, 0.0);
                draw_letters(ctx, canvas, title, size, at, false, |i, c| Letter {
                    color: with_alpha(color, 0.55),
                    ..letter(i, c)
                })?;
            }
            set_blend_mode(canvas, graphics::BlendMode::ALPHA);
            draw_letters(ctx, canvas, title, size, center, true, letter)
        }
    }
}

/// window clear color behind the title screens
pub fn title_clear_color() -> Color {
    TITLE_COLORS.1
}

/// which backdrop is showing, while choosing one
pub fn draw_backdrop_label(
    ctx: &Context,
    canvas: &mut Painter,
    backdrop: TitleBackdrop,
    view_rect: &Rect,
) -> GameResult {
    let label = format!("Backdrop: {} (Tab)", backdrop.name());
    let text = ui_text(&label, SMALL_TEXT_SIZE);
    let size = text.measure_ui(ctx)?;
    draw_text_centered(
        ctx,
        canvas,
        &text,
        Vec2::new(view_rect.x + 16.0 + size.x / 2.0, view_rect.y + 20.0),
        LABEL_COLOR,
    )
}
