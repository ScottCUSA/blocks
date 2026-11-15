use ggez::glam::Vec2;
use ggez::graphics::{self, Canvas, Color, DrawMode, Rect, StrokeOptions, TextFragment};
use ggez::{Context, GameResult};

use crate::menus::{self, Menu};
use crate::playfield::{self, LineClear, Playfield, SlotState};
use crate::rustomino::{Rustomino, RustominoType};
use crate::effects::Effects;
use crate::score_display::ScoreDisplay;
use crate::util;
use strum::IntoEnumIterator;

const BLOCK_SIZE: f32 = 30.;
const BLOCK_PADDING: f32 = 1.;
const STAGING_PADDING: f32 = 2.;

pub const BACKGROUND_COLOR: Color = Color::new(0.0, 0.29, 0.38, 1.0);
pub const VIEW_WIDTH: f32 = 1024.0;
pub const VIEW_HEIGHT: f32 = 768.;
const UI_FONT_SIZE: f32 = 24.0;
pub const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PLAYFIELD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const BACKGROUND_TOP_COLOR: Color = Color::new(0.02, 0.06, 0.14, 1.0);
const FRAME_COLOR: Color = Color::new(0.55, 0.8, 0.9, 0.35);
const FRAME_WIDTH: f32 = 3.0;
const GRID_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.06);
const BEVEL: f32 = 3.0; // block edge highlight and shadow width
const GHOST_FILL_ALPHA: f32 = 0.12;
const GHOST_OUTLINE_ALPHA: f32 = 0.6;
const LABEL_COLOR: Color = Color::new(0.7, 0.88, 1.0, 1.0);
const TEXT_SHADOW_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PANEL_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.7);
const HIGHLIGHT_COLOR: Color = Color::new(1.0, 0.92, 0.72, 0.15);
const ACCENT_COLOR: Color = LINE_CLEAR_COLOR;
const SLIDER_EMPTY_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.12);
const TITLE_SIZE: f32 = 96.0;
const TITLE_TINT: f32 = 0.05; // how far title letters are lightened from the piece colors
const MENU_ITEM_SIZE: f32 = 40.0;
const OPTION_LABEL_SIZE: f32 = 32.0;
const MENU_ROW_HEIGHT: f32 = 60.0;
const MENU_PANEL_WIDTH: f32 = 520.0;
const OPTIONS_PANEL_WIDTH: f32 = 600.0;
const PANEL_PADDING: f32 = 20.0;
const SMALL_TEXT_SIZE: f32 = 16.0;
const PAUSED_TITLE_SIZE: f32 = 64.0;
const PREVIEW_PADDING: f32 = 12.0;
const CONTROLS_ROW_HEIGHT: f32 = 26.0;
const SLIDER_SEGMENT: f32 = 18.0;
const SLIDER_GAP: f32 = 4.0;
const SCORE_POP_SCALE: f32 = 0.3; // how much the score grows when it increases
const SCORE_POPUP_OFFSET: f32 = 34.0; // where "+points" starts above the score
const SCORE_POPUP_RISE: f32 = 40.0; // how far "+points" floats up
const BACKDROP_PIECES: usize = 16;
const CONTROLS: [(&str, &str); 6] = [
    ("Move", "Left/Right  A/D"),
    ("Rotate", "Up/W  Ctrl/Z"),
    ("Soft drop", "Down/S"),
    ("Hard drop", "Space"),
    ("Hold", "Shift/C"),
    ("Pause", "Esc"),
];
const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);
const LINE_CLEAR_COLOR: Color = Color::new(1.0, 0.92, 0.72, 1.0); // soft gold

#[derive(Debug)]
pub struct ViewSettings {
    pub view_rect: Rect,
    pub playfield_rect: Rect,
    pub staging_rect: Rect,
    pub preview_rect: Rect,
    pub hold_rect: Rect,
    pub score_label_pos: Vec2,
    pub level_label_pos: Vec2,
    pub title_pos: Vec2,
    pub level_pos: Vec2,
    pub score_pos: Vec2,
}

