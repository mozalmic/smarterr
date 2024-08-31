use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("The first error [data={data}]")]
pub struct Error1 {
    pub data: u32,
    #[source]
    pub source: ParseIntError,
}

#[derive(Error, Debug)]
#[error("The second error [msg={message}]")]
pub struct Error2 {
    pub message: String,
}

// test functions for the atomic_error module
#[cfg(test)]
mod tests {
    use super::*;
    use errorset::errorset;

    #[errorset(pub(crate) mod errors)]
    fn error_set() -> core::result::Result<(), (Error1, Error2)> {
        let _t = "-123".parse::<u32>().map_err(|source| Error1 { data: 42, source })?;
        Err(Error2 { message: "Error message".to_owned() }.into())
    }

    #[test]
    fn fire_error1() {
        match error_set() {
            Ok(_) => println!("No error occurred"),
            Err(errors::ErrorSetErrors::Error1(e)) => print!("Error1: {:#?}", e),
            Err(errors::ErrorSetErrors::Error2(e)) => print!("Error2: {:#?}", e),
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
