use smarterr::{error, errorset::errorset};

error!(pub FileError {} => "File");

mod hidden {
    use super::*;

    #[errorset]
    fn read() -> Result<(), (FileError,)> {
        Ok(())
    }
}

#[errorset(from(hidden::ReadErrors))]
fn load() -> Result<(), (FileError,)> {
    Ok(())
}

fn main() {}
