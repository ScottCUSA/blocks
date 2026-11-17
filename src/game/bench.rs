//! benchmark: the viewer's scenes in turn, uncapped and without vsync, every frame timed

use std::time::Duration;

use bevy::app::AppExit;
use bevy::prelude::*;

use super::debug::{self, Animation, DebugView, Recording, RunInfo};
use super::Viewer;
use crate::background::{Background, BackgroundStyle};
use crate::draw::ViewSettings;
use crate::style::{BlockStyle, Style};

pub const DEFAULT_HOLD: f32 = 0.0; // least seconds per case, past one whole play
const WARMUP: Duration = Duration::from_secs(1); // unmeasured, before the first case
const BUDGET_MS: f64 = 1000.0 / 120.0;
const FALL_TIME: Duration = Duration::from_secs(3); // of a slow fall, not the whole drop

#[derive(Clone, Copy)]
struct Case {
    animation: Animation,
    background: BackgroundStyle,
    block: BlockStyle,
    level: usize,
}

const TOP_LEVEL: usize = 20;
const RAMP_STEP: usize = 4; // levels per level up, ramping to the top
const ANIMATIONS: [Animation; 6] = [
    Animation::Gravity,
    Animation::Single,
    Animation::Double,
    Animation::Triple,
    Animation::Quad,
    Animation::LevelUp,
];

/// per background: the animations at level 1, level ups to the top, the animations again
fn cases() -> Vec<Case> {
    let block = BlockStyle::ALL[0];
    let mut cases = Vec::new();
    for background in BackgroundStyle::ALL {
        let mut case = |animation, level| {
            cases.push(Case {
                animation,
                background,
                block,
                level,
            })
        };
        // the background alone has nothing to show when it's off
        let alone = background != BackgroundStyle::Off;
        for animation in ANIMATIONS {
            case(animation, 1);
        }
        if alone {
            case(Animation::Background, 1);
        }
        for level in (1 + RAMP_STEP..TOP_LEVEL).step_by(RAMP_STEP) {
            case(Animation::LevelUp, level);
        }
        for animation in ANIMATIONS {
            case(animation, TOP_LEVEL);
        }
        if alone {
            case(Animation::Background, TOP_LEVEL);
        }
        case(Animation::GameOver, TOP_LEVEL);
    }
    // the other block styles, over the heaviest background
    for block in &BlockStyle::ALL[1..] {
        cases.push(Case {
            animation: Animation::Quad,
            background: BackgroundStyle::Storm,
            block: *block,
            level: TOP_LEVEL,
        });
    }
    cases
}

impl Case {
    fn name(&self) -> String {
        format!(
            "{} / {} / {} / L{}",
            self.animation.name(),
            self.background.name(),
            self.block.name(),
            self.level
        )
    }
}

/// start a benchmark, from the command line or the viewer
#[derive(Resource, Clone)]
pub struct StartBench {
    pub hold: f32,
    pub filter: Option<String>, // only cases whose names contain it, any case
    pub exit: bool,             // quit when done
}

/// a benchmark in progress
#[derive(Resource)]
pub struct Bench {
    cases: Vec<Case>,
    case: usize,
    elapsed: Duration,
    plays: usize, // the viewer's count when the case started
    warming: bool,
    hold: Duration,
    exit: bool,
    starts: Vec<usize>, // first recorded frame of each case
    saved: Option<(usize, bool, bool, usize, usize)>, // viewer settings, restored after
    viewer_seen: bool,
}

pub struct BenchPlugin;

impl Plugin for BenchPlugin {
    fn build(&self, app: &mut App) {
        // after the frame, so resets don't land between queued commands
        app.add_systems(
            Last,
            (
                start.run_if(resource_exists::<StartBench>),
                step.run_if(resource_exists::<Bench>),
            )
                .chain(),
        );
    }
}

fn start(world: &mut World) {
    let Some(request) = world.remove_resource::<StartBench>() else {
        return;
    };
    if *world.resource::<State<Viewer>>().get() == Viewer::Off {
        debug::open_viewer(world);
    }
    let view = world.resource::<ViewSettings>();
    let run = RunInfo {
        drawable: (view.view_rect.w * view.scale, view.view_rect.h * view.scale),
        scale: view.scale,
        fps_cap: 0,
        kind: "bench",
    };
    let mut debug = world.resource_mut::<DebugView>();
    if let Some(recording) = debug.recording.take() {
        debug::stop_recording(recording);
    }
    debug.recording = Some(Recording::start(run));
    let saved = (
        debug.selected,
        debug.hidden,
        debug.replay,
        debug.level,
        debug.speed_index(),
    );
    debug.hidden = false;
    debug.stats_only = true;
    // the benchmark starts each play
    debug.replay = false;
    debug.paused = false;
    debug.set_speed_index(0);
    let filter = request.filter.as_deref().map(str::to_lowercase);
    let cases: Vec<Case> = cases()
        .into_iter()
        .filter(|case| {
            filter
                .as_deref()
                .is_none_or(|f| case.name().to_lowercase().contains(f))
        })
        .collect();
    if cases.is_empty() {
        println!("benchmark: no cases match");
        if request.exit {
            world.write_message(AppExit::Success);
        }
        return;
    }
    println!(
        "benchmark: {} cases, one whole play each, Esc to stop",
        cases.len()
    );
    world.insert_resource(Bench {
        cases,
        case: 0,
        elapsed: Duration::ZERO,
        plays: 0,
        warming: true,
        hold: Duration::from_secs_f32(request.hold),
        exit: request.exit,
        starts: Vec::new(),
        saved: Some(saved),
        viewer_seen: false,
    });
    begin_case(world);
}

