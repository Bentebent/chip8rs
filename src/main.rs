use color_eyre::eyre::Result;
fn main() -> Result<()> {
    color_eyre::install()?;

    let path = r"roms/IBM Logo.ch8";
    chip8rs::run(path.into())?;

    Ok(())
}
