use smarterr::{error, errorset::errorset};

error!(pub FileError {} => "File");

mod hidden {
    use super::*;

    #[errorset(mod sets)]
    pub fn read() -> Result<(), (FileError,)> {
        Ok(())
    }
}

#[errorset(from(hidden::sets::ReadErrors))]
fn load() -> Result<(), (FileError,)> {
    Ok(())
}

fn main() {}
