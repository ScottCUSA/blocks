# blocks
Rust clone of a famous blocky game

![screenshot](img/Blocks-Screenshot.png)

# build
Install Rust using https://rustup.rs. Clone or download this repo, and run using cargo.

> \> cargo run

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
