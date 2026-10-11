use smarterr::{error, errorset::errorset, ErrorSet};

error!(pub OtherError {} => "Other");

#[errorset(from(reader::api::ReadErrors))]
fn partial() -> Result<(), (OtherError,)> {
    match reader::api::read(true).unwrap_err().split::<PartialErrors>()? {
        ReadErrorsForPartialErrors::FileError(_) => {}
    }
    Ok(())
}

fn main() {}
