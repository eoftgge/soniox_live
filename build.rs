use image::{ImageFormat, open};

fn main() {
    if cfg!(target_os = "windows") {
        let img = open("assets/icon.png").expect("Couldn't open icon.png");
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let ico_path = std::path::PathBuf::from(out_dir).join("icon.ico");
        img.save_with_format(&ico_path, ImageFormat::Ico)
            .expect("Failed to write icon.png");

        let mut res = winres::WindowsResource::new();
        res.set_icon(
            ico_path
                .to_str()
                .expect("Couldn't convert icon path to string"),
        );
        res.compile().expect("Failed to compile resource");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,Info.plist");
    }
}
