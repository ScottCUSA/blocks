//! immediate mode 2d drawing: draws are tessellated into vertex runs each frame,
//! the renderer hands the runs to bevy

use std::sync::OnceLock;

use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont};
pub use bevy::math::{IVec2, Vec2};

/// srgb color, converted to linear for the gpu
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn linear(self) -> [f32; 4] {
        let channel = |c: f32| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        [channel(self.r), channel(self.g), channel(self.b), self.a]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn point(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// depth band of a draw: under or over the board's sprites
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Depth {
    #[default]
    Back,
    Front,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Alpha,
    Add,
}

/// placement of a draw: scaled, rotated, then moved to dest
#[derive(Debug, Clone, Copy)]
pub struct DrawParam {
    pub dest: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub color: Color,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            dest: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
            color: Color::WHITE,
        }
    }
}

impl DrawParam {
    pub fn dest(mut self, dest: impl Into<Vec2>) -> Self {
        self.dest = dest.into();
        self
    }

    pub fn scale(mut self, scale: impl Into<Vec2>) -> Self {
        self.scale = scale.into();
        self
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// unit shapes fill the rect
    pub fn dest_rect(self, rect: Rect) -> Self {
        self.dest(rect.point()).scale(Vec2::new(rect.w, rect.h))
    }

    fn apply(&self, p: Vec2) -> Vec2 {
        let p = p * self.scale;
        let (sin, cos) = self.rotation.sin_cos();
        self.dest + Vec2::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Color,
}

/// triangles in local coordinates
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn rect(rect: Rect, color: Color) -> Self {
        let corners = [
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.right(), rect.y),
            Vec2::new(rect.right(), rect.bottom()),
            Vec2::new(rect.x, rect.bottom()),
        ];
        Mesh {
            vertices: corners.map(|position| Vertex { position, color }).to_vec(),
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    pub fn rounded_rect(rect: Rect, radius: f32, color: Color) -> Self {
        let outline = rounded_outline(rect, radius);
        let mut vertices = vec![Vertex {
            position: rect.center(),
            color,
        }];
        vertices.extend(outline.iter().map(|&position| Vertex { position, color }));
        let n = outline.len() as u32;
        let indices = (0..n).flat_map(|k| [0, 1 + k, 1 + (k + 1) % n]).collect();
        Mesh { vertices, indices }
    }
}

/// points around a rounded rect, clockwise from the top left corner
fn rounded_outline(rect: Rect, radius: f32) -> Vec<Vec2> {
    const CORNER_SEGMENTS: usize = 8;
    let r = radius.min(rect.w / 2.0).min(rect.h / 2.0).max(0.0);
    let corners = [
        (Vec2::new(rect.x + r, rect.y + r), std::f32::consts::PI),
        (
            Vec2::new(rect.right() - r, rect.y + r),
            1.5 * std::f32::consts::PI,
        ),
        (Vec2::new(rect.right() - r, rect.bottom() - r), 0.0),
        (
            Vec2::new(rect.x + r, rect.bottom() - r),
            0.5 * std::f32::consts::PI,
        ),
    ];
    let mut points = Vec::with_capacity(4 * (CORNER_SEGMENTS + 1));
    for (center, start) in corners {
        for s in 0..=CORNER_SEGMENTS {
            let a = start + s as f32 / CORNER_SEGMENTS as f32 * std::f32::consts::FRAC_PI_2;
            points.push(center + Vec2::new(a.cos(), a.sin()) * r);
        }
    }
    points
}

// FONT

static FONT: OnceLock<FontArc> = OnceLock::new();

/// the ui font, set once at startup
pub fn set_font(data: Vec<u8>) -> Result<(), String> {
    let font = FontArc::try_from_vec(data).map_err(|e| e.to_string())?;
    FONT.set(font).map_err(|_| "font already set".to_string())
}

fn font() -> &'static FontArc {
    FONT.get().expect("font not loaded")
}

#[derive(Debug, Clone)]
pub struct TextFragment {
    pub text: String,
    pub color: Option<Color>, // overrides the draw color
}

impl TextFragment {
    pub fn new(text: impl Into<TextFragment>) -> Self {
        text.into()
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl From<&str> for TextFragment {
    fn from(text: &str) -> Self {
        TextFragment {
            text: text.to_string(),
            color: None,
        }
    }
}

impl From<char> for TextFragment {
    fn from(c: char) -> Self {
        TextFragment {
            text: c.to_string(),
            color: None,
        }
    }
}

/// a line of text in the ui font, sized in physical pixels
#[derive(Debug, Clone)]
pub struct Text {
    fragments: Vec<TextFragment>,
    scale: f32,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            fragments: Vec::new(),
            scale: 16.0,
        }
    }
}

struct PlacedGlyph {
    id: GlyphId,
    x: f32,
    color: Option<Color>,
}

impl Text {
    pub fn new(text: impl Into<TextFragment>) -> Self {
        let mut t = Text::default();
        t.add(text);
        t
    }

