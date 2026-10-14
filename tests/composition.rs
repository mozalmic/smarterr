use smarterr::{error, errorset::errorset, ErrorSet};

error!(pub FileError { pub code: u32 } => "File {code}");
error!(pub VersionError {} => "Version");
error!(pub CryptoError {} => "Crypto");
error!(pub InvalidKey {} => "Invalid key");

#[errorset]
fn read(choice: u8) -> Result<(), (FileError, VersionError, CryptoError)> {
    match choice {
        0 => Ok(()),
        1 => Err(FileError { code: 7 }.into()),
        2 => Err(VersionError {}.into()),
        _ => Err(CryptoError {}.into()),
    }
}

#[errorset]
fn decrypt() -> Result<(), (CryptoError, FileError)> {
    Err(FileError { code: 8 }.into())
}

#[errorset(from(ReadErrors, DecryptErrors))]
fn full(choice: u8) -> Result<(), (FileError, VersionError, CryptoError, InvalidKey, FileError)> {
    read(choice)?;
    decrypt()?;
    Ok(())
}

#[errorset(from(FullErrors))]
fn outer(choice: u8) -> Result<(), (FileError, VersionError, CryptoError, InvalidKey)> {
    full(choice)?;
    Ok(())
}

#[errorset(from(ReadErrors))]
fn partial(choice: u8) -> Result<(), (CryptoError, InvalidKey)> {
    match read(choice) {
        Ok(()) => {}
        Err(error) => match error.split::<PartialErrors>()? {
            ReadErrorsForPartialErrors::FileError(e) => assert_eq!(e.code, 7),
            ReadErrorsForPartialErrors::VersionError(_) => {}
        },
    }
    Ok(())
}

#[test]
fn exact_partial_contract() {
    for choice in 0..3 {
        assert!(partial(choice).is_ok());
    }
    match partial(3) {
        Err(PartialErrors::CryptoError(_)) => {}
        Err(PartialErrors::InvalidKey(_)) => panic!("unexpected invalid key"),
        Ok(()) => panic!("crypto error must propagate"),
    }
}

#[test]
fn intersection_and_nested_conversion() {
    assert!(matches!(outer(1), Err(OuterErrors::FileError(FileError { code: 7 }))));
    assert!(matches!(outer(2), Err(OuterErrors::VersionError(_))));
    assert!(matches!(outer(3), Err(OuterErrors::CryptoError(_))));
    assert!(matches!(outer(0), Err(OuterErrors::FileError(FileError { code: 8 }))));
    let e = ReadErrors::CryptoError(CryptoError {});
    assert!(matches!(e.split::<FullErrors>(), Err(FullErrors::CryptoError(_))));
}

mod a {
    smarterr::error!(pub Error {} => "a");
}

mod b {
    smarterr::error!(pub Error {} => "b");
}

#[errorset(export)]
fn same_names(choice: bool) -> Result<(), (a::Error, b::Error)> {
    if choice {
        Err(a::Error {}.into())
    } else {
        Err(b::Error {}.into())
    }
}

#[errorset(from(SameNamesErrors))]
fn same_names_target(choice: bool) -> Result<(), (a::Error, b::Error)> {
    same_names(choice)?;
    Ok(())
}

#[test]
fn distinct_same_named_types() {
    match same_names_target(true) {
        Err(SameNamesTargetErrors::AError(_)) => {}
        Err(SameNamesTargetErrors::BError(_)) | Ok(()) => panic!("expected a"),
    }
    assert!(matches!(
        same_names_target(false),
        Err(SameNamesTargetErrors::BError(_))
    ));
}

#[errorset(from(ReadErrors { FileError => ReexportedFile }))]
fn renamed() -> Result<(), (ReexportedFile, CryptoError)> {
    match read(1).unwrap_err().split::<RenamedErrors>()? {
        ReadErrorsForRenamedErrors::VersionError(_) => {}
    }
    Ok(())
}

use FileError as ReexportedFile;

#[test]
fn explicit_alias_mapping() {
    assert!(matches!(renamed(), Err(RenamedErrors::ReexportedFile(_))));
}

error!(pub SemanticError<ReadErrors> {} => "Semantic read failure");

#[errorset]
fn semantic() -> Result<(), (SemanticError,)> {
    Err(SemanticError { source: read(2).unwrap_err() }.into())
}

