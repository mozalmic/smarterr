use smarterr::{error, errorset::errorset};

error!(pub FileError {} => "File");

mod hidden {
    use super::*;

    #[errorset(mod sets, export(hidden_module_source))]
    pub fn read() -> Result<(), (FileError,)> {
        Ok(())
    }
}

#[errorset(from(hidden::sets::ReadErrors => __errorset_hidden_module_source))]
fn load() -> Result<(), (FileError,)> {
    Ok(())
}

fn main() {}