    pub fn add(&mut self, fragment: impl Into<TextFragment>) -> &mut Self {
        self.fragments.push(fragment.into());
        self
    }

    pub fn set_scale(&mut self, scale: f32) -> &mut Self {
        self.scale = scale;
        self
    }

    /// glyphs along the line, x from the start
    fn layout(&self) -> (Vec<PlacedGlyph>, f32) {
        let font = font().as_scaled(PxScale::from(self.scale));
        let mut glyphs = Vec::new();
        let mut x = 0.0;
        let mut last = None;
        for fragment in &self.fragments {
            for c in fragment.text.chars() {
                let id = font.glyph_id(c);
                if let Some(last) = last {
                    x += font.kern(last, id);
                }
                glyphs.push(PlacedGlyph {
                    id,
                    x,
                    color: fragment.color,
                });
                x += font.h_advance(id);
                last = Some(id);
            }
        }
        (glyphs, x)
    }

    /// width and line height in physical pixels
    pub fn measure(&self) -> Vec2 {
        let height = font().as_scaled(PxScale::from(self.scale)).height();
        Vec2::new(self.layout().1, height)
    }
}

// GLYPH ATLAS

const ATLAS_SIZE: u32 = 2048;
const WHITE_SIZE: u32 = 4; // solid block at the origin, sampled by shapes
const GLYPH_PADDING: u32 = 1;

#[derive(Clone, Copy)]
struct AtlasGlyph {
    offset: Vec2, // from the pen position at the baseline
    size: Vec2,
    uv_min: Vec2,
    uv_max: Vec2,
}

struct Full;

/// rasterized glyphs packed in shelves, white with coverage in alpha
pub struct Atlas {
    pub pixels: Vec<u8>, // rgba
    pub dirty: bool,
    glyphs: std::collections::HashMap<(GlyphId, u32), Option<AtlasGlyph>>,
    cursor: (u32, u32),
    shelf: u32, // height of the current shelf
}

impl Default for Atlas {
    fn default() -> Self {
        let mut atlas = Atlas {
            pixels: Vec::new(),
            dirty: true,
            glyphs: Default::default(),
            cursor: (0, 0),
            shelf: 0,
        };
        atlas.clear();
        atlas
    }
}

impl Atlas {
    pub const SIZE: u32 = ATLAS_SIZE;

    fn clear(&mut self) {
        self.pixels = vec![0; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize];
        for y in 0..WHITE_SIZE {
            for x in 0..WHITE_SIZE {
                self.pixels[((y * ATLAS_SIZE + x) * 4) as usize..][..4].fill(255);
            }
        }
        self.glyphs.clear();
        self.cursor = (WHITE_SIZE + GLYPH_PADDING, 0);
        self.shelf = WHITE_SIZE;
        self.dirty = true;
    }

    fn white_uv() -> Vec2 {
        Vec2::splat(WHITE_SIZE as f32 / 2.0 / ATLAS_SIZE as f32)
    }

