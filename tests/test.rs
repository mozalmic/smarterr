use thiserror::Error;

macro_rules! atomic_err {
    ($evis:vis $name:ident<$source:ty> { $($vis:vis $field:ident: $ty:ty),* $(,)? } => $msg:literal) => {
        #[derive(Error, Debug)]
        #[error($msg)]
        $evis struct $name {
            $(
                $vis $field: $ty,
            )*
            #[source]
            pub source: $source,
        }
    };
    ($evis:vis $name:ident { $($vis:vis $field:ident: $ty:ty),* $(,)? } => $msg:literal) => {
        #[derive(Error, Debug)]
        #[error($msg)]
        $evis struct $name {
            $(
                $vis $field: $ty,
            )*
        }
    };
}

mod atomic_error {
    use std::num::ParseIntError;

    // define several atomic struct-based error types using the `thiserror` crate
    use thiserror::Error;

    atomic_err!(pub Error1<ParseIntError> { pub data: u32 } => "An error occurred in the atomic module [data={data}]");
    atomic_err!(pub Error2 { pub message: String } => "Another error occurred in the atomic module");
    atomic_err!(pub Error3 { pub data: u32, pub message: String } => "Yet another error occurred in the atomic module");
    atomic_err!(Error4 { pub data: u32, pub message: String } => "Yet another unused error");
}

// define errors enum with 2 first errors from atomic_error module, using the `thiserror` crate

#[derive(Error, Debug)]
pub enum ErrorSet1 {
    #[error(transparent)]
    AtomicError1(#[from] atomic_error::Error1),
    #[error(transparent)]
    AtomicError2(#[from] atomic_error::Error2),
}

// test functions for the atomic_error module
#[cfg(test)]
mod tests {
    use super::*;
    use atomic_error::*;
    use errorset::errorset;

    fn _atomic_error() -> core::result::Result<(), Error2> {
        Err(Error2 {
            message: "Atomic error".to_owned(),
        })
    }

    #[errorset(pub(crate) mod eee)]
    fn error_set() -> core::result::Result<(), (atomic_error::Error1, Error2)> {
        let _t = "-123"
            .parse::<u32>()
            .map_err(|source| atomic_error::Error1 { data: 42, source })?;
        Err(Error2 {
            message: "Atomic error".to_owned(),
        }
        .into())
    }

    #[test]
    fn fire_error1() {
        match error_set() {
            Ok(_) => println!("No error occurred"),
            Err(eee::ErrorSetErrors::Error1(e)) => print!("Error1: {:#?}", e),
            Err(eee::ErrorSetErrors::Error2(e)) => print!("Error2: {:#?}", e),
        }
    }

    struct _SomeStruct {}

    #[errorset(pub(crate) mod eei)]
    impl _SomeStruct {
        #[errorset]
        fn method_one(&self) -> Result<(), (Error1, Error2)> {
            todo!()
        }
        #[errorset]
        fn method_two(&self) -> Result<(), (Error1, Error2)> {
            todo!()
        }
        fn method_three(&self) -> Result<(), (Error1, Error2)> {
            todo!()
        }
        fn method_to_skip(&self) -> u32 {
            todo!()
        }
    }
}
