use crate::gfx::{Color, Rect};
use crate::GameResult;

use crate::draw::*;

pub(super) const DEBUG_ROW_HEIGHT: f32 = 26.0;
pub(super) const DEBUG_PANEL_GAP: f32 = 24.0; // below the hold and next previews
const COMPACT_ROW_HEIGHT: f32 = 18.0; // the benchmark's fps panel
pub(super) const DEBUG_HINTS: [&str; 13] = [
    "Up/Down: pick",
    "Enter: restart",
    "R: replay on/off",
    "Left/Right: speed",
    "-/+: level",
    "Shift -/+: level x10",
    "P: pause",
    "B/S: styles",
    "H: hide panels",
    "F/Shift F: fps",
    "C: record frames",
    "T: benchmark",
    "Esc: back",
];

/// averaged cost of drawing a frame
#[derive(Debug, Clone, Copy)]
pub struct FrameStats {
    pub draws: usize, // vertex runs
    pub vertices: usize,
    pub scene_ms: f32, // building the draw list
    pub fps: f32,      // bevy diagnostics, smoothed
    pub frame_ms: f32,
}

/// animation viewer list and controls
/// animation viewer controls
pub struct DebugScreen<'a> {
    pub names: &'a [&'a str],
    pub selected: usize,
    pub speed: f64,
    pub paused: bool,
    pub replay: bool,
    pub level: usize,
    pub fps: Option<u32>,          // cap, None unlocked
    pub stats: Option<FrameStats>, // None until the first window is averaged
    pub recording: bool,
    pub stats_only: bool, // a benchmark shows just the frame cost
}

impl Layer for DebugScreen<'_> {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let Self {
            names,
            selected,
            speed,
            paused,
            replay,
            level,
            fps,
            stats,
            recording,
            stats_only,
        } = *self;
        let time = frame.time;
        let meshes = frame.meshes;
        let view_settings = frame.view;
        let style = frame.block;
        let view = &view_settings.view_rect;
        let top = view_settings.preview_rect.bottom() + DEBUG_PANEL_GAP;
        // controls, left of the playfield
        let right = view_settings.hold_rect.right();
        let width = (right - PANEL_PADDING).max(view_settings.hold_rect.w);
        let controls = Rect::new(
            right - width,
            top,
            width,
            DEBUG_HINTS.len() as f32 * DEBUG_ROW_HEIGHT + 2.0 * PANEL_PADDING,
        );
        let Some(stats) = stats else {
            return Ok(());
        };
        if stats_only {
            // small, in the top left corner on every screen
            let hold = view_settings.hold_rect;
            let lines = [
                format!("{:.0} fps  {:.2}ms", stats.fps, stats.frame_ms),
                format!("scene {:.2}ms", stats.scene_ms),
            ];
            let at = Rect::new(PANEL_PADDING, PANEL_PADDING, hold.w, 0.0);
            return draw_lines(canvas, meshes, at, &lines, COMPACT_ROW_HEIGHT, 0.6);
        }
        let lines = [
            if recording {
                format!("{} draws  REC", stats.draws)
            } else {
                format!("{} draws", stats.draws)
            },
            format!("{} vertices", stats.vertices),
            format!("{:.0} fps  {:.2}ms", stats.fps, stats.frame_ms),
            format!("scene {:.2}ms", stats.scene_ms),
        ];
        let at = Rect::new(
            controls.x,
            controls.bottom() + DEBUG_PANEL_GAP,
            controls.w,
            0.0,
        );
        draw_lines(canvas, meshes, at, &lines, DEBUG_ROW_HEIGHT, 0.75)?;

        // animations, right of the playfield
        let left = view_settings.preview_rect.x;
        let width = (view.w - left - PANEL_PADDING).max(view_settings.preview_rect.w);
        let rows = names.len() + 4;
        let panel = Rect::new(
            left,
            top,
            width,
            rows as f32 * DEBUG_ROW_HEIGHT + 2.0 * PANEL_PADDING,
        );
        draw_panel(canvas, meshes, panel);
        let title = ui_text("Animations", SMALL_TEXT_SIZE);
        let title_row = menu_row_rect(panel, 0, DEBUG_ROW_HEIGHT);
        draw_text_centered(canvas, &title, title_row.center(), LABEL_COLOR)?;
        for (i, name) in names.iter().enumerate() {
            let row = menu_row_rect(panel, i + 1, DEBUG_ROW_HEIGHT);
            let chosen = i == selected;
            if chosen {
                draw_selection(time, canvas, meshes, row, style);
            }
            let color = if chosen { ACCENT_COLOR } else { Color::WHITE };
            let text = ui_text(name, SMALL_TEXT_SIZE);
            draw_text_centered(canvas, &text, row.center(), color)?;
        }
        let status = if paused {
            "Paused".to_string()
        } else {
            format!("Speed {speed}x")
        };
        let replay = if replay { "Replay On" } else { "Replay Off" };
        let level = match fps {
            Some(fps) => format!("Level {level}  {fps} FPS"),
            None => format!("Level {level}  Unlocked"),
        };
        for (i, line) in [status.as_str(), replay, level.as_str()]
            .into_iter()
            .enumerate()
        {
            let row = menu_row_rect(panel, rows - 3 + i, DEBUG_ROW_HEIGHT);
            let text = ui_text(line, SMALL_TEXT_SIZE);
            draw_text_centered(canvas, &text, row.center(), LABEL_COLOR)?;
        }

        let panel = controls;
        draw_panel(canvas, meshes, panel);
        for (i, hint) in DEBUG_HINTS.iter().enumerate() {
            let row = menu_row_rect(panel, i, DEBUG_ROW_HEIGHT);
            let text = ui_text(hint, SMALL_TEXT_SIZE * 0.75);
            draw_text_centered(canvas, &text, row.center(), LABEL_COLOR)?;
        }

        Ok(())
    }
}

/// a panel of text lines, `at` gives its top left and width
fn draw_lines(
    canvas: &mut Painter,
    meshes: &Meshes,
    at: Rect,
    lines: &[String],
    row_height: f32,
    text_scale: f32,
) -> GameResult {
    let panel = Rect::new(
        at.x,
        at.y,
        at.w,
        lines.len() as f32 * row_height + 2.0 * PANEL_PADDING,
    );
    draw_panel(canvas, meshes, panel);
    for (i, line) in lines.iter().enumerate() {
        let row = menu_row_rect(panel, i, row_height);
        let text = ui_text(line, SMALL_TEXT_SIZE * text_scale);
        draw_text_centered(canvas, &text, row.center(), LABEL_COLOR)?;
    }
    Ok(())
}