    fn glyph(&mut self, id: GlyphId, scale: f32) -> Option<AtlasGlyph> {
        let key = (id, scale.to_bits());
        if let Some(glyph) = self.glyphs.get(&key) {
            return *glyph;
        }
        let glyph = self.rasterize(id, scale).unwrap_or_else(|Full| {
            // start over, glyphs already placed this frame show garbage for a frame
            log::warn!("glyph atlas full, clearing");
            self.clear();
            self.rasterize(id, scale).unwrap_or(None)
        });
        self.glyphs.insert(key, glyph);
        glyph
    }

    /// None for glyphs with nothing to draw, like spaces
    fn rasterize(&mut self, id: GlyphId, scale: f32) -> Result<Option<AtlasGlyph>, Full> {
        let Some(outlined) = font().outline_glyph(id.with_scale(PxScale::from(scale))) else {
            return Ok(None);
        };
        let bounds = outlined.px_bounds();
        let (w, h) = (bounds.width() as u32, bounds.height() as u32);
        if w == 0 || h == 0 {
            return Ok(None);
        }
        if self.cursor.0 + w + GLYPH_PADDING > ATLAS_SIZE {
            self.cursor = (0, self.cursor.1 + self.shelf + GLYPH_PADDING);
            self.shelf = 0;
        }
        if self.cursor.1 + h > ATLAS_SIZE {
            return Err(Full);
        }
        let (x0, y0) = self.cursor;
        outlined.draw(|x, y, coverage| {
            let i = (((y0 + y) * ATLAS_SIZE + x0 + x) * 4) as usize;
            self.pixels[i..i + 3].fill(255);
            self.pixels[i + 3] = (coverage.clamp(0.0, 1.0) * 255.0).round() as u8;
        });
        self.cursor.0 += w + GLYPH_PADDING;
        self.shelf = self.shelf.max(h);
        self.dirty = true;
        let size = ATLAS_SIZE as f32;
        Ok(Some(AtlasGlyph {
            offset: Vec2::new(bounds.min.x, bounds.min.y),
            size: Vec2::new(w as f32, h as f32),
            uv_min: Vec2::new(x0 as f32, y0 as f32) / size,
            uv_max: Vec2::new((x0 + w) as f32, (y0 + h) as f32) / size,
        }))
    }
}

// DRAW LIST

/// gpu ready vertex, physical pixels from the top left
#[derive(Debug, Clone, Copy)]
pub struct GpuVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4], // linear
}

/// consecutive draws with one blend mode
#[derive(Default)]
pub struct Run {
    pub blend: Option<BlendMode>,
    pub layer: Depth,
    pub vertices: Vec<GpuVertex>,
    pub indices: Vec<u32>,
}

/// a frame's draws, the runs and the atlas they sample, kept across frames
pub struct DrawList {
    pub runs: Vec<Run>,
    pub used: usize, // runs drawn this frame
    pub atlas: Atlas,
    pub clear: Color,
    pub size: Vec2, // target in physical pixels
    screen: Rect,
    blend: BlendMode,
    layer: Depth,
}

impl Default for DrawList {
    fn default() -> Self {
        DrawList {
            runs: Vec::new(),
            used: 0,
            atlas: Atlas::default(),
            clear: Color::BLACK,
            size: Vec2::ONE,
            screen: Rect::new(0.0, 0.0, 1.0, 1.0),
            blend: BlendMode::Alpha,
            layer: Depth::Back,
        }
    }
}

impl DrawList {
    /// start a frame for a target of the given physical size
    pub fn begin(&mut self, size: Vec2, clear: Color) {
        for run in &mut self.runs[..self.used] {
            run.vertices.clear();
            run.indices.clear();
            run.blend = None;
        }
        self.used = 0;
        self.size = size;
        self.clear = clear;
        self.screen = Rect::new(0.0, 0.0, size.x, size.y);
        self.blend = BlendMode::Alpha;
        self.layer = Depth::Back;
    }

