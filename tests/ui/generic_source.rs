use smarterr::errorset::errorset;

#[errorset(from(reader::api::ReadErrors<u32>))]
fn wrong() -> Result<(), (reader::FileError,)> {
    Ok(())
}

fn main() {}
