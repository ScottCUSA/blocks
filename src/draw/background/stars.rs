use crate::gfx::Vec2;
use crate::gfx::{BlendMode, Color, DrawParam, Rect};

use crate::background::{Background, STAR_FIELDS};
use crate::rustomino::RustominoType;
use strum::IntoEnumIterator;

use super::*;
use crate::draw::*;

pub(super) const STAR_COUNT: usize = 400;
pub(super) const STAR_FLIGHT: f32 = 0.06; // depth per second
pub(super) const STAR_NEAR: f32 = 0.02;
pub(super) const STAR_SPREAD: f32 = 0.5; // of the view at full depth
pub(super) const NEBULA_CLOUDS: usize = 4;
pub(super) const NEBULA_BLOBS: usize = 7; // per cloud
pub(super) const NEBULA_ALPHA: f32 = 0.09;
pub(super) const NEBULA_DRIFT: f32 = 0.05;
pub(super) const NEBULA_COLORS: [Color; 5] = [
    Color::new(0.55, 0.25, 0.85, 1.0), // violet
    Color::new(0.85, 0.3, 0.6, 1.0),   // rose
    Color::new(0.25, 0.4, 0.95, 1.0),  // indigo
    Color::new(0.2, 0.75, 0.8, 1.0),   // teal
    Color::new(0.9, 0.55, 0.35, 1.0),  // amber
];
pub(super) const STARS_STILL_TWINKLE: f32 = 0.5; // twinkle depth once stopped
pub(super) const STARS_SPARKLE: f32 = 0.3; // extra brightness at the peak
pub(super) const STARS_SPARKLE_CHANCE: f32 = 0.05;
pub(super) const STARS_SPARKLE_PERIOD: f32 = 3.0; // seconds
pub(super) const STARS_SPARKLE_TIME: f32 = 0.5;
pub(super) const STAR_COLOR: Color = Color::new(0.8, 0.9, 1.0, 1.0);
pub(super) const WARP_STREAK: f32 = 0.4; // flight per px from center
pub(super) const WARP_ARRIVE: f32 = 0.35; // new stars start appearing
pub(super) const WARP_DOTS: usize = 48; // max per streak

