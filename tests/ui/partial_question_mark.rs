use smarterr::errorset::errorset;

#[errorset(from(reader::api::ReadErrors))]
fn partial() -> Result<(), (reader::FileError,)> {
    reader::api::read(true)?;
    Ok(())
}

fn main() {}
