use ggez::glam::Vec2;
use ggez::graphics::{self, Color, Rect};

use crate::background::Background;

use super::*;
use crate::draw::*;

pub(super) const SEA_LEVEL: f32 = 0.75; // sea fills the view below this
pub(super) const SEA_STRIP: f32 = 6.0; // px per wave column
pub(super) const SEA_LAYERS: [(Color, f32, f32); 3] = [
    // color, depth into the sea, wave height in px
    (Color::new(0.09, 0.15, 0.2, 1.0), 0.0, 10.0),
    (Color::new(0.05, 0.1, 0.15, 1.0), 0.22, 16.0),
    (Color::new(0.025, 0.055, 0.09, 1.0), 0.5, 24.0),
];
pub(super) const SEA_SPEED: f32 = 0.9;
pub(super) const CALM_SEA_DROP: f32 = 100.0; // px past the bottom, clears the tallest waves
pub(super) const CALM_RAIN: f32 = 0.5; // drops lost when calm
pub(super) const SEA_SURGE: f32 = 1.2; // extra wave height at full energy
pub(super) const CREST_COLOR: Color = Color::new(0.55, 0.68, 0.78, 1.0);
pub(super) const FOAM_SLOPE: f32 = 0.9; // steeper crests break white
pub(super) const RAIN_DROPS: (usize, usize) = (260, 800); // calm to full storm, most of them up front
pub(super) const RAIN_FALL: f32 = 0.55; // views per second
pub(super) const RAIN_SLANT: (f32, f32) = (0.18, 0.75); // px sideways per px down, calm to full storm
pub(super) const GUST: f32 = 0.2; // extra slant in gusts at full storm
pub(super) const STORM_WAVES: f32 = 1.4; // extra wave height at full storm
pub(super) const SPRAY: usize = 90; // blown spray at full storm
                                    // (share of the drops when calm, streak length px, width px, brightness, fall speed), far to near
pub(super) const RAIN_LAYERS: [(f32, f32, f32, f32, f32); 4] = [
    (0.4, 7.0, 1.0, 0.45, 0.6),
    (0.3, 12.0, 1.0, 0.7, 0.85),
    (0.2, 22.0, 2.0, 0.85, 1.2),
    (0.1, 48.0, 3.0, 0.75, 1.8),
];
// share of the drops in a whiteout: the rain closes in and the distance disappears
pub(super) const RAIN_SHARE_STORM: [f32; 4] = [0.05, 0.1, 0.35, 0.5];
pub(super) const RAIN_COLOR: Color = Color::new(0.6, 0.7, 0.85, 1.0);
pub(super) const SEA_SCALE: [f32; 3] = [0.6, 0.9, 1.3]; // splash and spray size per sea layer, far to near
pub(super) const WHITEOUT_BANKS: usize = 14;
pub(super) const WHITEOUT_VEIL: (f32, f32) = (0.22, 0.62); // how washed out the sky and the sea get at the peak
pub(super) const WHITEOUT_COLOR: Color = Color::new(0.42, 0.47, 0.54, 1.0); // storm gray, light text stays readable
pub(super) const LEVEL_UP_FLASH: f32 = 0.25; // of the level up, how long the opening flash takes to fade
pub(super) const LEVEL_UP_WHITE: f32 = 0.55; // brightest the sky gets at the opening flash
pub(super) const LEVEL_UP_BOLTS: usize = 6; // bolts in the level up barrage
pub(super) const LEVEL_UP_STAGGER: f32 = 0.11; // of the level up, between bolts
pub(super) const LEVEL_UP_BOLT_TIME: f32 = 0.22; // of the level up, how long each bolt lasts
pub(super) const LEVEL_UP_SWELL: f32 = 0.8; // extra heave as the sea steps up to the new level
pub(super) const SPLASHES: usize = 40;
pub(super) const CLOUD_BLOBS: usize = 26;
pub(super) const CLOUD_DRIFT: f32 = 0.012; // views per second
pub(super) const CLOUD_COLOR: Color = Color::new(0.01, 0.012, 0.02, 1.0);
pub(super) const STRIKE_PERIOD: f32 = 7.0; // seconds of background time
pub(super) const STRIKE_CHANCE: f32 = 0.75;
pub(super) const STRIKE_TIME: f32 = 0.6; // seconds a strike lasts
pub(super) const BOLT_SEGMENTS: usize = 16;
pub(super) const BOLT_COLOR: Color = Color::new(0.85, 0.9, 1.0, 1.0);
pub(super) const BOLT_GLOW_COLOR: Color = Color::new(0.55, 0.65, 1.0, 1.0);
pub(super) const BOLT_BLOOM: [(f32, f32); 2] = [(14.0, 0.3), (4.0, 0.6)]; // (width in bolt sizes, strength)
pub(super) const FLASH_COLOR: Color = Color::new(0.75, 0.82, 1.0, 1.0);