impl ViewSettings {
    pub fn new(drawable_width: f32, drawable_height: f32) -> Self {
        // calculate the playfield dimensions based on block size, padding and playfield slots
        let playfield_w =
            (playfield::PLAYFIELD_SLOTS[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let playfield_h = ((playfield::PLAYFIELD_SLOTS[1] - 2) as f32
            * (BLOCK_SIZE + BLOCK_PADDING))
            + BLOCK_PADDING;

        // calculate the dimentions of the staging area
        let staging_w = playfield_w;
        let staging_h = (2. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        // calculate the dimensions of the preview area
        let preview_w = (4. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING + 2.0 * PREVIEW_PADDING;
        let preview_h = staging_h + 2.0 * PREVIEW_PADDING;
        // calculate the dimensions of the hold area
        let hold_w = preview_w;
        let hold_h = preview_h;

        // center playfield
        let playfield_x = drawable_width / 2.0 - playfield_w / 2.0;
        let playfield_y = drawable_height / 2.0 - playfield_h / 2.0 + staging_h / 2.0 + 1.0;
        // center staging area above playfield
        let staging_x = playfield_x;
        let staging_y = playfield_y - staging_h - STAGING_PADDING;
        // center preview area to the right of playfield
        let preview_x = playfield_x + playfield_w + 10.0;
        let preview_y = playfield_y;
        // center hold area to the left of playfield
        let hold_x = playfield_x - preview_w - 10.0;
        let hold_y = playfield_y;

        Self {
            view_rect: Rect::new(0., 0., drawable_width, drawable_height),
            playfield_rect: Rect::new(playfield_x, playfield_y, playfield_w, playfield_h),
            staging_rect: Rect::new(staging_x, staging_y, staging_w, staging_h),
            preview_rect: Rect::new(preview_x, preview_y, preview_w, preview_h),
            hold_rect: Rect::new(hold_x, hold_y, hold_w, hold_h),
            score_label_pos: Vec2::new(
                playfield_x + playfield_w + 30.0,
                playfield_y + playfield_h - 30.0,
            ),
            level_label_pos: Vec2::new(playfield_x - 180.0, playfield_y + playfield_h - 30.0),
            title_pos: Vec2::new(hold_x, staging_y - 4.0),
            level_pos: Vec2::new(playfield_x - 60.0, playfield_y + playfield_h - 30.0),
            score_pos: Vec2::new(
                playfield_x + playfield_w + 150.0,
                playfield_y + playfield_h - 30.0,
            ),
        }
    }
}

/// meshes reused every frame
pub struct Meshes {
    rect: graphics::Mesh,       // unit rectangle, scaled and colored per draw
    ghost: graphics::Mesh,      // unit rectangle outline
    background: graphics::Mesh, // unit rectangle with a vertical gradient
}

impl Meshes {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let unit = Rect::new(0.0, 0.0, 1.0, 1.0);
        let vertex = |x: f32, y: f32, color: Color| graphics::Vertex {
            position: [x, y],
            uv: [x, y],
            color: [color.r, color.g, color.b, color.a],
        };
        Ok(Self {
            rect: graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), unit, Color::WHITE)?,
            ghost: graphics::Mesh::new_rectangle(
                ctx,
                DrawMode::Stroke(StrokeOptions::default().with_line_width(0.08)),
                unit,
                Color::WHITE,
            )?,
            background: graphics::Mesh::from_data(
                ctx,
                graphics::MeshData {
                    vertices: &[
                        vertex(0.0, 0.0, BACKGROUND_TOP_COLOR),
                        vertex(1.0, 0.0, BACKGROUND_TOP_COLOR),
                        vertex(1.0, 1.0, BACKGROUND_COLOR),
                        vertex(0.0, 1.0, BACKGROUND_COLOR),
                    ],
                    indices: &[0, 1, 2, 0, 2, 3],
                },
            ),
        })
    }
}

pub fn draw_background(canvas: &mut Canvas, meshes: &Meshes, view_rect: &Rect) {
    canvas.draw(
        &meshes.background,
        graphics::DrawParam::default().dest_rect(*view_rect),
    );
}

fn draw_rect(canvas: &mut Canvas, meshes: &Meshes, rect: Rect, color: Color) {
    canvas.draw(
        &meshes.rect,
        graphics::DrawParam::default().dest_rect(rect).color(color),
    );
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

fn mix_color(from: Color, to: Color, amount: f32) -> Color {
    Color::new(
        from.r + (to.r - from.r) * amount,
        from.g + (to.g - from.g) * amount,
        from.b + (to.b - from.b) * amount,
        from.a + (to.a - from.a) * amount,
    )
}

fn shade(color: Color, amount: f32) -> Color {
    Color::new(
        color.r * amount,
        color.g * amount,
        color.b * amount,
        color.a,
    )
}

fn tint(color: Color, amount: f32) -> Color {
    Color::new(
        color.r + (1.0 - color.r) * amount,
        color.g + (1.0 - color.g) * amount,
        color.b + (1.0 - color.b) * amount,
        color.a,
    )
}

/// bevelled block with a light top left edge, dark bottom right edge and gloss
fn draw_block(canvas: &mut Canvas, meshes: &Meshes, rect: Rect, color: Color) {
    draw_rect(canvas, meshes, rect, shade(color, 0.55));
    let highlight = Rect::new(rect.x, rect.y, rect.w - BEVEL, rect.h - BEVEL);
    draw_rect(canvas, meshes, highlight, tint(color, 0.45));
    let face = Rect::new(
        rect.x + BEVEL,
        rect.y + BEVEL,
        rect.w - 2.0 * BEVEL,
        rect.h - 2.0 * BEVEL,
    );
    draw_rect(canvas, meshes, face, color);
    let gloss = Rect::new(face.x, face.y, face.w, face.h * 0.45);
    draw_rect(
        canvas,
        meshes,
        gloss,
        with_alpha(Color::WHITE, 0.12 * color.a),
    );
}

fn block_color(rtype: RustominoType, game_over: bool) -> Color {
    if game_over {
        util::rgb_to_grayscale(rtype.color())
    } else {
        rtype.color()
    }
}

/// fade out in a wipe from where the rustomino landed to the edges
fn line_clear_alpha(line_clear: &LineClear, x: usize) -> f32 {
    let start = line_clear.wipe_start(x);
    (1.0 - (line_clear.progress() - start) / playfield::LINE_CLEAR_FADE).clamp(0.0, 1.0) as f32
}

fn frame_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x - FRAME_WIDTH,
        rect.y - FRAME_WIDTH,
        rect.w + 2.0 * FRAME_WIDTH,
        rect.h + 2.0 * FRAME_WIDTH,
    )
}

