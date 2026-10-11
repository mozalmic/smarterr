use smarterr::{errorset::errorset, ErrorSet};

// Both the crate name and the atomic error name differ from the producer.
pub use reader::FileError as LocalFileError;

#[errorset(export, from(reader::api::ReadErrors {
    FileError => LocalFileError
}))]
pub fn load(version: bool) -> Result<(), (LocalFileError,)> {
    match reader::api::read(version) {
        Ok(()) => {}
        Err(error) => match error.split::<LoadErrors>()? {
            ReadErrorsForLoadErrors::VersionError(e) => assert_eq!(e.version, 7),
        },
    }
    Ok(())
}

#[errorset(from(LoadErrors))]
pub fn outer(version: bool) -> Result<(), (LocalFileError,)> {
    load(version)?;
    Ok(())
}

#[errorset(from(reader::api::ReadErrors))]
pub fn all(version: bool) -> Result<(), (reader::FileError, reader::VersionError)> {
    reader::api::read(version)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_across_crates() {
        assert!(load(true).is_ok());
        assert!(matches!(
            load(false),
            Err(LoadErrors::LocalFileError(LocalFileError { code: 42 }))
        ));
        assert!(matches!(outer(false), Err(OuterErrors::LocalFileError(_))));
        assert!(matches!(all(true), Err(AllErrors::VersionError(_))));
        let e = reader::api::ReadErrors::VersionError(reader::VersionError { version: 8 });
        match e.split::<LoadErrors>() {
            Ok(ReadErrorsForLoadErrors::VersionError(e)) => assert_eq!(e.version, 8),
            Err(_) => panic!("expected handled error"),
        }
    }
}