/// lightning strike at a moment: (strike number, brightness 0.0 to 1.0)
pub(super) fn storm_strike(time: f32) -> Option<(usize, f32)> {
    let k = (time / STRIKE_PERIOD).floor().max(0.0) as usize;
    if hash(k, 21.0) > STRIKE_CHANCE {
        return None;
    }
    let at = (k as f32 + hash(k, 22.0) * 0.8) * STRIKE_PERIOD;
    let age = time - at;
    if !(0.0..STRIKE_TIME).contains(&age) {
        return None;
    }
    // bright, a quick dip, a second flicker, then gone
    let fade = (1.0 - age / STRIKE_TIME).powi(2);
    let flicker = 0.65 + 0.35 * (age * 45.0).cos();
    Some((k, (fade * flicker).clamp(0.0, 1.0)))
}

#[derive(Debug, Clone, Copy, PartialEq, strum::VariantArray)]
pub enum StrikeKind {
    Fork,    // one bolt from the clouds into the sea
    Double,  // two bolts at once, more common in a bigger storm
    Sheet,   // the clouds light up from inside, no bolt
    Crawl,   // a bolt crawling sideways along the cloud base
    Distant, // a small, dim bolt far out to sea
}

impl StrikeKind {
    /// how brightly the sky flashes
    fn flash(self) -> f32 {
        match self {
            StrikeKind::Fork => 1.0,
            StrikeKind::Double => 1.3,
            StrikeKind::Sheet => 1.5,
            StrikeKind::Crawl => 0.8,
            StrikeKind::Distant => 0.3,
        }
    }
}

/// which kind strike k is
pub(super) fn strike_kind(k: usize, storm: f32) -> StrikeKind {
    let r = hash(k, 30.0);
    let double = 0.08 + 0.17 * storm;
    let distant = 0.2 * (1.0 - storm) + 0.05;
    if r < double {
        StrikeKind::Double
    } else if r < double + distant {
        StrikeKind::Distant
    } else if r < double + distant + 0.2 {
        StrikeKind::Sheet
    } else if r < double + distant + 0.37 {
        StrikeKind::Crawl
    } else {
        StrikeKind::Fork
    }
}

/// lightning clock of the first bright strike of a kind, for screenshots
pub fn storm_strike_time(kind: StrikeKind, storm: f32) -> f32 {
    (0..100_000)
        .map(|i| i as f32 * 0.01)
        .find(|t| storm_strike(*t).is_some_and(|(k, b)| b > 0.85 && strike_kind(k, storm) == kind))
        .unwrap_or(0.0)
}

/// height of a sea layer at x, 0.0 is calm
pub(super) fn sea_wave(x: f32, layer: usize, time: f32) -> f32 {
    let t = time * SEA_SPEED;
    let l = layer as f32 * 1.7;
    0.55 * (x * 0.011 + t * 1.3 + l).sin()
        + 0.3 * (x * 0.027 - t * 0.9 + l * 2.0).sin()
        + 0.15 * (x * 0.061 + t * 2.1 - l).sin()
}

