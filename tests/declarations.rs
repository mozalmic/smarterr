#![deny(private_interfaces, private_bounds)]

use smarterr::{error, errorset::errorset, ErrorSet};

error!(pub FileError { pub code: u32 } => "File {code}");
error!(pub VersionError {} => "Version");
error!(pub CryptoError {} => "Crypto");

#[errorset]
fn read() -> Result<(), (FileError, VersionError)> {
    Err(FileError { code: 7 }.into())
}

/// The public API contract is an enum, not a tuple alias.
#[errorset(from(ReadErrors, CryptoOnly))]
pub type ApiErrors = (FileError, VersionError, CryptoError);

#[errorset]
type CryptoOnly = (CryptoError,);

#[errorset(from(ApiErrors))]
pub(crate) type StorageErrors = (FileError,);

#[errorset(from(ApiErrors))]
fn load() -> Result<(), (FileError, VersionError, CryptoError)> {
    let result: Result<(), ApiErrors> = read().map_err(Into::into);
    result?;
    Ok(())
}

#[test]
fn declaration_is_target_and_source() {
    let error: ApiErrors = read().unwrap_err().into();
    assert_eq!(error.downcast_ref::<FileError>().unwrap().code, 7);
    assert_eq!(error.to_string(), "File 7");
    assert!(std::error::Error::source(&error).is_none());
    assert!(matches!(
        error.split::<StorageErrors>(),
        Err(StorageErrors::FileError(FileError { code: 7 }))
    ));
    assert!(matches!(
        ApiErrors::VersionError(VersionError {}).split::<StorageErrors>(),
        Ok(ApiErrorsForStorageErrors::VersionError(_))
    ));
    assert!(matches!(load(), Err(LoadErrors::FileError(_))));
    let error: ApiErrors = CryptoOnly::from(CryptoError {}).into();
    assert!(error.downcast_ref::<CryptoError>().is_some());
}

mod visibility {
    use super::*;

    #[errorset(export(storage_descriptor))]
    pub(super) type ParentErrors = (FileError, VersionError);

    #[errorset(from(ParentErrors => storage_descriptor))]
    pub(in crate::visibility) type ScopedErrors = (FileError,);

    #[errorset(from(ScopedErrors))]
    type PrivateErrors = (FileError,);

    pub(super) fn check() {
        let error = ParentErrors::FileError(FileError { code: 9 });
        let error = error.split::<ScopedErrors>().unwrap_err();
        let error: PrivateErrors = error.into();
        assert_eq!(error.downcast_ref::<FileError>().unwrap().code, 9);
    }
}

#[errorset(from(visibility::ParentErrors))]
type ParentTarget = (FileError, VersionError);

use FileError as LocalFileError;

#[errorset(from(ApiErrors { FileError => LocalFileError }))]
type Renamed = (LocalFileError,);

#[test]
fn declaration_supports_explicit_variant_mapping() {
    let error = ApiErrors::FileError(FileError { code: 17 });
    assert!(matches!(
        error.split::<Renamed>(),
        Err(Renamed::LocalFileError(LocalFileError { code: 17 }))
    ));
}

#[test]
fn visibility_and_named_descriptor() {
    visibility::check();
    let error: ParentTarget = visibility::ParentErrors::VersionError(VersionError {}).into();
    assert!(error.downcast_ref::<VersionError>().is_some());
}

#[errorset]
type Empty = ();

#[errorset(from(Empty))]
type EmptyTarget = ();

#[cfg(any())]
#[errorset(from(DoesNotExist))]
type Disabled = (MissingAtom,);

#[errorset]
#[cfg_attr(all(), cfg(any()))]
type AlsoDisabled = (MissingAtom,);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Generic {0}")]
pub struct GenericError<T: std::fmt::Debug + std::fmt::Display>(T);

#[errorset]
type Concrete = (GenericError<u32>,);

#[test]
fn empty_and_concrete_generic_sets() {
    fn assert_empty_conversion<T: From<Empty>>() {}
    assert_empty_conversion::<EmptyTarget>();
    let error: Concrete = GenericError(42u32).into();
    assert_eq!(error.downcast_ref::<GenericError<u32>>().unwrap().0, 42);
}

#[errorset]
#[derive(PartialEq, Eq)]
type Comparable = (GenericError<u32>,);

#[test]
fn declaration_attributes_are_preserved() {
    assert_eq!(
        Comparable::from(GenericError(3u32)),
        Comparable::from(GenericError(3u32))
    );
}