fn draw_playing_backgound(canvas: &mut Canvas, meshes: &Meshes, view_settings: &ViewSettings) {
    // frames around the staging area and playfield, and the side panels
    let staging = view_settings.staging_rect;
    let playfield = view_settings.playfield_rect;
    let play_area = Rect::new(
        staging.x,
        staging.y,
        staging.w,
        playfield.y + playfield.h - staging.y,
    );
    for rect in [
        play_area,
        view_settings.preview_rect,
        view_settings.hold_rect,
    ] {
        draw_rect(canvas, meshes, frame_rect(rect), FRAME_COLOR);
    }

    let backgrounds = [
        (view_settings.staging_rect, STAGING_BACKGROUND_COLOR),
        (view_settings.playfield_rect, PLAYFIELD_BACKGROUND_COLOR),
        (view_settings.preview_rect, PREVIEW_BACKGROUND_COLOR),
        (view_settings.hold_rect, HOLD_BACKGROUND_COLOR),
    ];
    for (rect, color) in backgrounds {
        draw_rect(canvas, meshes, rect, color);
    }

    // grid lines in the gaps between playfield cells
    let cell = BLOCK_SIZE + BLOCK_PADDING;
    for col in 1..playfield::PLAYFIELD_SLOTS[0] {
        let x = playfield.x + col as f32 * cell;
        let line = Rect::new(x, playfield.y, BLOCK_PADDING, playfield.h);
        draw_rect(canvas, meshes, line, GRID_COLOR);
    }
    for row in 1..playfield::PLAYFIELD_SIZE[1] {
        let y = playfield.y + playfield.h - 2.0 * BLOCK_PADDING - row as f32 * cell;
        let line = Rect::new(playfield.x, y, playfield.w, BLOCK_PADDING);
        draw_rect(canvas, meshes, line, GRID_COLOR);
    }
}