pub(super) fn draw_storm(canvas: &mut Painter, view_rect: &Rect, background: &Background) {
    let (x0, w, h) = (view_rect.x, view_rect.w, view_rect.h);
    // calm, the sea sinks out of view
    let calm = background.calm;
    let sea_top = view_rect.y + h * SEA_LEVEL + calm * (h * (1.0 - SEA_LEVEL) + CALM_SEA_DROP);
    let sea_h = view_rect.y + h - sea_top;
    let strike = storm_strike(background.bolt)
        .map(|(k, bright)| (k, bright * (1.0 - calm), strike_kind(k, background.storm)))
        .filter(|(_, bright, _)| *bright > 0.0);
    // line clears light up the clouds too
    // level up: a white-out flash at the start, then a barrage of strikes
    let level_up = background.wave();
    let level_flash = level_up.map_or(0.0, |p| {
        let burst = 1.0 - smoothstep(0.0, LEVEL_UP_FLASH, p);
        // a second, smaller flash as the last bolt lands
        let echo = (1.0 - ((p - 0.7) / 0.12).abs()).max(0.0) * 0.5;
        burst.max(echo)
    });
    let flash = strike
        .map_or(0.0, |(_, b, kind)| (b * kind.flash()).min(1.0))
        .max(background.energy * 0.7)
        .max(level_flash);

    // heavy clouds rolling across the top
    for i in 0..CLOUD_BLOBS {
        let n = |k: f32| hash(i, k + 300.0);
        let x = x0
            + ((n(1.0) - background.wind * CLOUD_DRIFT * (0.6 + 0.4 * n(2.0))).rem_euclid(1.0)
                * 1.4
                - 0.2)
                * w;
        let y = view_rect.y + h * (0.02 + 0.22 * n(3.0));
        let radius = w * (0.12 + 0.1 * n(4.0));
        draw_glow(
            canvas,
            Vec2::new(x, y),
            Vec2::new(radius, radius * (0.35 + 0.2 * n(5.0))),
            0.0,
            with_alpha(CLOUD_COLOR, 0.55 + 0.3 * n(6.0)),
        );
    }

    // sky flash and the bolt
    if flash > 0.0 {
        draw_rect(canvas, *view_rect, with_alpha(FLASH_COLOR, 0.12 * flash));
    }
    if let Some(p) = level_up {
        draw_level_up_storm(canvas, view_rect, background, p, sea_top, level_flash);
    }
    if let Some((k, bright, kind)) = strike {
        let cloud_base = view_rect.y + h * 0.18;
        let start_x = |seed: usize| x0 + w * (0.12 + 0.76 * hash(seed, 23.0));
        set_blend_mode(canvas, graphics::BlendMode::ADD);
        match kind {
            StrikeKind::Fork => {
                draw_fork(canvas, k, start_x(k), cloud_base, sea_top, w, bright);
            }
            StrikeKind::Double => {
                let first = start_x(k);
                // keep the second bolt well away from the first
                let second = x0
                    + (first - x0 + w * (0.3 + 0.4 * hash(k, 31.0))).rem_euclid(w * 0.8)
                    + w * 0.1;
                draw_fork(canvas, k, first, cloud_base, sea_top, w, bright);
                draw_fork(
                    canvas,
                    k + 7_919,
                    second,
                    cloud_base,
                    sea_top,
                    w,
                    bright * 0.85,
                );
            }
            StrikeKind::Sheet => {
                // pockets of cloud flare up one after another
                for p in 0..5 {
                    let pulse = (bright * (1.0 + 0.6 * (p as f32 * 1.7 + bright * 9.0).sin()))
                        .clamp(0.0, 1.0);
                    let at = Vec2::new(
                        x0 + w * (0.05 + 0.9 * hash(k * 13 + p, 32.0)),
                        view_rect.y + h * (0.05 + 0.18 * hash(k * 13 + p, 33.0)),
                    );
                    draw_glow(
                        canvas,
                        at,
                        Vec2::new(w * 0.22, w * 0.09),
                        0.0,
                        with_alpha(FLASH_COLOR, 0.4 * pulse),
                    );
                }
            }
            StrikeKind::Crawl => {
                // spider lightning along the cloud base, with a few short tendrils
                let y = view_rect.y + h * (0.1 + 0.1 * hash(k, 34.0));
                let from = Vec2::new(x0 + w * (0.05 + 0.3 * hash(k, 35.0)), y);
                let to = Vec2::new(
                    x0 + w * (0.6 + 0.35 * hash(k, 36.0)),
                    y + h * (hash(k, 37.0) - 0.5) * 0.08,
                );
                let path = draw_bolt(
                    canvas,
                    from,
                    to,
                    22,
                    2.5,
                    h * 0.03,
                    k,
                    with_alpha(BOLT_COLOR, bright),
                );
                for t in 0..3 {
                    let at = path[4 + t * 6];
                    let to = at
                        + Vec2::new(
                            (hash(k * 3 + t, 38.0) - 0.5) * w * 0.08,
                            h * (0.05 + 0.06 * hash(k * 3 + t, 39.0)),
                        );
                    draw_bolt(
                        canvas,
                        at,
                        to,
                        5,
                        1.5,
                        h * 0.012,
                        k * 3 + t,
                        with_alpha(BOLT_COLOR, bright * 0.5),
                    );
                }
            }
            StrikeKind::Distant => {
                // small and dim, low on the horizon
                let x = start_x(k);
                let from = Vec2::new(x, view_rect.y + h * 0.3);
                let to = Vec2::new(x + (hash(k, 40.0) - 0.5) * w * 0.05, sea_top);
                draw_bolt(
                    canvas,
                    from,
                    to,
                    9,
                    1.5,
                    w * 0.012,
                    k,
                    with_alpha(BOLT_COLOR, bright * 0.45),
                );
            }
        }
        set_blend_mode(canvas, graphics::BlendMode::ALPHA);
    }

    // three layers of rain, each falling into its own layer of sea:
    // far rain behind the back swell, near rain sweeping past in front
    let storm = background.storm;
    let gust = GUST * storm * (0.5 + 0.5 * (background.wind * 0.35).sin());
    let whiteout = background.whiteout;
    let slant = RAIN_SLANT.0 + (RAIN_SLANT.1 - RAIN_SLANT.0) * storm + gust + 0.4 * whiteout;
    let drops = ((RAIN_DROPS.0 + ((RAIN_DROPS.1 - RAIN_DROPS.0) as f32 * storm) as usize) as f32
        * (1.0 + 3.0 * whiteout)
        * (1.0 - CALM_RAIN * calm)) as usize;
    let rain_alpha =
        ((0.18 + 0.1 * storm + 0.15 * whiteout) + 0.12 * background.energy) * (1.0 + flash);

    // the sea, back layer to front, swelling with line clears and level ups
    // the sea follows the level on its own, stepping up as a level up plays out
    let sea_storm = background.sea_storm;
    let swell = (1.0 + STORM_WAVES * sea_storm)
        * (1.0 + SEA_SURGE * background.energy + LEVEL_UP_SWELL * background.rush());
    let sea = background.sea;
    let foam_slope = FOAM_SLOPE * (1.0 - 0.45 * sea_storm);
    let lit = flash * 0.35;
    let columns = (w / SEA_STRIP).ceil() as usize + 1;
    let surface = |x: f32, layer: usize| {
        let (_, depth, height) = SEA_LAYERS[layer];
        sea_top + sea_h * depth - sea_wave(x, layer, sea) * height * swell
    };
    let rain = RainView {
        view: *view_rect,
        slant,
        drops,
        alpha: rain_alpha,
        clock: background.rain,
        storm,
        forward: (0.7 * storm + 0.3 * whiteout).min(1.0),
    };
    for (layer, (color, depth, height)) in SEA_LAYERS.iter().enumerate() {
        // rain for this depth falls first, so the swell in front hides where it lands
        draw_rain_layer(canvas, &rain, layer, &|x| surface(x, layer));
        let base = sea_top + sea_h * depth;
        let color = mix_color(*color, FLASH_COLOR, lit * (1.0 - *depth));
        for c in 0..columns {
            let x = x0 + c as f32 * SEA_STRIP;
            let wave = sea_wave(x, layer, sea);
            let y = (base - wave * height * swell).round();
            draw_rect(
                canvas,
                Rect::new(x, y, SEA_STRIP, view_rect.y + h - y),
                color,
            );
            // crest line, breaking to foam where the wave is steep
            let slope =
                (sea_wave(x + SEA_STRIP, layer, sea) - wave).abs() * height * swell / SEA_STRIP;
            let foam = smoothstep(foam_slope * 0.5, foam_slope, slope);
            let crest = mix_color(color, CREST_COLOR, 0.25 + 0.5 * foam + lit);
            draw_rect(canvas, Rect::new(x, y, SEA_STRIP, 2.0 + 2.0 * foam), crest);
        }
        // rain bouncing off this swell, smaller and fainter further out
        let scale = SEA_SCALE[layer];
        let splashes = ((SPLASHES as f32 * (1.0 + storm)) * scale) as usize;
        for i in 0..splashes {
            let n = |k: f32| hash(i, k + 600.0 + layer as f32 * 70.0);
            let phase = (n(1.0) + background.rain * (1.5 + n(2.0))).fract();
            if phase > 0.25 {
                continue;
            }
            let p = phase / 0.25;
            let x = x0 + n(3.0) * w;
            let y = surface(x, layer);
            // two droplets jump up and out, then fall back
            let spread = (2.0 + 6.0 * p) * scale;
            let hop = (p * std::f32::consts::PI).sin() * 5.0 * scale + 2.0;
            let alpha = 0.55 * (1.0 - p) * (0.5 + 0.5 * scale).min(1.0);
            let size = if scale > 1.0 { 3.0 } else { 2.0 };
            for side in [-1.0, 1.0] {
                draw_rect(
                    canvas,
                    Rect::new((x + side * spread).round(), (y - hop).round(), size, size),
                    with_alpha(RAIN_COLOR, alpha),
                );
            }
        }
        // wind tears spray off the crests in a strong storm
        let spray = (SPRAY as f32 * smoothstep(0.2, 1.0, sea_storm) * scale / 1.5) as usize;
        for i in 0..spray {
            let n = |k: f32| hash(i, k + 800.0 + layer as f32 * 70.0);
            let phase = (n(1.0) + background.wind * (0.25 + 0.2 * n(2.0))).fract();
            let start = x0 + n(3.0) * (w + 160.0);
            let x = start - phase * (80.0 + 120.0 * n(4.0)) * (0.5 + storm) * scale;
            let y = surface(start, layer)
                - (phase * std::f32::consts::PI).sin() * (10.0 + 25.0 * n(5.0)) * storm * scale;
            let alpha = 0.45 * (1.0 - phase) * storm;
            let size = if n(6.0) < 0.3 { 3.0 } else { 2.0 } * scale.max(0.75);
            draw_rect(
                canvas,
                Rect::new(x.round(), y.round(), size.round(), size.round()),
                with_alpha(CREST_COLOR, alpha),
            );
        }
    }
    // past a full storm, sheets of rain and mist wash out the view
    if whiteout > 0.0 {
        draw_whiteout(canvas, view_rect, background, whiteout, flash);
    }
    // the nearest rain sweeps across in front of everything
    let front = SEA_LAYERS.len() - 1;
    draw_rain_layer(canvas, &rain, RAIN_LAYERS.len() - 1, &|x| {
        surface(x, front) + sea_h
    });
}

