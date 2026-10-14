use smarterr::errorset::errorset;

#[errorset(from(reader::api::ReadErrors))]
fn partial() -> Result<(), (reader::FileError,)> {
    reader::api::read(true)?;
    Ok(())
}

#[errorset(from(reader::api::ReadErrors))]
type NamedPartial = (reader::FileError,);

fn named_partial() -> Result<(), NamedPartial> {
    reader::api::read(true)?;
    Ok(())
}

fn main() {}