fn draw_playfield(
    canvas: &mut Canvas,
    meshes: &Meshes,
    playfield: &Playfield,
    staging_rect: &Rect,
    playfield_rect: &Rect,
    game_over: bool,
) {
    // draw the locked blocks
    for (y, slots_x) in playfield.slots.iter().enumerate() {
        let clearing = playfield
            .line_clear
            .as_ref()
            .filter(|line_clear| line_clear.lines.contains(&y));
        for (x, slot) in slots_x.iter().enumerate() {
            if let SlotState::Locked(rtype) = slot {
                let mut color = block_color(*rtype, game_over);
                if let Some(line_clear) = clearing {
                    let flash = (1.0 - line_clear.progress() as f32 * 4.0).max(0.0);
                    color = mix_color(LINE_CLEAR_COLOR, Color::WHITE, flash);
                    color.a *= line_clear_alpha(line_clear, x);
                }
                let rect = playfield_block_rect([x as i32, y as i32], staging_rect, playfield_rect);
                draw_block(canvas, meshes, rect, color);
            }
        }
    }

    // draw the active rustomino over the locked blocks
    if let Some(active) = &playfield.active_rustomino {
        for block in active.playfield_slots() {
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            draw_block(canvas, meshes, rect, block_color(active.rtype, game_over));
        }
    }

    if let Some(ghost) = &playfield.ghost_rustomino {
        let color = block_color(ghost.rtype, game_over);
        for block in ghost.playfield_slots() {
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            draw_rect(canvas, meshes, rect, with_alpha(color, GHOST_FILL_ALPHA));
            canvas.draw(
                &meshes.ghost,
                graphics::DrawParam::default()
                    .dest_rect(rect)
                    .color(with_alpha(color, GHOST_OUTLINE_ALPHA)),
            );
        }
    }
}

fn draw_preview(
    canvas: &mut Canvas,
    meshes: &Meshes,
    rustomino: &Option<Rustomino>,
    preview_rect: &Rect,
    game_over: bool,
) {
    if let Some(rustomino) = rustomino {
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
        let left = preview_rect.x + (preview_rect.w - width) / 2.0;
        let top = preview_rect.y + (preview_rect.h - height) / 2.0;
        for block in rustomino.blocks {
            let rect = Rect::new(
                left + (block.x - min.x) as f32 * cell,
                top + (max.y - block.y) as f32 * cell,
                BLOCK_SIZE,
                BLOCK_SIZE,
            );
            draw_block(
                canvas,
                meshes,
                rect,
                block_color(rustomino.rtype, game_over),
            );
        }
    }
}

pub fn draw_playing(
    canvas: &mut Canvas,
    meshes: &Meshes,
    playfield: &Playfield,
    next_rustomino: &Option<Rustomino>,
    hold_rustomino: &Option<Rustomino>,
    view_settings: &ViewSettings,
    game_over: bool,
) {
    draw_playing_backgound(canvas, meshes, view_settings);
    draw_playfield(
        canvas,
        meshes,
        playfield,
        &view_settings.staging_rect,
        &view_settings.playfield_rect,
        game_over,
    );
    draw_preview(
        canvas,
        meshes,
        hold_rustomino,
        &view_settings.hold_rect,
        game_over,
    );
    draw_preview(
        canvas,
        meshes,
        next_rustomino,
        &view_settings.preview_rect,
        game_over,
    );
}

fn ui_text(text: &str, size: f32) -> graphics::Text {
    let mut text = graphics::Text::new(text);
    text.set_font("04b30")
        .set_scale(graphics::PxScale::from(size));
    text
}

fn draw_shadowed(canvas: &mut Canvas, text: &graphics::Text, pos: Vec2, color: Color) {
    let param = graphics::DrawParam::default();
    // the shadow fades with the text
    let shadow = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * color.a);
    canvas.draw(text, param.dest(pos + Vec2::new(2.0, 2.0)).color(shadow));
    canvas.draw(text, param.dest(pos).color(color));
}

/// text in the ui font with a drop shadow
fn draw_text(canvas: &mut Canvas, text: &str, pos: Vec2, size: f32, color: Color) {
    draw_shadowed(canvas, &ui_text(text, size), pos, color);
}

/// text positioned by its center
fn draw_text_centered(
    ctx: &Context,
    canvas: &mut Canvas,
    text: &graphics::Text,
    center: Vec2,
    color: Color,
) -> GameResult {
    let size = text.measure(ctx)?;
    draw_shadowed(
        canvas,
        text,
        center - Vec2::new(size.x / 2.0, size.y / 2.0),
        color,
    );
    Ok(())
}