/// blowing mist and a pale veil over the whole storm
pub(super) fn draw_whiteout(
    canvas: &mut Painter,
    view_rect: &Rect,
    background: &Background,
    whiteout: f32,
    flash: f32,
) {
    let (x0, w, h) = (view_rect.x, view_rect.w, view_rect.h);
    // banks of mist racing downwind
    for i in 0..WHITEOUT_BANKS {
        let n = |k: f32| hash(i, k + 900.0);
        let x = x0
            + ((n(1.0) - background.wind * 0.03 * (0.7 + 0.6 * n(2.0))).rem_euclid(1.0) * 1.6
                - 0.3)
                * w;
        let y = view_rect.y + h * (0.1 + 0.85 * n(3.0));
        let size = w * (0.2 + 0.2 * n(4.0));
        draw_glow(
            canvas,
            Vec2::new(x, y),
            Vec2::new(size, size * 0.35),
            0.0,
            with_alpha(WHITEOUT_COLOR, 0.18 * whiteout * (0.6 + 0.4 * n(5.0))),
        );
    }
    // the veil thickens and thins with the gusts, and glows when lightning hits
    let gusting = 0.85 + 0.15 * (background.wind * 0.5).sin();
    let color = mix_color(WHITEOUT_COLOR, Color::WHITE, flash * 0.3);
    draw_rect(
        canvas,
        *view_rect,
        with_alpha(color, WHITEOUT_VEIL.0 * whiteout * gusting),
    );
    // the sea disappears behind the downpour first
    let sea_top = view_rect.y + h * (SEA_LEVEL - 0.06);
    let over_sea = (WHITEOUT_VEIL.1 - WHITEOUT_VEIL.0) * whiteout * gusting;
    draw_rect(
        canvas,
        Rect::new(x0, sea_top, w, view_rect.y + h - sea_top),
        with_alpha(color, over_sea),
    );
    // soft edge so the sea fades in rather than starting at a line
    draw_fade(
        canvas,
        Rect::new(x0, sea_top, w, -h * 0.12),
        with_alpha(color, over_sea),
    );
}

