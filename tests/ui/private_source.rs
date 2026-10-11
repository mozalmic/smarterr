use smarterr::{error, errorset::errorset};

error!(pub FileError {} => "File");

mod hidden {
    use super::*;

    #[errorset(export(hidden_source))]
    fn read() -> Result<(), (FileError,)> {
        Ok(())
    }
}

// Even the publicly exported internal descriptor cannot bypass type privacy.
#[errorset(from(hidden::ReadErrors => __errorset_hidden_source))]
fn load() -> Result<(), (FileError,)> {
    Ok(())
}

fn main() {}