pub fn draw_playing_text(
    _ctx: &mut Context,
    canvas: &mut Canvas,
    level: usize,
    score: &ScoreDisplay,
    view_settings: &ViewSettings,
) -> GameResult {
    let label_size = UI_FONT_SIZE * 0.75;
    let label_offset = Vec2::new(0.0, -(label_size + FRAME_WIDTH + 8.0));
    let hold = view_settings.hold_rect;
    let preview = view_settings.preview_rect;

    draw_piece_colored(canvas, "Blocks!", view_settings.title_pos, UI_FONT_SIZE);
    draw_text(
        canvas,
        "Hold",
        Vec2::new(hold.x, hold.y) + label_offset,
        label_size,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        "Next",
        Vec2::new(preview.x, preview.y) + label_offset,
        label_size,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        "Level:",
        view_settings.level_label_pos,
        UI_FONT_SIZE,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        "Score:",
        view_settings.score_label_pos,
        UI_FONT_SIZE,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        &level.to_string(),
        view_settings.level_pos,
        UI_FONT_SIZE,
        Color::WHITE,
    );
    draw_score(canvas, score, view_settings.score_pos);

    Ok(())
}

/// score that counts up and pops when it increases, with rising "+points"
fn draw_score(canvas: &mut Canvas, score: &ScoreDisplay, pos: Vec2) {
    let pop = score.pop();
    let size = UI_FONT_SIZE * (1.0 + SCORE_POP_SCALE * pop);
    let grow = Vec2::new(0.0, (size - UI_FONT_SIZE) / 2.0);
    let color = mix_color(Color::WHITE, ACCENT_COLOR, pop);
    draw_text(canvas, &score.shown().to_string(), pos - grow, size, color);

    for popup in &score.popups {
        let t = popup.progress();
        let rise = Vec2::new(0.0, SCORE_POPUP_OFFSET + SCORE_POPUP_RISE * t);
        let color = with_alpha(ACCENT_COLOR, 1.0 - t * t);
        let text = format!("+{}", popup.points);
        draw_text(canvas, &text, pos - rise, UI_FONT_SIZE * 0.8, color);
    }
}

/// translucent pieces drifting down behind the menus
pub fn draw_backdrop(ctx: &Context, canvas: &mut Canvas, meshes: &Meshes, view_rect: &Rect) {
    let time = ctx.time.time_since_start().as_secs_f32();
    let types: Vec<RustominoType> = RustominoType::iter().collect();
    for i in 0..BACKDROP_PIECES {
        // repeatable pseudo random values per piece
        let random = |n: f32| {
            ((i as f32 * 12.9898 + n * 78.233).sin() * 43758.547)
                .fract()
                .abs()
        };
        let size = 16.0 + random(1.0) * 14.0;
        let speed = 20.0 + random(2.0) * 40.0;
        let span = view_rect.h + size * 8.0;
        let x = random(3.0) * (view_rect.w - size * 4.0);
        let y = (random(4.0) * span + time * speed) % span - size * 4.0;
        let rtype = types[i % types.len()];
        let color = with_alpha(rtype.color(), 0.12 + random(5.0) * 0.1);
        for block in Rustomino::new(rtype).blocks {
            let rect = Rect::new(
                x + block.x as f32 * size,
                y - block.y as f32 * size,
                size - 2.0,
                size - 2.0,
            );
            draw_rect(canvas, meshes, rect, color);
        }
    }
}

pub fn draw_overlay(canvas: &mut Canvas, meshes: &Meshes, view_rect: &Rect) {
    draw_rect(canvas, meshes, *view_rect, PAUSED_OVERLAY_COLOR);
}

/// large wobbling title, optionally with each letter in a piece color
/// text with each letter colored like a piece, and a matching shadow text
/// per letter colors override the draw color, so the shadow is its own text
fn piece_colored_text(text: &str, size: f32) -> (graphics::Text, graphics::Text) {
    let colors: Vec<Color> = RustominoType::iter().map(|t| t.color()).collect();
    let letters = |color_at: &dyn Fn(usize) -> Color| {
        let mut letters = graphics::Text::default();
        for (i, c) in text.chars().enumerate() {
            letters.add(TextFragment::new(c).color(color_at(i)));
        }
        letters
            .set_font("04b30")
            .set_scale(graphics::PxScale::from(size));
        letters
    };
    (
        letters(&|i| tint(colors[i % colors.len()], TITLE_TINT)),
        letters(&|_| TEXT_SHADOW_COLOR),
    )
}

