use smarterr::{error, errorset::errorset};

error!(pub FileError {} => "Different type with the same name");

#[errorset(from(reader::api::ReadErrors))]
fn wrong() -> Result<(), (FileError,)> {
    Ok(())
}

fn main() {}