    pub fn screen_coordinates(&self) -> Rect {
        self.screen
    }

    /// the view rect mapped onto the whole target, for the draws after this
    pub fn set_screen_coordinates(&mut self, rect: Rect) {
        self.screen = rect;
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend = mode;
    }

    /// depth band for the draws after this
    pub fn set_layer(&mut self, layer: Depth) {
        self.layer = layer;
    }

    /// screen coordinates to physical pixels
    fn to_target(&self, p: Vec2) -> Vec2 {
        (p - self.screen.point()) * self.target_scale()
    }

    fn target_scale(&self) -> Vec2 {
        self.size / Vec2::new(self.screen.w, self.screen.h)
    }

    /// the run taking the next draw
    fn run(&mut self) -> &mut Run {
        let (blend, layer) = (self.blend, self.layer);
        let current = self.used.checked_sub(1).map(|i| &self.runs[i]);
        if current.is_none_or(|run| run.blend != Some(blend) || run.layer != layer) {
            if self.used == self.runs.len() {
                self.runs.push(Run::default());
            }
            self.used += 1;
        }
        let run = &mut self.runs[self.used - 1];
        run.blend = Some(blend);
        run.layer = layer;
        run
    }

    pub fn draw_mesh(&mut self, mesh: &Mesh, param: DrawParam) {
        let tint = param.color.linear();
        let uv = Atlas::white_uv().to_array();
        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(|v| {
                let c = v.color.linear();
                GpuVertex {
                    position: self.to_target(param.apply(v.position)).to_array(),
                    uv,
                    color: [
                        c[0] * tint[0],
                        c[1] * tint[1],
                        c[2] * tint[2],
                        c[3] * tint[3],
                    ],
                }
            })
            .collect();
        let run = self.run();
        let base = run.vertices.len() as u32;
        run.vertices.extend(vertices);
        run.indices.extend(mesh.indices.iter().map(|i| base + i));
    }

    /// text from its top left, at its own pixel size times the param scale
    pub fn draw_text(&mut self, text: &Text, param: DrawParam) {
        let scale = param.scale * self.target_scale();
        // rasterize at the size it lands on screen, so it stays crisp
        let px = (text.scale * scale.y).max(1.0);
        let stretch = Vec2::new(scale.x / scale.y, 1.0);
        let (glyphs, _) = text.layout();
        let ascent = font().as_scaled(PxScale::from(px)).ascent();
        let origin = self.to_target(param.dest).round();
        let mut quads = Vec::with_capacity(glyphs.len());
        for placed in glyphs {
            let Some(glyph) = self.atlas.glyph(placed.id, px) else {
                continue;
            };
            let pen = Vec2::new(placed.x * px / text.scale, ascent);
            let min = origin + ((pen + glyph.offset) * stretch).round();
            let max = min + glyph.size * stretch;
            let color = placed.color.unwrap_or(param.color).linear();
            quads.push((min, max, glyph, color));
        }
        let run = self.run();
        for (min, max, glyph, color) in quads {
            let base = run.vertices.len() as u32;
            let corners = [
                (
                    Vec2::new(min.x, min.y),
                    Vec2::new(glyph.uv_min.x, glyph.uv_min.y),
                ),
                (
                    Vec2::new(max.x, min.y),
                    Vec2::new(glyph.uv_max.x, glyph.uv_min.y),
                ),
                (
                    Vec2::new(max.x, max.y),
                    Vec2::new(glyph.uv_max.x, glyph.uv_max.y),
                ),
                (
                    Vec2::new(min.x, max.y),
                    Vec2::new(glyph.uv_min.x, glyph.uv_max.y),
                ),
            ];
            run.vertices.extend(corners.map(|(position, uv)| GpuVertex {
                position: position.to_array(),
                uv: uv.to_array(),
                color,
            }));
            run.indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    /// vertex count drawn this frame
    pub fn vertex_count(&self) -> usize {
        self.runs[..self.used]
            .iter()
            .map(|r| r.vertices.len())
            .sum()
    }
}