pub(super) fn overlaps(a: Rect, b: &Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

pub(super) struct Star {
    pos: Vec2,
    size: f32,
    depth: f32, // 0.0 far to 1.0 near
    alpha: f32,
    color: Color,
}

/// star i of a field, flying toward the camera
pub(super) fn star_of(i: usize, field: usize, background: &Background, view_rect: &Rect) -> Star {
    let n = |k: f32| hash(i, k + field as f32 * 10.0);
    let time = background.time;
    let z = (n(1.0) - time * STAR_FLIGHT).rem_euclid(1.0).max(STAR_NEAR);
    let depth = 1.0 - z;
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let spread = Vec2::new(n(2.0) * 2.0 - 1.0, n(3.0) * 2.0 - 1.0) * STAR_SPREAD / z;
    let pos = center + spread * Vec2::new(view_rect.w, view_rect.h) / 2.0;
    let size = (1.0 + 3.0 * depth * depth).round();
    // twinkle deepens and some stars sparkle once stopped
    let chill = background.chill();
    let clock = background.twinkle;
    let wave = (clock * (2.0 + 3.0 * n(4.0)) + n(5.0) * std::f32::consts::TAU).sin();
    let swing = 0.4 + (STARS_STILL_TWINKLE - 0.4) * chill;
    let twinkle = 1.0 - swing + swing * wave;
    let sparkle = if n(7.0) < STARS_SPARKLE_CHANCE {
        let at = (clock / STARS_SPARKLE_PERIOD + n(8.0)).fract();
        (1.0 - (at * STARS_SPARKLE_PERIOD / STARS_SPARKLE_TIME).min(1.0)).powi(2) * chill
    } else {
        0.0
    };
    // fade in far away
    let arrive = smoothstep(1.0, 0.8, z);
    let alpha = ((0.15 + 0.45 * depth) * twinkle * (1.0 + background.energy)
        + STARS_SPARKLE * sparkle)
        .min(1.0)
        * arrive;
    let size = size + (sparkle * STARS_SPARKLE * 2.0).round();
    let color = if n(6.0) < 0.15 {
        RustominoType::iter()
            .nth(i % 7)
            .map_or(STAR_COLOR, |t| t.color())
    } else {
        STAR_COLOR
    };
    Star {
        pos: pos - Vec2::splat(size / 2.0),
        size,
        depth,
        alpha,
        color,
    }
}

pub(super) struct Stars;

impl Scenery for Stars {
    const COLORS: (Color, Color) = STARS_COLORS;

    fn draw(
        &self,
        canvas: &mut Painter,
        meshes: &Meshes,
        view_rect: &Rect,
        background: &Background,
    ) {
        draw_stars(canvas, meshes, view_rect, background);
    }
}

fn draw_stars(canvas: &mut Painter, meshes: &Meshes, view_rect: &Rect, background: &Background) {
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let field = background.stars;
    let Some(p) = background.wave() else {
        draw_nebula(canvas, meshes, view_rect, background, field, 1.0, 1.0);
        for i in 0..STAR_COUNT {
            let star = star_of(i, field, background, view_rect);
            let rect = Rect::new(star.pos.x, star.pos.y, star.size, star.size);
            if star.alpha > 0.0 && overlaps(rect, view_rect) {
                draw_rect(canvas, meshes, rect, with_alpha(star.color, star.alpha));
            }
        }
        return;
    };
    let old = (field + STAR_FIELDS - 1) % STAR_FIELDS;
    let appear = smoothstep(WARP_ARRIVE, 1.0, p);
    // old nebula rushes past, the next fades in behind it
    let leave = 1.0 - smoothstep(0.1, 0.7, p);
    draw_nebula(
        canvas,
        meshes,
        view_rect,
        background,
        old,
        leave,
        1.0 + smoothstep(0.0, 0.7, p),
    );
    draw_nebula(
        canvas,
        meshes,
        view_rect,
        background,
        field,
        appear,
        0.8 + 0.2 * appear,
    );
    for i in 0..STAR_COUNT {
        // old field streaks out toward the camera
        let star = star_of(i, old, background, view_rect);
        draw_warp_star(canvas, meshes, view_rect, center, &star, p);
        // new field arrives from behind, small and dim, spreading into place
        if appear <= 0.0 {
            continue;
        }
        let star = star_of(i, field, background, view_rect);
        let pos = center + (star.pos - center) * (0.7 + 0.3 * appear);
        let size = (star.size * appear).round().max(1.0);
        let rect = Rect::new(pos.x, pos.y, size, size);
        if overlaps(rect, view_rect) {
            draw_rect(
                canvas,
                meshes,
                rect,
                with_alpha(star.color, star.alpha * appear),
            );
        }
    }
}

/// soft clouds behind a star field, drawn additively
pub(super) fn draw_nebula(
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    background: &Background,
    field: usize,
    alpha: f32,
    zoom: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let view = Vec2::new(view_rect.w, view_rect.h);
    let span = view_rect.w.max(view_rect.h);
    let drift = background.time * NEBULA_DRIFT;
    set_blend_mode(canvas, BlendMode::Add);
    for cloud in 0..NEBULA_CLOUDS {
        let n = |k: f32| hash(cloud, k + field as f32 * 10.0 + 500.0);
        let hue = NEBULA_COLORS[(n(1.0) * NEBULA_COLORS.len() as f32) as usize];
        let edge = NEBULA_COLORS[(n(2.0) * NEBULA_COLORS.len() as f32) as usize];
        let at = Vec2::new(n(3.0) - 0.5, n(4.0) - 0.5) * view * 1.1;
        let size = span * (0.18 + 0.12 * n(5.0));
        for blob in 0..NEBULA_BLOBS {
            let m = |k: f32| hash(cloud * NEBULA_BLOBS + blob, k + field as f32 * 10.0 + 700.0);
            let angle = m(1.0) * std::f32::consts::TAU;
            let out = Vec2::new(angle.cos(), angle.sin()) * size * m(2.0);
            // slow swirl
            let phase = m(3.0) * std::f32::consts::TAU;
            let sway = Vec2::new((drift + phase).sin(), (drift * 0.7 + phase).cos()) * size * 0.08;
            let pos = center + (at + out + sway) * zoom;
            let radius = size * (0.5 + 0.6 * m(4.0)) * zoom;
            let stretch = 0.45 + 0.55 * m(5.0);
            let color = mix_color(hue, edge, m(6.0));
            let strength = NEBULA_ALPHA * (0.5 + 0.5 * m(7.0)) * alpha;
            draw_mesh(
                canvas,
                &meshes.glow,
                DrawParam::default()
                    .dest(pos)
                    .rotation(m(8.0) * std::f32::consts::PI + drift * 0.1)
                    .scale(Vec2::new(radius, radius * stretch))
                    .color(with_alpha(color, strength)),
            );
        }
    }
    set_blend_mode(canvas, BlendMode::Alpha);
}

/// a star streaking out from the center, head first, tail catching up
pub(super) fn draw_warp_star(
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    center: Vec2,
    star: &Star,
    p: f32,
) {
    // streaks only head outward, so off screen stars stay off
    if !overlaps(
        Rect::new(star.pos.x, star.pos.y, star.size, star.size),
        view_rect,
    ) {
        return;
    }
    let reach = (star.pos - center) * WARP_STREAK * star.depth;
    let head = reach * smoothstep(0.0, 0.45, p);
    let tail = reach * smoothstep(0.2, 0.65, p);
    // grows as it nears
    let size = (star.size * (1.0 + smoothstep(0.0, 0.65, p))).round();
    let length = (head - tail).length();
    let dots = ((length / size) as usize).clamp(1, WARP_DOTS);
    let bright =
        (star.alpha * (1.0 + smoothstep(0.0, 0.3, p))).min(1.0) * (1.0 - smoothstep(0.4, 0.65, p));
    for d in 0..=dots {
        let f = d as f32 / dots as f32;
        let at = star.pos + tail + (head - tail) * f;
        let dot = Rect::new(at.x, at.y, size, size);
        if overlaps(dot, view_rect) {
            let fade = if dots == 1 { 1.0 } else { 0.3 + 0.7 * f };
            draw_rect(canvas, meshes, dot, with_alpha(star.color, bright * fade));
        }
    }
}
