// #![windows_subsystem = "windows"]

use macroquad::{
    audio::{load_sound, play_sound, PlaySoundParams},
    prelude::*,
    text::load_ttf_font,
    window::Conf,
};
use simplelog::{format_description, ConfigBuilder};

mod controls;
mod game;
mod playfield;
mod rustomino;
mod view;

const VIEW_WH: [i32; 2] = [1024, 768];
const ASSETS_FOLDER: &str = "assets";
const MUSIC_VOL: f32 = 0.1;
// const MINIMUM_FRAME_TIME: f32 = 1. / 60.; // used to limit framerate to 60fps

fn window_conf() -> Conf {
    Conf {
        window_title: "Rustris".to_owned(),
        window_width: VIEW_WH[0],
        window_height: VIEW_WH[1],
        window_resizable: false,
        ..Default::default()
    }
}

// https://tetris.wiki/Tetris_Guideline
// TODO: implement rotation wall kicks
// TODO: debug repeat collision call wierdness
// TODO: load icon for rustris window
// https://docs.rs/macroquad/0.3.25/macroquad/texture/struct.Image.html

#[macroquad::main(window_conf())]
async fn main() {
    let config = ConfigBuilder::new()
        .set_time_format_custom(format_description!("[hour]:[minute]:[second].[subsecond]"))
        .build();
    // initialize the logger
    if simplelog::TermLogger::init(
        simplelog::LevelFilter::Debug,
        config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .is_err()
    {
        eprintln!("WARNING: unable to initialize logger");
    }
    log::info!("startup: initializing Rustris;");
    log::info!("loading Resources");

    // find our assets path
    let assets_path = find_folder::Search::ParentsThenKids(2, 2)
        .for_folder(ASSETS_FOLDER)
        .expect("unable to find assets folder");

    // load the font
    let font_path = assets_path.join("04b30.ttf");
    log::info!("loading font: {:?}", font_path);
    let font = load_ttf_font(&font_path.to_string_lossy())
        .await
        .expect("unable to load font");

    // setup two different sized "fonts"
    let font_20pt = TextParams {
        font,
        font_size: 20,
        ..Default::default()
    };
    let font_30pt = TextParams {
        font,
        font_size: 30,
        ..Default::default()
    };

    // init the game and control states
    let mut game = game::RustrisGame::new(playfield::RustrisPlayfield::new());
    let mut controls = controls::ControlStates::default();

    // load the background music
    let background_path = assets_path.join("background.ogg");
    log::info!("loading background music: {:?}", background_path);
    let background_music = load_sound(&background_path.to_string_lossy())
        .await
        .expect("unable to load background music");

    // play background music
    let mut music_volume = MUSIC_VOL;
    log::info!("playing background music at volume: {music_volume}");
    play_sound(
        background_music,
        PlaySoundParams {
            looped: true,
            volume: music_volume,
        },
    );

    loop {
        // attempt to limit framerate to 60fps
        // let frame_time = get_frame_time();
        // // log::debug!("frame_time: {}", frame_time);
        // if frame_time < MINIMUM_FRAME_TIME {
        //     let time_to_sleep = (MINIMUM_FRAME_TIME - frame_time) * 1000.;
        //     // log::debug!("sleeping: {}", time_to_sleep);
        //     std::thread::sleep(std::time::Duration::from_millis(time_to_sleep as u64));
        // }

        clear_background(view::BACKGROUND_COLOR);

        // // draw FPS
        // draw_text(
        //     &get_fps().to_string(),
        //     VIEW_DIMENSIONS[0] as f32 - 50.0,
        //     50.0,
        //     30.,
        //     WHITE,
        // );

        // handle global controls
        controls::handle_global_controls(&background_music, &mut music_volume);

        // run the rustris game update
        game.update(&mut controls);

        // draw the menus, game, overlays, etc.
        view::draw(&game, &font_20pt, &font_30pt);

        next_frame().await;
    }
}