/// level up in the storm: a blinding flash, then bolts striking one after another across the sea
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_level_up_storm(
    canvas: &mut Painter,
    view_rect: &Rect,
    background: &Background,
    p: f32,
    sea_top: f32,
    flash: f32,
) {
    let (x0, w, h) = (view_rect.x, view_rect.w, view_rect.h);
    // the whole sky goes white for a moment
    draw_rect(
        canvas,
        *view_rect,
        with_alpha(Color::WHITE, LEVEL_UP_WHITE * flash),
    );
    set_blend_mode(canvas, graphics::BlendMode::ADD);
    let cloud_base = view_rect.y + h * 0.18;
    let seed = background.stars * 131;
    for b in 0..LEVEL_UP_BOLTS {
        // each bolt strikes a beat after the last, sweeping across the view
        let age = p - b as f32 * LEVEL_UP_STAGGER;
        if !(0.0..LEVEL_UP_BOLT_TIME).contains(&age) {
            continue;
        }
        let t = age / LEVEL_UP_BOLT_TIME;
        let flicker = 0.7 + 0.3 * (age * 120.0).cos();
        let bright = ((1.0 - t).powi(2) * flicker).clamp(0.0, 1.0);
        let lane = (b as f32 + 0.5) / LEVEL_UP_BOLTS as f32;
        let x = x0 + w * (0.05 + 0.9 * lane) + (hash(seed + b, 50.0) - 0.5) * w * 0.08;
        draw_fork(canvas, seed + b * 7, x, cloud_base, sea_top, w, bright);
    }
    set_blend_mode(canvas, graphics::BlendMode::ALPHA);
}

