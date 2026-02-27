use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    {
        static MANIFEST: &str = include_str!("manifest.xml");

        winres::WindowsResource::new()
            .set_icon("_assets/logo.ico") // ordinal 1
            .set_manifest(MANIFEST)
            .compile()?;
    }

    Ok(())
}
