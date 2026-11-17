//! the bevy app: window, camera, frame pacing, screenshots and quitting

use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot as Capture, ScreenshotCaptured};
use bevy::render::RenderPlugin;
use bevy::window::{
    Monitor, PresentMode, PrimaryWindow, WindowCloseRequested, WindowResizeConstraints,
    WindowResolution,
};
use bevy::winit::{UpdateMode, WinitSettings, WINIT_WINDOWS};

use crate::audio::AudioPlugin;
use crate::draw::{SpritesPlugin, ViewSettings, VIEW_HEIGHT, VIEW_WIDTH};
use crate::effects::EffectsPlugin;
use crate::game::bench::{Bench, StartBench};
use crate::game::debug::DebugView;
use crate::game::Viewer;
use crate::game::{self, Focused, GamePlugin, NoSave, Prefs, RequestQuit, Screenshot};
use crate::render::{PaintCamera, PaintPlugin};
use crate::scores::ScoreBoard;
use crate::settings::{Settings, WindowState};
use crate::Resources;

const MIN_WINDOW_SCALE: f32 = 0.5; // smallest window, in logical pixels, relative to the view
const SCREENSHOT_FRAME: u32 = 3; // frames drawn before a capture, so the atlas is uploaded
const SCREENSHOT_TRIES: u32 = 100; // blank captures while pipelines compile, before giving up
                                   // frames run after saving: exiting while pipelines still compile in the background can crash
const SCREENSHOT_EXIT_FRAMES: u32 = 30;

/// where a screenshot run saves, and the image it renders into
#[derive(Resource)]
struct ScreenshotTarget {
    image: Handle<Image>,
    path: std::path::PathBuf,
    frames: u32,
    tries: u32,
    exit_in: Option<u32>, // frames left after saving
}

pub fn run(
    resources: Resources,
    screenshot: Option<Screenshot>,
    bench: Option<StartBench>,
    settings: Settings,
) {
    let font = resources
        .load("04b30.ttf")
        .expect("could not load the font");
    crate::gfx::set_font(font).expect("could not read the font");
    let capture_path = screenshot.as_ref().map(|s| s.path.clone());
    let window = primary_window(capture_path.is_some(), &settings.window);
    let view = if capture_path.is_some() {
        (VIEW_WIDTH, VIEW_HEIGHT)
    } else {
        // replaced by the real size on the first frame
        (window.width(), window.height())
    };
    let scores = if screenshot.is_none() {
        ScoreBoard::load()
    } else {
        ScoreBoard::default()
    };

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                // saved before closing, in quit
                close_when_requested: false,
                ..default()
            })
            // a screenshot's first frames are complete, not missing what's still compiling
            .set(RenderPlugin {
                synchronous_pipeline_compilation: capture_path.is_some(),
                ..default()
            }),
    )
    .insert_resource(resources)
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::Continuous,
    })
    .add_plugins((
        AudioPlugin {
            silent: screenshot.is_some(),
        },
        PaintPlugin,
        SpritesPlugin,
        EffectsPlugin,
        GamePlugin {
            screenshot: screenshot.as_ref().map(|s| s.scene),
            settings,
            scores,
            view,
        },
    ))
    .add_systems(Startup, (capture_setup, setup_camera).chain())
    .add_systems(First, pace)
    .add_systems(PreUpdate, resize)
    .add_systems(Update, show_window)
    .add_systems(Last, (capture, quit));
    if let Some(bench) = bench {
        app.insert_resource(bench);
    }
    if let Some(path) = capture_path {
        app.insert_resource(PendingCapture(path));
    }
    app.run();
}

