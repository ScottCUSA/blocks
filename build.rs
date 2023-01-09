// build.rs

#[cfg(windows)]
use winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/rustris.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
