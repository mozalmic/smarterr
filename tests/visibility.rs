#![deny(private_interfaces, private_bounds)]

use smarterr::{error, errorset::errorset, ErrorSet};

error!(pub FileError {} => "File");
error!(pub VersionError {} => "Version");

mod functions {
    use super::*;

    error!(PrivateError {} => "Private");

    #[errorset(export(private_source))]
    fn private_read() -> Result<(), (PrivateError, VersionError)> {
        Err(VersionError {}.into())
    }

    #[errorset(from(PrivateReadErrors => private_source))]
    fn private_load() -> Result<(), (PrivateError,)> {
        match private_read().unwrap_err().split::<PrivateLoadErrors>()? {
            PrivateReadErrorsForPrivateLoadErrors::VersionError(_) => {}
        }
        Ok(())
    }

    #[errorset(from(PrivateReadErrors => private_source))]
    pub fn public_load_from_private_source() -> Result<(), (VersionError,)> {
        match private_read()
            .unwrap_err()
            .split::<PublicLoadFromPrivateSourceErrors>()?
        {
            PrivateReadErrorsForPublicLoadFromPrivateSourceErrors::PrivateError(_) => {}
        }
        Ok(())
    }

    #[errorset(export(crate_source))]
    pub(crate) fn crate_read() -> Result<(), (FileError, VersionError)> {
        Err(VersionError {}.into())
    }

    #[errorset(export(super_source))]
    pub(super) fn super_read() -> Result<(), (FileError, VersionError)> {
        Err(VersionError {}.into())
    }

    #[errorset(export(in_source))]
    pub(in crate::functions) fn scoped_read() -> Result<(), (FileError, VersionError)> {
        Err(VersionError {}.into())
    }

    pub(super) mod child {
        use super::*;

        #[errorset(from(super::PrivateReadErrors => super::private_source))]
        pub(super) fn load_parent_private() -> Result<(), (VersionError,)> {
            match super::private_read().unwrap_err().split::<LoadParentPrivateErrors>()? {
                PrivateReadErrorsForLoadParentPrivateErrors::PrivateError(_) => {}
            }
            Ok(())
        }

        #[errorset(from(super::ScopedReadErrors => super::in_source))]
        pub(super) fn load_scoped() -> Result<(), (FileError,)> {
            match super::scoped_read().unwrap_err().split::<LoadScopedErrors>()? {
                ScopedReadErrorsForLoadScopedErrors::VersionError(_) => {}
            }
            Ok(())
        }
    }

    pub(super) fn check_private() {
        assert!(private_load().is_ok());
        assert!(matches!(
            child::load_parent_private(),
            Err(child::LoadParentPrivateErrors::VersionError(_))
        ));
        assert!(child::load_scoped().is_ok());
    }
}

#[errorset(from(functions::CrateReadErrors => functions::crate_source))]
pub(crate) fn load_crate() -> Result<(), (FileError,)> {
    match functions::crate_read().unwrap_err().split::<LoadCrateErrors>()? {
        CrateReadErrorsForLoadCrateErrors::VersionError(_) => {}
    }
    Ok(())
}

#[errorset(from(functions::SuperReadErrors => functions::super_source))]
fn load_super() -> Result<(), (FileError,)> {
    match functions::super_read().unwrap_err().split::<LoadSuperErrors>()? {
        SuperReadErrorsForLoadSuperErrors::VersionError(_) => {}
    }
    Ok(())
}

mod methods {
    use super::*;

    pub(super) struct Service;

    #[errorset(pub(super) mod sets)]
    impl Service {
        #[errorset(export(method_private_source))]
        fn private_method(&self) -> Result<(), (FileError, VersionError)> {
            Err(VersionError {}.into())
        }

        #[errorset(from(sets::PrivateMethodErrors => sets::method_private_source))]
        pub(crate) fn crate_method(&self) -> Result<(), (FileError,)> {
            match self.private_method().unwrap_err().split::<sets::CrateMethodErrors>()? {
                sets::PrivateMethodErrorsForCrateMethodErrors::VersionError(_) => {}
            }
            Ok(())
        }
    }

    pub(super) struct NoModuleService;

    #[errorset]
    impl NoModuleService {
        #[errorset(export(no_module_source))]
        pub(super) fn source(&self) -> Result<(), (FileError, VersionError)> {
            Err(VersionError {}.into())
        }

        #[errorset(from(SourceErrors => no_module_source))]
        fn target(&self) -> Result<(), (FileError,)> {
            match self.source().unwrap_err().split::<TargetErrors>()? {
                SourceErrorsForTargetErrors::VersionError(_) => {}
            }
            Ok(())
        }
    }

    pub(super) fn check_no_module() {
        assert!(NoModuleService.target().is_ok());
    }
}

mod generated_module {
    use super::*;

    #[errorset(pub(crate) mod source_sets, export(module_source))]
    pub(crate) fn read() -> Result<(), (FileError, VersionError)> {
        Err(VersionError {}.into())
    }

    #[errorset(pub(super) mod target_sets, from(source_sets::ReadErrors => source_sets::module_source))]
    pub(super) fn load() -> Result<(), (FileError,)> {
        match read().unwrap_err().split::<target_sets::LoadErrors>()? {
            target_sets::ReadErrorsForLoadErrors::VersionError(_) => {}
        }
        Ok(())
    }
}

#[test]
fn function_visibility_and_descendant_access() {
    functions::check_private();
    assert!(matches!(
        functions::public_load_from_private_source(),
        Err(functions::PublicLoadFromPrivateSourceErrors::VersionError(_))
    ));
    assert!(load_crate().is_ok());
    assert!(load_super().is_ok());
}

#[test]
fn impl_visibility_with_and_without_generated_module() {
    assert!(methods::Service.crate_method().is_ok());
    methods::check_no_module();
}

#[test]
fn generated_module_visibility() {
    assert!(generated_module::load().is_ok());
}