fn begin_case(world: &mut World) {
    let bench = world.resource::<Bench>();
    let case = bench.cases[bench.case];
    let warming = bench.warming;
    let frames = world
        .resource::<DebugView>()
        .recording
        .as_ref()
        .map_or(0, Recording::len);
    world.resource_mut::<Style>().block = case.block;
    // a new background starts calm, not easing down from the last level
    let mut background = world.resource_mut::<Background>();
    if background.style != case.background {
        *background = Background::new();
        background.style = case.background;
    }
    let mut debug = world.resource_mut::<DebugView>();
    debug.selected = Animation::ALL
        .iter()
        .position(|a| *a == case.animation)
        .unwrap_or(0);
    debug.level = case.level;
    debug::start_animation(world);
    let plays = world.resource::<DebugView>().plays;
    let mut bench = world.resource_mut::<Bench>();
    bench.elapsed = Duration::ZERO;
    bench.plays = plays;
    if !warming {
        bench.starts.push(frames);
    }
}

fn step(world: &mut World) {
    let viewer = *world.resource::<State<Viewer>>().get();
    let delta = world.resource::<Time<Real>>().delta();
    let plays = world.resource::<DebugView>().plays;
    let mut bench = world.resource_mut::<Bench>();
    // Esc closes the viewer, which ends the run
    if viewer == Viewer::On {
        bench.viewer_seen = true;
    } else if bench.viewer_seen {
        finish(world, false);
        return;
    }
    bench.elapsed += delta;
    if bench.warming {
        if bench.elapsed >= WARMUP {
            bench.warming = false;
            begin_case(world);
        }
        return;
    }
    // falls are cut short, the rest play through at least once
    let cut = bench.cases[bench.case].animation == Animation::Gravity && bench.elapsed >= FALL_TIME;
    if plays == bench.plays && !cut {
        return;
    }
    // played through, again until the hold is up
    if !cut && bench.elapsed < bench.hold {
        bench.plays = plays;
        debug::start_animation(world);
        return;
    }
    bench.case += 1;
    if bench.case < bench.cases.len() {
        begin_case(world);
    } else {
        finish(world, true);
    }
}

fn finish(world: &mut World, complete: bool) {
    let Some(mut bench) = world.remove_resource::<Bench>() else {
        return;
    };
    let recording = world.resource_mut::<DebugView>().recording.take();
    if let Some((selected, hidden, replay, level, speed)) = bench.saved.take() {
        let mut debug = world.resource_mut::<DebugView>();
        debug.selected = selected;
        debug.hidden = hidden;
        debug.stats_only = false;
        debug.replay = replay;
        debug.level = level;
        debug.set_speed_index(speed);
    }
    if let Some(recording) = recording {
        summarize(&bench, &recording, complete);
        debug::stop_recording(recording);
    }
    if bench.exit {
        world.write_message(AppExit::Success);
    } else if world.resource::<State<Viewer>>().get() == &Viewer::On {
        debug::start_animation(world);
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

/// a table per case on stdout
fn summarize(bench: &Bench, recording: &Recording, complete: bool) {
    let frames = recording.frames();
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    println!(
        "\n{:<34} {:>6} {:>7} {:>6} {:>6} {:>6} {:>6} {:>6} {:>5}",
        "case", "frames", "fps", "mean", "p99", "max", "scene", "draws", ">8.3"
    );
    let mut all = Vec::new();
    let mut worst: (f64, String) = (0.0, String::new());
    for (i, start) in bench.starts.iter().enumerate() {
        let end = bench.starts.get(i + 1).copied().unwrap_or(frames.len());
        let mut times = Vec::new();
        let (mut scene, mut draws) = (0.0, 0);
        for frame in &frames[*start..end] {
            times.push(ms(frame.delta));
            scene += ms(frame.sample.scene);
            draws += frame.sample.draws;
        }
        if times.is_empty() {
            continue;
        }
        let case = bench.cases[i];
        let name = case.name();
        let n = times.len() as f64;
        let mean = times.iter().sum::<f64>() / n;
        let over = times.iter().filter(|t| **t > BUDGET_MS).count();
        all.extend_from_slice(&times);
        times.sort_by(f64::total_cmp);
        let max = *times.last().unwrap_or(&0.0);
        let p99 = percentile(&times, 0.99);
        if p99 > worst.0 {
            worst = (p99, name.clone());
        }
        println!(
            "{name:<34} {:>6} {:>7.0} {mean:>6.2} {p99:>6.2} {max:>6.2} {:>6.2} {:>6} {over:>5}",
            times.len(),
            1000.0 / mean,
            scene / n,
            draws / times.len(),
        );
    }
    if all.is_empty() {
        return;
    }
    let n = all.len() as f64;
    let mean = all.iter().sum::<f64>() / n;
    let over = all.iter().filter(|t| **t > BUDGET_MS).count();
    all.sort_by(f64::total_cmp);
    println!(
        "\nall: {} frames, {:.0} fps mean, p99 {:.2} ms, max {:.2} ms, {over} over {BUDGET_MS:.2} ms ({:.2}%)",
        all.len(),
        1000.0 / mean,
        percentile(&all, 0.99),
        all.last().unwrap_or(&0.0),
        100.0 * over as f64 / n
    );
    println!("worst p99: {} ({:.2} ms)", worst.1, worst.0);
    if !complete {
        println!("stopped early");
    }
}
