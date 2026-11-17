use crate::gfx::{BlendMode, Depth, DrawList, DrawParam, Mesh, Rect, Text};

/// the frame's draw list, drawn into by the layers
pub struct Painter<'a> {
    list: &'a mut DrawList,
}

impl<'a> Painter<'a> {
    pub fn new(list: &'a mut DrawList) -> Self {
        Painter { list }
    }

    /// depth band for the draws after this
    pub fn set_layer(&mut self, layer: Depth) {
        self.list.set_layer(layer);
    }

    pub(super) fn screen_coordinates(&self) -> Option<Rect> {
        Some(self.list.screen_coordinates())
    }
}

/// draw a mesh
pub(super) fn draw_mesh(painter: &mut Painter, mesh: &Mesh, param: impl Into<DrawParam>) {
    painter.list.draw_mesh(mesh, param.into());
}

/// draw text
pub(super) fn draw(painter: &mut Painter, text: &Text, param: impl Into<DrawParam>) {
    painter.list.draw_text(text, param.into());
}

/// blend mode for the draws after this
pub(super) fn set_blend_mode(painter: &mut Painter, mode: BlendMode) {
    painter.list.set_blend_mode(mode);
}

/// screen coordinates for the draws after this
pub(super) fn set_screen_coordinates(painter: &mut Painter, rect: Rect) {
    painter.list.set_screen_coordinates(rect);
}

/// depth band for the draws after this
pub(super) fn set_layer(painter: &mut Painter, layer: Depth) {
    painter.list.set_layer(layer);
}
