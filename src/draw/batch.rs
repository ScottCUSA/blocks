use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use ggez::graphics::{BlendMode, Canvas, DrawParam, Drawable, InstanceArray, Mesh, Rect};
use ggez::Context;

use super::{Frame, Meshes, ViewSettings};
use crate::style::BlockStyle;

// ggez canvases don't batch, every canvas draw is its own gpu draw
// so runs of draws of the same mesh are queued here and sent as one instanced draw
// anything else drawn, or a canvas state change, sends the queue first to keep the order

// gpu draws since the last take
static DRAWS: AtomicUsize = AtomicUsize::new(0);

const MIN_POOL: usize = 16; // instance arrays to start a frame with

/// draws counted since the last call
pub fn take_draw_count() -> usize {
    DRAWS.swap(0, Ordering::Relaxed)
}

fn count(draws: usize) {
    DRAWS.fetch_add(draws, Ordering::Relaxed);
}

/// draw queue and instance array pool, kept across frames
#[derive(Default)]
pub struct Batch {
    mesh: Option<(*const Mesh, Mesh)>, // the queued mesh, compared by address
    params: Vec<DrawParam>,
    // one array per batch in a frame: ggez uploads an array when it's drawn,
    // but the gpu reads it when the frame is submitted
    pool: Vec<InstanceArray>,
    used: usize,  // arrays drawn this frame
    short: usize, // batches drawn one by one for want of an array, the pool grows next frame
}

impl Batch {
    /// start a frame on the canvas, sizing the pool for the last one
    pub fn begin<'a>(&'a mut self, canvas: &'a mut Canvas, frame: Frame<'a>) -> Painter<'a> {
        self.mesh = None;
        self.params.clear();
        let wanted = (self.used + self.short).max(MIN_POOL);
        while self.pool.len() < wanted {
            self.pool.push(InstanceArray::new(frame.ctx, None));
        }
        self.used = 0;
        self.short = 0;
        let Frame {
            ctx,
            meshes,
            view,
            block,
        } = frame;
        Painter {
            canvas,
            batch: self,
            ctx,
            meshes,
            view,
            block,
        }
    }

    fn flush(&mut self, canvas: &mut Canvas) {
        let Some((_, mesh)) = self.mesh.take() else {
            return;
        };
        if self.params.len() > 1 {
            if let Some(instances) = self.pool.get_mut(self.used) {
                instances.set(self.params.drain(..));
                canvas.draw_instanced_mesh(mesh, instances, DrawParam::default());
                self.used += 1;
                count(1);
                return;
            }
            self.short += 1;
        }
        count(self.params.len());
        for param in self.params.drain(..) {
            canvas.draw(&mesh, param);
        }
    }
}

/// a canvas drawn through the batch for a frame
pub struct Painter<'a> {
    canvas: &'a mut Canvas,
    batch: &'a mut Batch,
    pub(super) ctx: &'a Context,
    pub(super) meshes: &'a Meshes,
    pub(super) view: &'a ViewSettings,
    pub(super) block: BlockStyle, // style for every block drawn
}

impl Painter<'_> {
    /// send what's queued, before the canvas is finished
    pub fn end(self) {
        self.batch.flush(self.canvas);
    }

    /// seconds since the game started
    pub(super) fn time(&self) -> f32 {
        self.ctx.time.time_since_start().as_secs_f32()
    }

    pub(super) fn screen_coordinates(&self) -> Option<Rect> {
        self.canvas.screen_coordinates()
    }

    fn flush(&mut self) {
        self.batch.flush(self.canvas);
    }
}

/// draw a mesh, batched with the draws of the same mesh around it
pub(super) fn draw_mesh(painter: &mut Painter, mesh: &Mesh, param: impl Into<DrawParam>) {
    let batch = &mut *painter.batch;
    if batch
        .mesh
        .as_ref()
        .is_some_and(|(queued, _)| !ptr::eq(*queued, mesh))
    {
        batch.flush(painter.canvas);
    }
    if batch.mesh.is_none() {
        batch.mesh = Some((ptr::from_ref(mesh), mesh.clone()));
    }
    batch.params.push(param.into());
}

/// draw anything else, after what's queued
pub(super) fn draw(painter: &mut Painter, drawable: &impl Drawable, param: impl Into<DrawParam>) {
    painter.flush();
    count(1);
    painter.canvas.draw(drawable, param);
}

/// blend mode for the draws after this
pub(super) fn set_blend_mode(painter: &mut Painter, mode: BlendMode) {
    painter.flush();
    painter.canvas.set_blend_mode(mode);
}

/// screen coordinates for the draws after this
pub(super) fn set_screen_coordinates(painter: &mut Painter, rect: Rect) {
    painter.flush();
    painter.canvas.set_screen_coordinates(rect);
}