/// what every rain layer shares
pub(super) struct RainView {
    view: Rect,
    slant: f32, // px sideways per px down
    drops: usize,
    alpha: f32,
    clock: f32,
    storm: f32,
    forward: f32, // 0.0 rain spread into the distance to 1.0 crowded up front
}

/// one layer of rain, falling until it meets the surface below it
pub(super) fn draw_rain_layer(
    canvas: &mut Painter,
    rain: &RainView,
    layer: usize,
    surface: &dyn Fn(f32) -> f32,
) {
    let (calm_share, length, width, alpha, speed) = RAIN_LAYERS[layer];
    // as the storm builds, drops move up front and the far rain thins out
    let share = calm_share + (RAIN_SHARE_STORM[layer] - calm_share) * rain.forward;
    let view = rain.view;
    // drops fall along the wind: down and sideways by the slant
    let lean = rain.slant.atan();
    let fall = view.h * 1.15;
    let count = (rain.drops as f32 * share) as usize;
    for i in 0..count {
        let n = |k: f32| hash(i, k + 400.0 + layer as f32 * 50.0);
        let y = view.y - view.h * 0.1
            + (n(2.0) + rain.clock * RAIN_FALL * speed * (0.9 + 0.2 * n(1.0))).fract() * fall;
        // spread wider upwind so slanted rain still covers the view
        let x = view.x + (n(3.0) * (1.3 + rain.slant) - 0.05) * view.w - (y - view.y) * rain.slant;
        let len =
            (length * (0.8 + 0.4 * n(4.0)) * (1.0 + 0.5 * rain.storm)).min(surface(x) + 2.0 - y);
        if len <= 0.0 {
            continue;
        }
        draw_mesh(
            canvas,
            &canvas.meshes.rect,
            graphics::DrawParam::default()
                .dest(Vec2::new(x, y))
                .rotation(lean)
                .scale(Vec2::new(width, len))
                .color(with_alpha(RAIN_COLOR, (rain.alpha * alpha).min(1.0))),
        );
    }
}

