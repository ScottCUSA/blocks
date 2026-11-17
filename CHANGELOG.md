# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Benchmark: `--bench [filter] [seconds]`, or T in the animation viewer.
- Frame stats in the animation viewer; C records them to a CSV.
- Attract mode: after 30 s idle at the menu, the computer plays a demo game
  from level 7.
- Storm level up: a white flash, a barrage of six lightning bolts sweeping
  across the sea, and the sea heaving up into the new level's state.
- Storm background: rain, wind and a three-layer sea under rolling clouds.
  The storm builds with the level, reaching full strength at level 12, and
  a whiteout of rain and mist creeps in from level 10.
- Five kinds of lightning: forked, double, sheet, crawling and distant, with
  a glow along each bolt. Line clears light up the clouds.
- Storms, a dark 3/4 waltz music theme, with a loop option and a length
  check in `tools/make_chiptune.py`.
- Audio option to keep the music playing when the window loses focus.
- Shanty, a stormy 6/8 sea shanty music theme in D minor with a folk-dance
  bridge, and a jig drum pattern in `tools/make_chiptune.py`.
- `storm`, `storm-max`, `storm-whiteout` and `options-audio` screenshot scenes.
- `demo` screenshot scene.

### Changed

- Song, Music and Sound moved to a new Audio page in the options.

## [0.5.1] - 2026-11-15

### Changed

- Smaller release binary: fat LTO, one codegen unit and size optimization.
- Sound effects are Ogg Vorbis instead of WAV.

### Removed

- A music track.
- Gamepad support from ggez, which the game never used.

## [0.5.0] - 2026-11-15

### Added

- Standard and Hard modes. Standard stops speeding up at level 15; Hard
  keeps going to instant drops at level 19.
- Points for soft drops (1 per row) and hard drops (2 per row).
- Level progress bar under the playfield that fills as you clear lines and
  bursts on level up.
- Top 10 high score table with arcade-style initials entry.
- Options menu: mode, song, background, block style, board style, music
  volume and sound volume. Options and window state are saved between runs.
- Five music tracks: Dance, Bounce, Baroque, Nocturne and Original. Music
  speeds up with the level without changing pitch.
- Sound effects for moving, rotating, locking, hard drops, holding, line
  clears, quads, level ups, new high scores and menu navigation, plus a game
  over jingle.
- Backgrounds (stars, plasma, off) that react to line clears and level ups
  and freeze on game over.
- Block styles (bevel, flat, neon, retro, jelly) and board styles (grid,
  dots, checker, clean).
- Effects: drop trails, landing impacts and dust, screen flash and shake,
  line clear callouts and a quad glint.
- Game over animation: rows gray out from the bottom up, then the overlay
  drops in.
- Selectable main and pause menus, and a high scores screen.
- Resizable window with pixel-snapped scaling.
- Release packages built in CI when a `v*` tag is pushed: Linux AppImage
  and tarball, Windows zip, and universal macOS `.dmg`.
- Packaging scripts in `tools/` for building each package locally.
- `THIRD_PARTY_LICENSES.md`, generated with cargo-about and included in
  every release package.
- cargo-deny checks for licenses, sources and advisories in CI.
- Debug animation viewer and screenshot scenes.

### Changed

- Renamed the project to Blocks.
- Switched the game engine from macroquad to ggez.
- Simplified controls and smoother held-key handling.
- New piece color palette and app icon to match.
- Redrawn playfield and in-game UI.
- On macOS, resources are also loaded from the app bundle.
- CI uses `Swatinem/rust-cache` and `actions/checkout@v4`.
- Updated dependencies and pinned exact versions.

### Fixed

- Music volume not applying correctly.

## [0.3.2] - 2023-01-31

### Changed

- Builds on the stable toolchain.
- The Windows console window is only hidden in release builds.

### Fixed

- Panic from an out-of-bounds index when a piece reached the top of the
  playfield.

## [0.3.1] - 2023-01-27

### Fixed

- A piece could lock in mid-air during lockdown.
- Clearing a line left stale state on locked blocks at the very top of the
  playfield.

## [0.3.0] - 2023-01-26

### Added

- Wall kicks.

## [0.2.2] - 2023-01-23

### Added

- Lockdown delay.
- Game over when a piece locks entirely above the playfield.

### Changed

- Smoother difficulty ramp-up.
- Games start at level 0.
- Resized the icon to 256x256.
- The console window is hidden on Windows.

### Fixed

- Lines above row 16 did not clear correctly.
- Restarting a game began at level 1.
- Several line clearing and piece state bugs.

## [0.2.1] - 2023-01-08

### Fixed

- Unpausing the game.

## [0.2.0] - 2023-01-08

### Added

- First release: playfield, falling pieces from a shuffled bag, line
  clears and scoring.
- Keyboard controls for moving, rotating, soft and hard dropping, and
  holding a piece, with repeat on held keys.
- Ghost piece, next piece preview, and score and level display.
- Menu and pause screen.
- Looping background music with volume controls, starting at 50%.
- On-screen help listing the controls.
- Window icon.
- CI workflow.

[Unreleased]: https://github.com/ScottCUSA/blocks/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/ScottCUSA/blocks/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/ScottCUSA/blocks/compare/79e4126...v0.5.0
[0.3.2]: https://github.com/ScottCUSA/blocks/compare/v0.3.1...79e4126
[0.3.1]: https://github.com/ScottCUSA/blocks/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ScottCUSA/blocks/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/ScottCUSA/blocks/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ScottCUSA/blocks/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ScottCUSA/blocks/releases/tag/v0.2.0