/// resizable window, sized in logical pixels so high dpi screens get a larger window
/// starts hidden, show_window places and shows it
fn primary_window(screenshot: bool, saved: &WindowState) -> Window {
    let (min_width, min_height) = (
        VIEW_WIDTH * MIN_WINDOW_SCALE,
        VIEW_HEIGHT * MIN_WINDOW_SCALE,
    );
    let resolution = if screenshot {
        // screenshots stay at a fixed pixel size
        WindowResolution::new(VIEW_WIDTH as u32, VIEW_HEIGHT as u32).with_scale_factor_override(1.0)
    } else {
        WindowResolution::new(
            saved.width.max(min_width) as u32,
            saved.height.max(min_height) as u32,
        )
    };
    Window {
        title: "Blocks".into(),
        resolution,
        resize_constraints: WindowResizeConstraints {
            min_width,
            min_height,
            ..default()
        },
        resizable: true,
        present_mode: PresentMode::AutoVsync,
        visible: false,
        ..default()
    }
}

#[derive(Resource)]
struct PendingCapture(std::path::PathBuf);

/// screenshots render into an image instead of the hidden window
fn capture_setup(
    mut commands: Commands,
    pending: Option<Res<PendingCapture>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(pending) = pending else {
        return;
    };
    let image = Image::new_target_texture(
        VIEW_WIDTH as u32,
        VIEW_HEIGHT as u32,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    commands.insert_resource(ScreenshotTarget {
        image: images.add(image),
        path: pending.0.clone(),
        frames: 0,
        tries: 0,
        exit_in: None,
    });
}

fn setup_camera(mut commands: Commands, target: Option<Res<ScreenshotTarget>>) {
    let mut camera = commands.spawn((Camera2d, Msaa::Off, PaintCamera));
    if let Some(target) = target {
        camera.insert(RenderTarget::Image(ImageRenderTarget::from(
            target.image.clone(),
        )));
    }
}

const SPIN: Duration = Duration::from_micros(250); // before a frame is due

/// cap the frame rate at the tick rate, 15fps in the background
/// the viewer sets its own cap and vsync, a benchmark runs uncapped without vsync
fn pace(
    debug: Res<DebugView>,
    viewer: Res<State<Viewer>>,
    focused: Res<Focused>,
    bench: Option<Res<Bench>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut next: Local<Option<Instant>>,
    mut fixed: ResMut<Time<Fixed>>,
) {
    let viewing = *viewer.get() == Viewer::On;
    let unlocked = bench.is_some() || (viewing && debug.fps_cap().is_none());
    let vsync = bench.is_none() && (!viewing || debug.vsync);
    let present = if vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    if let Ok(mut window) = windows.single_mut() {
        if window.present_mode != present {
            window.present_mode = present;
        }
    }
    // unfocused wins over unlocked, but not over a benchmark
    let frame = if !focused.0 && bench.is_none() {
        game::UNFOCUSED_FRAME
    } else if unlocked {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(1.0 / debug.tick_rate() as f64)
    };
    // against a running deadline, so a late wake is made up next frame
    let now = Instant::now();
    if frame.is_zero() {
        *next = None;
    } else {
        // over a frame behind, start again from now
        let due = next.filter(|due| *due + frame > now).unwrap_or(now);
        if let Some(wait) = due.checked_duration_since(now) {
            // sleep wakes late, spin the rest
            if let Some(sleep) = wait.checked_sub(SPIN) {
                std::thread::sleep(sleep);
            }
            while Instant::now() < due {
                std::hint::spin_loop();
            }
        }
        *next = Some(due + frame);
    }
    // the viewer changes the tick rate
    let step = Duration::from_secs_f64(1.0 / debug.tick_rate() as f64);
    if fixed.timestep() != step {
        fixed.set_timestep(step);
    }
}

/// lay the view out again when the drawable size changes
fn resize(
    windows: Query<&Window, With<PrimaryWindow>>,
    no_save: Option<Res<NoSave>>,
    mut view: ResMut<ViewSettings>,
    mut size: Local<UVec2>,
) {
    // screenshots keep their fixed size
    if no_save.is_some() {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let physical = window.physical_size();
    if physical != *size && physical.x > 0 && physical.y > 0 {
        *size = physical;
        *view = ViewSettings::new(physical.x as f32, physical.y as f32);
        crate::draw::set_pixel_scale(view.scale);
    }
}

/// restore the saved position if it is on a monitor, then show the window
fn show_window(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    settings: Res<Settings>,
    no_save: Option<Res<NoSave>>,
    mut shown: Local<bool>,
) {
    if *shown || no_save.is_some() {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    *shown = true;
    let saved = &settings.window;
    if let (Some(x), Some(y)) = (saved.x, saved.y) {
        let on_screen = monitors.iter().any(|m| {
            let (pos, w, h) = (m.physical_position, m.physical_width, m.physical_height);
            (pos.x..pos.x + w as i32).contains(&x) && (pos.y..pos.y + h as i32).contains(&y)
        });
        if on_screen {
            window.position = WindowPosition::At(IVec2::new(x, y));
        }
    }
    if saved.maximized {
        window.set_maximized(true);
    }
    window.visible = true;
}

/// save the screenshot once a few frames are drawn
fn capture(
    mut commands: Commands,
    target: Option<ResMut<ScreenshotTarget>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(mut target) = target else {
        return;
    };
    if let Some(frames) = &mut target.exit_in {
        *frames = frames.saturating_sub(1);
        if *frames == 0 {
            exit.write(AppExit::Success);
        }
        return;
    }
    target.frames += 1;
    if target.frames != SCREENSHOT_FRAME {
        return;
    }
    let path = target.path.clone();
    commands
        .spawn(Capture::image(target.image.clone()))
        .observe(
            move |captured: On<ScreenshotCaptured>, mut target: ResMut<ScreenshotTarget>| {
                // pipelines compile in the background, the first frames can be one flat color
                let blank = captured.image.data.as_ref().is_none_or(|data| {
                    let mut pixels = data.as_chunks::<4>().0.iter();
                    let first = pixels.next();
                    pixels.all(|pixel| Some(pixel) == first)
                });
                target.tries += 1;
                if blank && target.tries < SCREENSHOT_TRIES {
                    target.frames = 0;
                    return;
                }
                let saved = save_capture(&captured.image, &path);
                match saved {
                    Ok(()) => println!("saved screenshot to {}", path.display()),
                    Err(e) => log::error!("could not save the screenshot: {e}"),
                }
                target.exit_in = Some(SCREENSHOT_EXIT_FRAMES);
            },
        );
}

/// on a close request or the quit menu item, save and exit
fn quit(
    mut close: MessageReader<WindowCloseRequested>,
    mut requests: MessageReader<RequestQuit>,
    mut exit: MessageWriter<AppExit>,
    mut prefs: Prefs,
    debug: Res<DebugView>,
    no_save: Option<Res<NoSave>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    _main_thread: NonSendMarker,
) {
    let requested = close.read().count() + requests.read().count() > 0;
    if !requested {
        return;
    }
    if no_save.is_none() {
        if let Ok((entity, window)) = windows.single() {
            let (maximized, position) = WINIT_WINDOWS.with_borrow(|winit| {
                winit
                    .get_window(entity)
                    .map(|w| (w.is_maximized(), w.outer_position().ok()))
                    .unwrap_or((false, None))
            });
            // the size and position are kept while maximized
            let saved = &mut prefs.settings.window;
            saved.maximized = maximized;
            if !maximized {
                saved.width = window.width();
                saved.height = window.height();
                // not available on wayland
                if let Some(position) = position {
                    (saved.x, saved.y) = (Some(position.x), Some(position.y));
                }
            }
        }
        // not the viewer's styles
        if let Some((style, background)) = debug.saved_style() {
            *prefs.style = style;
            prefs.background.style = background;
        }
        prefs.save();
    }
    exit.write(AppExit::Success);
}

fn save_capture(image: &Image, path: &std::path::Path) -> crate::GameResult {
    let mut pixels = image
        .data
        .clone()
        .ok_or_else(|| crate::GameError("the screenshot has no pixels".into()))?;
    match image.texture_descriptor.format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {}
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => {
            for pixel in pixels.as_chunks_mut::<4>().0 {
                pixel.swap(0, 2);
            }
        }
        format => {
            return Err(crate::GameError(format!(
                "cannot save a screenshot in the {format:?} format"
            )))
        }
    }
    game::save_png(&pixels, image.width(), image.height(), path)
}
