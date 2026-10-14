use smarterr::error;

error!(pub FileError { pub code: u32 } => "File error {code}");
error!(pub VersionError { pub version: u32 } => "Version {version}");

pub mod api {
    use super::*;
    use smarterr::errorset::errorset;

    #[errorset]
    pub fn read(version: bool) -> Result<(), (FileError, VersionError)> {
        if version {
            Err(VersionError { version: 7 }.into())
        } else {
            Err(FileError { code: 42 }.into())
        }
    }
}

pub mod other {
    use super::*;
    use smarterr::errorset::errorset;

    #[errorset]
    pub fn read() -> Result<(), (FileError,)> {
        Err(FileError { code: 99 }.into())
    }
}

pub use api::ReadErrors as PublicReadErrors;

#[smarterr::errorset::errorset(from(api::ReadErrors, other::ReadErrors))]
pub type ApiErrors = (FileError, VersionError);

pub use ApiErrors as PublicApiErrors;
