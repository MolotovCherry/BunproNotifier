use std::{env, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    {
        static MANIFEST: &str = include_str!("manifest.xml");

        winres::WindowsResource::new()
            .set_icon("_assets/logo.ico") // ordinal 1
            .set_manifest(MANIFEST)
            .compile()?;

        println!("cargo::rerun-if-changed=_assets/logo.ico");
        println!("cargo::rerun-if-changed=manifest.xml");
        println!("cargo::rerun-if-changed=Cargo.toml");
    }

    let i = image::open("_assets/logo-original.png").unwrap();
    let raw_data = i.into_rgba8().into_vec();

    let var = env::var("OUT_DIR").unwrap();
    let path = Path::new(&var).join("tray.bin");
    fs::write(path, raw_data).expect("tray.bin write to succeed");

    println!("cargo::rerun-if-changed=_assets/logo-original.png");

    Ok(())
}
