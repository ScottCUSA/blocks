use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use super::*;

pub(super) const DEBUG_ROW_HEIGHT: f32 = 26.0;
pub(super) const DEBUG_PANEL_GAP: f32 = 24.0; // below the hold and next previews
const COMPACT_ROW_HEIGHT: f32 = 18.0; // the corner fps panel
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
    pub draws: usize,
    pub scene_ms: f32,    // building the draw list
    pub encode_ms: f32,   // ggez turning it into gpu commands
    pub gpu_wait_ms: f32, // blocked on the last frame's gpu work, near 0 when the gpu keeps up
    pub fps: f32,
    pub frame_ms: f32,
}

/// animation viewer list and controls
pub struct DebugScreen<'a> {
    pub names: &'a [&'a str],
    pub selected: usize,
    pub speed: f64,
    pub paused: bool,
    pub replay: bool,
    pub level: usize,
    pub fps: u32,
    pub stats: Option<FrameStats>, // None until the first window is averaged
    pub recording: bool,
    pub stats_only: bool, // a benchmark shows just the frame cost
}

impl Layer for DebugScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let view = canvas.view;
        let top = view.preview_rect.bottom() + DEBUG_PANEL_GAP;

        // frame cost, small in the top left corner
        if let Some(stats) = self.stats {
            let mut lines = vec![
                format!("{:.0} fps  {:.2}ms", stats.fps, stats.frame_ms),
                format!("scene {:.2} enc {:.2}", stats.scene_ms, stats.encode_ms),
            ];
            if !self.stats_only {
                let rec = if self.recording { "  REC" } else { "" };
                lines.push(format!("{} draws{rec}", stats.draws));
                lines.push(format!("GPU wait {:.2}ms", stats.gpu_wait_ms));
            }
            let lines: Vec<_> = lines.iter().map(|l| (l.as_str(), LABEL_COLOR)).collect();
            let at = Rect::new(PANEL_PADDING, PANEL_PADDING, view.hold_rect.w, 0.0);
            draw_lines(
                canvas,
                at,
                &lines,
                COMPACT_ROW_HEIGHT,
                SMALL_TEXT_SIZE * 0.6,
                None,
            )?;
        }
        if self.stats_only {
            return Ok(());
        }

        // animations, right of the playfield
        let status = if self.paused {
            "Paused".to_string()
        } else {
            format!("Speed {}x", self.speed)
        };
        let replay = if self.replay {
            "Replay On"
        } else {
            "Replay Off"
        };
        let level = format!("Level {}  {} FPS", self.level, self.fps);
        let mut lines = vec![("Animations", LABEL_COLOR)];
        lines.extend(self.names.iter().map(|name| (*name, Color::WHITE)));
        lines.extend([status.as_str(), replay, &level].map(|line| (line, LABEL_COLOR)));
        let left = view.preview_rect.x;
        let width = (view.view_rect.w - left - PANEL_PADDING).max(view.preview_rect.w);
        let at = Rect::new(left, top, width, 0.0);
        let selected = Some(self.selected + 1);
        draw_lines(
            canvas,
            at,
            &lines,
            DEBUG_ROW_HEIGHT,
            SMALL_TEXT_SIZE,
            selected,
        )?;

        // controls, left of the playfield
        let right = view.hold_rect.right();
        let width = (right - PANEL_PADDING).max(view.hold_rect.w);
        let hints = DEBUG_HINTS.map(|hint| (hint, LABEL_COLOR));
        let at = Rect::new(right - width, top, width, 0.0);
        draw_lines(
            canvas,
            at,
            &hints,
            DEBUG_ROW_HEIGHT,
            SMALL_TEXT_SIZE * 0.75,
            None,
        )
    }
}

/// a panel of centered text lines, `at` gives its top left and width
fn draw_lines(
    canvas: &mut Painter,
    at: Rect,
    lines: &[(&str, Color)],
    row_height: f32,
    size: f32,
    selected: Option<usize>,
) -> GameResult {
    let height = lines.len() as f32 * row_height + 2.0 * PANEL_PADDING;
    let panel = Rect::new(at.x, at.y, at.w, height);
    draw_panel(canvas, panel);
    for (i, &(line, color)) in lines.iter().enumerate() {
        let row = menu_row_rect(panel, i, row_height);
        let color = if selected == Some(i) {
            draw_selection(canvas, row);
            ACCENT_COLOR
        } else {
            color
        };
        draw_text_centered(canvas, &ui_text(line, size), row.center().into(), color)?;
    }
    Ok(())
}
