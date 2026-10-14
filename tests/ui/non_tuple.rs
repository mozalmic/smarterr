use smarterr::errorset::errorset;

#[errorset(from(reader::api::ReadErrors))]
fn wrong() -> Result<(), reader::FileError> {
    Ok(())
}

#[errorset]
fn bare() -> Result<(), reader::FileError> {
    Ok(())
}

struct Service;

#[errorset]
impl Service {
    #[errorset]
    fn method(&self) -> Result<(), (reader::FileError)> {
        Ok(())
    }
}

fn main() {}
