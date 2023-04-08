// build.rs

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/rustris256x256.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