fn draw_piece_colored(canvas: &mut Canvas, text: &str, pos: Vec2, size: f32) {
    let (text, shadow) = piece_colored_text(text, size);
    let param = graphics::DrawParam::default();
    canvas.draw(
        &shadow,
        param.dest(pos + Vec2::splat((size / 16.0).max(2.0))),
    );
    canvas.draw(&text, param.dest(pos));
}

/// large wobbling piece colored title
fn draw_title(
    ctx: &Context,
    canvas: &mut Canvas,
    title: &str,
    center_y: f32,
    size: f32,
    view_rect: &Rect,
) -> GameResult {
    let wobble = util::slow_wobble(ctx.time.time_since_start().as_secs_f32());
    let center = Vec2::new(view_rect.w / 2.0, center_y + wobble * size / 12.0);
    let dims = piece_colored_text(title, size).0.measure(ctx)?;
    let pos = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
    draw_piece_colored(canvas, title, pos, size);
    Ok(())
}

fn draw_hint(ctx: &Context, canvas: &mut Canvas, hint: &str, view_rect: &Rect) -> GameResult {
    let center = Vec2::new(view_rect.w / 2.0, view_rect.h - 32.0);
    draw_text_centered(
        ctx,
        canvas,
        &ui_text(hint, SMALL_TEXT_SIZE),
        center,
        LABEL_COLOR,
    )
}

fn draw_panel(canvas: &mut Canvas, meshes: &Meshes, rect: Rect) {
    draw_rect(canvas, meshes, frame_rect(rect), FRAME_COLOR);
    draw_rect(canvas, meshes, rect, PANEL_COLOR);
}

fn menu_panel_rect(view_rect: &Rect, rows: usize, top: f32, width: f32) -> Rect {
    let height = rows as f32 * MENU_ROW_HEIGHT + 2.0 * PANEL_PADDING;
    Rect::new(view_rect.w / 2.0 - width / 2.0, top, width, height)
}

fn menu_row_rect(panel: Rect, row: usize) -> Rect {
    Rect::new(
        panel.x + PANEL_PADDING,
        panel.y + PANEL_PADDING + row as f32 * MENU_ROW_HEIGHT,
        panel.w - 2.0 * PANEL_PADDING,
        MENU_ROW_HEIGHT,
    )
}

/// highlight bar with a bobbing gold block at each end
fn draw_selection(ctx: &Context, canvas: &mut Canvas, meshes: &Meshes, row: Rect) {
    draw_rect(canvas, meshes, row, HIGHLIGHT_COLOR);
    let bob = util::fast_wobble(ctx.time.time_since_start().as_secs_f32()) * 3.0;
    let size = row.h * 0.3;
    let y = row.y + row.h / 2.0 - size / 2.0;
    for x in [row.x + 12.0 + bob, row.x + row.w - 12.0 - size - bob] {
        draw_block(canvas, meshes, Rect::new(x, y, size, size), ACCENT_COLOR);
    }
}

fn draw_menu_list<T: Menu>(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &T,
    panel: Rect,
) -> GameResult {
    for (i, item) in menu.items().iter().enumerate() {
        let row = menu_row_rect(panel, i);
        let selected = menu.selected() == i;
        if selected {
            draw_selection(ctx, canvas, meshes, row);
        }
        let mut text = item.clone();
        text.set_font("04b30")
            .set_scale(graphics::PxScale::from(MENU_ITEM_SIZE));
        let color = if selected { ACCENT_COLOR } else { Color::WHITE };
        draw_text_centered(ctx, canvas, &text, row.center().into(), color)?;
    }
    Ok(())
}

pub fn draw_menu(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &menus::MenuList,
    view_settings: &ViewSettings,
) -> GameResult {
    let view_rect = &view_settings.view_rect;
    draw_backdrop(ctx, canvas, meshes, view_rect);
    draw_title(
        ctx,
        canvas,
        "Blocks!",
        view_rect.h * 0.16,
        TITLE_SIZE,
        view_rect,
    )?;
    let panel = menu_panel_rect(
        view_rect,
        menu.items().len(),
        view_rect.h * 0.42,
        MENU_PANEL_WIDTH,
    );
    draw_panel(canvas, meshes, panel);
    draw_menu_list(ctx, canvas, meshes, menu, panel)?;
    draw_hint(ctx, canvas, "Up/Down: select   Enter: choose", view_rect)
}

