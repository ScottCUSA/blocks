# blocks
Rust clone of a famous blocky game

![screenshot](img/Blocks-Screenshot.png)

# build
Install Rust using https://rustup.rs. Clone or download this repo, and run using cargo.

> \> cargo run

# package
Each script builds a release and writes the package to `target/dist`.
Pushing a `v*` tag runs the same scripts in CI and publishes a GitHub release.

| Package | Script | Run on |
|---|---|---|
| Linux AppImage | `tools/build_appimage.sh` | Linux |
| Linux tarball | `tools/build_tarball.sh` | Linux |
| Windows zip | `tools/build_windows_zip.sh` | Windows (Git Bash) |
| macOS dmg | `tools/build_macos_dmg.sh` | macOS |

Optional arguments: `[version] [out_dir]`.

# license
Blocks is licensed under the [Mozilla Public License 2.0](LICENSE).

It includes open source crates under their own licenses, listed in
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md). The 04b30 font is freeware
([details](resources/LICENSE.txt)).

After changing dependencies, regenerate the list and check the dependency
policy in `deny.toml`:

```
cargo install --locked --features cli cargo-about
cargo install --locked cargo-deny
cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
cargo deny check
```