/// a jagged bolt from one point to another, returns the points along it
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_bolt(
    canvas: &mut Painter,
    from: Vec2,
    to: Vec2,
    segments: usize,
    size: f32,
    jag: f32,
    seed: usize,
    color: Color,
) -> Vec<Vec2> {
    let along = to - from;
    let side = Vec2::new(-along.y, along.x).normalize_or_zero();
    let mut offset = 0.0;
    let mut path = vec![from];
    for s in 1..=segments {
        // wander off the line, pulled back so the bolt still lands on target
        offset = if s == segments {
            0.0
        } else {
            offset * 0.6 + (hash(seed * 97 + s, 25.0) - 0.5) * 2.0 * jag
        };
        let next = from + along * (s as f32 / segments as f32) + side * offset;
        draw_bolt_segment(canvas, path[s - 1], next, size, color);
        path.push(next);
    }
    path
}

/// cloud to sea bolt with a fork, a glow where it leaves the cloud and where it hits the water
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_fork(
    canvas: &mut Painter,
    seed: usize,
    x: f32,
    cloud_base: f32,
    sea_top: f32,
    w: f32,
    bright: f32,
) {
    let from = Vec2::new(x, cloud_base);
    let to = Vec2::new(x + (hash(seed, 41.0) - 0.5) * w * 0.25, sea_top);
    draw_glow(
        canvas,
        from,
        Vec2::new(w * 0.25, w * 0.12),
        0.0,
        with_alpha(FLASH_COLOR, 0.35 * bright),
    );
    let path = draw_bolt(
        canvas,
        from,
        to,
        BOLT_SEGMENTS,
        3.0,
        w * 0.035,
        seed,
        with_alpha(BOLT_COLOR, bright),
    );
    let fork_at = path[BOLT_SEGMENTS / 3 + (hash(seed, 24.0) * 3.0) as usize];
    let side = if hash(seed, 26.0) < 0.5 { -1.0 } else { 1.0 };
    let fork_to = fork_at
        + Vec2::new(
            side * w * (0.06 + 0.08 * hash(seed, 27.0)),
            (sea_top - cloud_base) * 0.3,
        );
    draw_bolt(
        canvas,
        fork_at,
        fork_to,
        6,
        2.0,
        w * 0.015,
        seed + 1,
        with_alpha(BOLT_COLOR, bright * 0.6),
    );
    // the strike lights up the water where it lands
    draw_glow(
        canvas,
        to,
        Vec2::new(w * 0.1, 22.0),
        0.0,
        with_alpha(FLASH_COLOR, 0.55 * bright),
    );
}

/// dotted line of a lightning bolt
pub(super) fn draw_bolt_segment(
    canvas: &mut Painter,
    from: Vec2,
    to: Vec2,
    size: f32,
    color: Color,
) {
    // bloom: soft glow stretched along the whole segment, a wide halo and a tight core
    let along = to - from;
    let middle = from + along / 2.0;
    let angle = along.y.atan2(along.x);
    let half = along.length() / 2.0;
    for (spread, strength) in BOLT_BLOOM {
        let width = size * spread;
        draw_glow(
            canvas,
            middle,
            Vec2::new(half + width, width),
            angle,
            with_alpha(BOLT_GLOW_COLOR, color.a * strength),
        );
    }
    let dots = (along.length() / (size * 0.75)).ceil().max(1.0) as usize;
    for d in 0..=dots {
        let at = from + (to - from) * (d as f32 / dots as f32);
        draw_rect(
            canvas,
            Rect::new(
                (at.x - size / 2.0).round(),
                (at.y - size / 2.0).round(),
                size,
                size,
            ),
            color,
        );
    }
}
