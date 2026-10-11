use smarterr::errorset::errorset;

#[errorset(from(reader::api::ReadErrors))]
fn wrong() -> Result<(), reader::FileError> {
    Ok(())
}

fn main() {}
