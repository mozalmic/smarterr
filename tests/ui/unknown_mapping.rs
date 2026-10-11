use smarterr::errorset::errorset;

#[errorset(from(reader::api::ReadErrors { Missing => FileError }))]
fn wrong() -> Result<(), (reader::FileError,)> {
    Ok(())
}

fn main() {}