#[test]
fn source_is_not_expanded() {
    match semantic() {
        Err(SemanticErrors::SemanticError(e)) => assert!(matches!(e.source, ReadErrors::VersionError(_))),
        Ok(()) => panic!("expected semantic error"),
    }
}

mod remote {
    use super::*;

    #[errorset(pub mod sets, export(remote_read))]
    pub fn read_remote() -> Result<(), (FileError, VersionError)> {
        Err(VersionError {}.into())
    }
}

#[errorset(from(remote::sets::ReadRemoteErrors => remote::sets::remote_read))]
fn across_modules() -> Result<(), (FileError,)> {
    match remote::read_remote().unwrap_err().split::<AcrossModulesErrors>()? {
        ReadRemoteErrorsForAcrossModulesErrors::VersionError(_) => {}
    }
    Ok(())
}

#[test]
fn module_and_custom_descriptor() {
    assert!(across_modules().is_ok());
}

#[errorset(from(LaterErrors))]
fn before_source() -> Result<(), (FileError,)> {
    later()?;
    Ok(())
}

#[errorset]
fn later() -> Result<(), (FileError,)> {
    Err(FileError { code: 9 }.into())
}

#[test]
fn declaration_order_is_not_required() {
    assert!(matches!(before_source(), Err(BeforeSourceErrors::FileError(_))));
}

struct Service<T>(T);

#[errorset(pub mod method_sets)]
impl<T> Service<T> {
    #[errorset(export(service_read))]
    pub fn service_read(&self) -> Result<(), (FileError, VersionError)> {
        Err(FileError { code: 10 }.into())
    }

    #[errorset(from(method_sets::ServiceReadErrors => method_sets::service_read))]
    pub fn service_load(&self) -> Result<(), (FileError,)> {
        match self
            .service_read()
            .unwrap_err()
            .split::<method_sets::ServiceLoadErrors>()?
        {
            method_sets::ServiceReadErrorsForServiceLoadErrors::VersionError(_) => {}
        }
        Ok(())
    }
}

#[test]
fn impl_methods_and_generic_success_scope() {
    assert!(matches!(
        Service(42).service_load(),
        Err(method_sets::ServiceLoadErrors::FileError(_))
    ));
}

#[derive(Debug, thiserror::Error)]
#[error("Concrete generic error {0}")]
pub struct GenericError<T: std::fmt::Debug + std::fmt::Display>(T);

#[errorset(export)]
fn concrete<T>(value: T) -> Result<T, (GenericError<u32>,)> {
    Ok(value)
}

#[errorset(from(ConcreteErrors))]
fn concrete_target() -> Result<u32, (GenericError<u32>,)> {
    concrete(42).map_err(Into::into)
}

#[test]
fn concrete_generic_member_and_map_err() {
    assert_eq!(concrete_target().unwrap(), 42);
    let error = ConcreteErrors::GenericError(GenericError(17));
    assert_eq!(error.downcast_ref::<GenericError<u32>>().unwrap().0, 17);
}

mod first {
    use super::*;

    pub struct Storage;

    #[errorset]
    impl Storage {
        #[errorset(export)]
        pub fn open(&self) -> Result<(), (FileError,)> {
            Err(FileError { code: 21 }.into())
        }
    }
}

mod second {
    use super::*;

    pub struct Connection;

    #[errorset]
    impl Connection {
        #[errorset(export)]
        pub fn open(&self) -> Result<(), (VersionError,)> {
            Err(VersionError {}.into())
        }
    }
}

pub use first::OpenErrors as StorageOpenErrors;

#[errorset(from(StorageOpenErrors, second::OpenErrors))]
fn both_open() -> Result<(), (FileError, VersionError)> {
    first::Storage.open()?;
    second::Connection.open()?;
    Ok(())
}

#[errorset(from(second::OpenErrors => second::OpenErrorsDescriptor))]
fn legacy_descriptor() -> Result<(), (VersionError,)> {
    second::Connection.open()?;
    Ok(())
}

#[test]
fn exported_method_names_are_module_scoped_and_reexports_carry_metadata() {
    assert!(matches!(both_open(), Err(BothOpenErrors::FileError(FileError { code: 21 }))));
    assert!(matches!(
        second::Connection.open().unwrap_err().split::<BothOpenErrors>(),
        Err(BothOpenErrors::VersionError(_))
    ));
    assert!(matches!(legacy_descriptor(), Err(LegacyDescriptorErrors::VersionError(_))));
}