pub fn draw_paused(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &menus::MenuList,
    view_settings: &ViewSettings,
) -> GameResult {
    let view_rect = &view_settings.view_rect;
    draw_overlay(canvas, meshes, view_rect);
    let panel = menu_panel_rect(
        view_rect,
        menu.items().len(),
        view_rect.h * 0.36,
        MENU_PANEL_WIDTH,
    );
    let title_y = panel.y - PAUSED_TITLE_SIZE * 0.75;
    draw_title(ctx, canvas, "Paused", title_y, PAUSED_TITLE_SIZE, view_rect)?;
    draw_panel(canvas, meshes, panel);
    draw_menu_list(ctx, canvas, meshes, menu, panel)?;
    draw_hint(
        ctx,
        canvas,
        "Up/Down: select   Enter: choose   Esc: resume",
        view_rect,
    )
}

/// row of blocks filled up to the level, right aligned to right_x
fn draw_slider(
    canvas: &mut Canvas,
    meshes: &Meshes,
    right_x: f32,
    center_y: f32,
    level: u8,
    max: u8,
) {
    let width = max as f32 * (SLIDER_SEGMENT + SLIDER_GAP) - SLIDER_GAP;
    for i in 0..max {
        let rect = Rect::new(
            right_x - width + i as f32 * (SLIDER_SEGMENT + SLIDER_GAP),
            center_y - SLIDER_SEGMENT / 2.0,
            SLIDER_SEGMENT,
            SLIDER_SEGMENT,
        );
        if i < level {
            draw_block(canvas, meshes, rect, ACCENT_COLOR);
        } else {
            draw_rect(canvas, meshes, rect, SLIDER_EMPTY_COLOR);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_options(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &menus::MenuList,
    theme: &str,
    music_level: u8,
    sound_level: u8,
    max_level: u8,
    view_settings: &ViewSettings,
) -> GameResult {
    let view_rect = &view_settings.view_rect;
    draw_title(
        ctx,
        canvas,
        "Options",
        view_rect.h * 0.16,
        TITLE_SIZE,
        view_rect,
    )?;

    // theme, volume sliders and back
    let panel = menu_panel_rect(
        view_rect,
        menu.items().len(),
        view_rect.h * 0.235,
        OPTIONS_PANEL_WIDTH,
    );
    let slider_width = max_level as f32 * (SLIDER_SEGMENT + SLIDER_GAP) - SLIDER_GAP;
    draw_panel(canvas, meshes, panel);
    for (i, item) in menu.items().iter().enumerate() {
        let row = menu_row_rect(panel, i);
        let selected = menu.selected() == i;
        if selected {
            draw_selection(ctx, canvas, meshes, row);
        }
        let color = if selected { ACCENT_COLOR } else { Color::WHITE };
        let mut text = item.clone();
        text.set_font("04b30");
        if i == menus::OPTION_BACK {
            text.set_scale(graphics::PxScale::from(MENU_ITEM_SIZE));
            draw_text_centered(ctx, canvas, &text, row.center().into(), color)?;
            continue;
        }

        // label on the left, setting on the right
        text.set_scale(graphics::PxScale::from(OPTION_LABEL_SIZE));
        let size = text.measure(ctx)?;
        let center_y = row.y + row.h / 2.0;
        draw_shadowed(
            canvas,
            &text,
            Vec2::new(row.x + 48.0, center_y - size.y / 2.0),
            color,
        );
        let right_x = row.x + row.w - 48.0;
        match i {
            menus::OPTION_THEME => {
                let center_x = right_x - slider_width / 2.0;
                draw_text_centered(
                    ctx,
                    canvas,
                    &ui_text(theme, OPTION_LABEL_SIZE * 0.75),
                    Vec2::new(center_x, center_y),
                    color,
                )?;
                // arrow blocks either side of the theme name
                let arrow = SLIDER_SEGMENT * 0.6;
                let arrow_color = if selected {
                    ACCENT_COLOR
                } else {
                    SLIDER_EMPTY_COLOR
                };
                for x in [right_x - slider_width, right_x - arrow] {
                    let rect = Rect::new(x, center_y - arrow / 2.0, arrow, arrow);
                    draw_block(canvas, meshes, rect, arrow_color);
                }
            }
            menus::OPTION_MUSIC => {
                draw_slider(canvas, meshes, right_x, center_y, music_level, max_level)
            }
            _ => draw_slider(canvas, meshes, right_x, center_y, sound_level, max_level),
        }
    }

    // controls reference
    let header = PANEL_PADDING + CONTROLS_ROW_HEIGHT * 1.2;
    let controls = Rect::new(
        panel.x,
        panel.y + panel.h + 24.0,
        panel.w,
        header + CONTROLS.len() as f32 * CONTROLS_ROW_HEIGHT + PANEL_PADDING,
    );
    draw_panel(canvas, meshes, controls);
    let left = controls.x + PANEL_PADDING + 28.0;
    let right = controls.x + controls.w - PANEL_PADDING - 28.0;
    draw_text(
        canvas,
        "Controls",
        Vec2::new(left, controls.y + PANEL_PADDING),
        SMALL_TEXT_SIZE * 1.25,
        LABEL_COLOR,
    );
    for (i, (action, keys)) in CONTROLS.iter().enumerate() {
        let y = controls.y + header + i as f32 * CONTROLS_ROW_HEIGHT;
        draw_text(
            canvas,
            action,
            Vec2::new(left, y),
            SMALL_TEXT_SIZE,
            LABEL_COLOR,
        );
        let keys = ui_text(keys, SMALL_TEXT_SIZE);
        let width = keys.measure(ctx)?.x;
        draw_shadowed(canvas, &keys, Vec2::new(right - width, y), Color::WHITE);
    }

    draw_hint(
        ctx,
        canvas,
        "Up/Down: select   Left/Right: adjust   Esc: back",
        view_rect,
    )
}

pub fn draw_gameover(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_rect: &Rect,
) -> GameResult {
    draw_overlay(canvas, meshes, view_rect);
    let panel = Rect::new(
        view_rect.w / 2.0 - MENU_PANEL_WIDTH / 2.0,
        view_rect.h / 2.0 - MENU_ROW_HEIGHT,
        MENU_PANEL_WIDTH,
        2.0 * MENU_ROW_HEIGHT,
    );
    draw_panel(canvas, meshes, panel);
    let wobble = util::slow_wobble(ctx.time.time_since_start().as_secs_f32());
    let center = Vec2::new(view_rect.w / 2.0, view_rect.h / 2.0 - wobble * 6.0);
    draw_text_centered(
        ctx,
        canvas,
        &ui_text("Game Over!", 56.0),
        center,
        Color::WHITE,
    )?;
    draw_hint(ctx, canvas, "Press any key", view_rect)
}

/// particles and playfield flash
pub fn draw_effects(
    canvas: &mut Canvas,
    meshes: &Meshes,
    effects: &Effects,
    view_settings: &ViewSettings,
) {
    let playfield = view_settings.playfield_rect;
    let flash = effects.flash();
    if flash > 0.0 {
        draw_rect(canvas, meshes, playfield, with_alpha(Color::WHITE, flash));
    }
    let cell = BLOCK_SIZE + BLOCK_PADDING;
    let left = view_settings.staging_rect.x + 1.0;
    let bottom = playfield.y + playfield.h - 1.0;
    for particle in effects.particles.iter().filter(|p| p.visible()) {
        let fade = particle.fade();
        let size = BLOCK_SIZE * particle.size * (0.4 + 0.6 * fade);
        let center = Vec2::new(left + particle.pos.x * cell, bottom - particle.pos.y * cell);
        let rect = Rect::new(center.x - size / 2.0, center.y - size / 2.0, size, size);
        draw_rect(canvas, meshes, rect, with_alpha(particle.color, fade));
    }
}

fn playfield_block_rect(block: [i32; 2], staging_rect: &Rect, playfield_rect: &Rect) -> Rect {
    // block[x,y] absolute units
    let x = staging_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of playfield_rect
    let y = playfield_rect.y + playfield_rect.h
        - ((block[1] + 1) as f32 * (BLOCK_SIZE + BLOCK_PADDING))
        - 1.0;

    Rect::new(x, y, BLOCK_SIZE, BLOCK_SIZE)
}
