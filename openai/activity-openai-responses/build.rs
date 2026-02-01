use anyhow::Result;
use wit_bindgen_rust::Opts;

fn main() -> Result<()> {
    Opts {
        generate_all: true,
        ..Default::default()
    }
    .build()
    .generate_to_out_dir(None)?;

    Ok(())
}