macro_rules! repeated_source {
    ($module:ident) => {
        mod $module {
            use super::*;

            #[errorset(export)]
            pub fn repeated() -> Result<(), (FileError,)> {
                Err(FileError { code: 23 }.into())
            }
        }
    };
}

repeated_source!(macro_first);
repeated_source!(macro_second);

#[errorset(from(macro_first::RepeatedErrors, macro_second::RepeatedErrors))]
fn repeated_target() -> Result<(), (FileError,)> {
    macro_first::repeated()?;
    macro_second::repeated()?;
    Ok(())
}

#[errorset(from(macro_first::RepeatedErrors, macro_second::RepeatedErrors))]
fn repeated_partial() -> Result<(), (VersionError,)> {
    match macro_first::repeated().unwrap_err().split::<RepeatedPartialErrors>()? {
        MacroFirstRepeatedErrorsForRepeatedPartialErrors::FileError(error) => assert_eq!(error.code, 23),
    }
    match macro_second::repeated().unwrap_err().split::<RepeatedPartialErrors>()? {
        MacroSecondRepeatedErrorsForRepeatedPartialErrors::FileError(error) => assert_eq!(error.code, 23),
    }
    Ok(())
}

#[test]
fn identical_macro_generated_sources_do_not_collide() {
    assert_eq!(repeated_target().unwrap_err().downcast_ref::<FileError>().unwrap().code, 23);
    assert!(repeated_partial().is_ok());
}

#[test]
fn downcast_borrows_active_atom_and_preserves_its_source() {
    use std::error::Error;

    let error = read(1).unwrap_err();
    let atom = error.downcast_ref::<FileError>().unwrap();
    assert!(error.source().is_none());
    assert!(std::ptr::eq(atom, error.as_error().downcast_ref::<FileError>().unwrap()));
    assert_eq!(atom.code, 7);
    assert!(std::ptr::eq(atom, match &error {
        ReadErrors::FileError(atom) => atom,
        _ => panic!("expected file error"),
    }));
    assert!(error.downcast_ref::<VersionError>().is_none());

    let semantic = semantic().unwrap_err();
    assert!(semantic.downcast_ref::<SemanticError>().is_some());
    assert!(semantic.downcast_ref::<VersionError>().is_none());
    assert!(semantic.source().unwrap().downcast_ref::<ReadErrors>().is_some());
    assert_eq!(semantic.to_string(), "Semantic read failure");

    let remainder = read(1).unwrap_err().split::<PartialErrors>().unwrap();
    assert_eq!(remainder.downcast_ref::<FileError>().unwrap().code, 7);
    assert!(remainder.downcast_ref::<CryptoError>().is_none());
}

#[errorset]
fn infallible() -> Result<(), ()> {
    Ok(())
}

#[test]
fn empty_sets_and_empty_remainders_support_introspection() {
    fn assert_atom<T: ErrorSet>() {}
    assert_atom::<InfallibleErrors>();
    assert_atom::<ReadErrorsForFullErrors>();
    assert!(infallible().is_ok());
}

#[test]
fn one_trait_bound_classifies_unrelated_error_sets() {
    fn file_code(error: &impl ErrorSet) -> Option<u32> {
        error.downcast_ref::<FileError>().map(|atom| atom.code)
    }

    assert_eq!(file_code(&read(1).unwrap_err()), Some(7));
    assert_eq!(file_code(&first::Storage.open().unwrap_err()), Some(21));
    assert_eq!(file_code(&second::Connection.open().unwrap_err()), None);
    assert_eq!(file_code(&read(1).unwrap_err().split::<PartialErrors>().unwrap()), Some(7));
}

#[cfg(any())]
#[errorset(export)]
fn disabled() -> Result<(), (MissingType,)> {
    unreachable!()
}

#[cfg(any())]
#[errorset(from(MissingSet))]
fn disabled_conversion() -> Result<(), (MissingType,)> {
    unreachable!()
}

#[cfg_attr(all(), errorset(export(cfg_export)))]
fn configured() -> Result<(), (FileError,)> {
    Err(FileError { code: 11 }.into())
}

#[cfg_attr(all(), errorset(from(ConfiguredErrors => cfg_export)))]
fn configured_target() -> Result<(), (FileError,)> {
    configured()?;
    Ok(())
}

#[test]
fn compiler_resolved_cfg_attr() {
    assert!(matches!(configured_target(), Err(ConfiguredTargetErrors::FileError(_))));
}
