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

#[derive(Error, Debug)]
#[error("E3")]
pub struct Error3;

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
        fn method_one<T>(&self, input: T) -> Result<(), (Error1, Error2)>
        where
            T: AsRef<str>,
        {
            Err(Error1 {
                data: 7,
                source: input.as_ref().parse::<u32>().unwrap_err(),
            }
            .into())
        }

        #[errorset]
        fn method_two(&self) -> Result<(), (Error1, Error2)> {
            Err(Error2 { message: "Another error".to_string() }.into())
        }

        fn method_three(&self) -> Result<(), (Error1, Error2)> {
            Ok(())
        }

        fn method_to_skip(&self) -> u32 {
            42
        }
    }

    #[test]
    fn method_errors() {
        let s = _SomeStruct {};
        match s.method_one("fail") {
            Err(eei::MethodOneErrors::Error1(e)) => assert!(e.data == 7),
            _ => panic!("Expected Error1"),
        }
        match s.method_two() {
            Err(eei::MethodTwoErrors::Error2(e)) => assert_eq!(e.message, "Another error"),
            _ => panic!("Expected Error2"),
        }
        assert_eq!(s.method_three().unwrap(), ());
        assert_eq!(s.method_to_skip(), 42);
    }

    // Test where-clause is preserved
    #[errorset(pub(crate) mod where_mod)]
    fn with_where<T: AsRef<str>>(x: T) -> Result<(), (Error1, Error2)>
    where
        T: Clone,
    {
        let _ = x.as_ref().parse::<u32>().map_err(|source| Error1 { data: 0, source })?;
        Ok(())
    }

    #[test]
    fn test_where_clause() {
        let _ = with_where("123".to_string());
    }

    // Test async, const, unsafe (compile-only presence)
    #[allow(unused)]
    #[errorset(pub(crate) mod async_mod)]
    pub async fn async_fn() -> Result<(), (Error1, Error2)> {
        Ok(())
    }

    #[allow(unused)]
    #[errorset(pub(crate) mod const_mod)]
    pub const fn const_fn() -> Result<(), (Error1, Error2)> {
        Ok(())
    }

    #[allow(unused)]
    #[errorset(pub(crate) mod unsafe_mod)]
    pub unsafe fn unsafe_fn() -> Result<(), (Error1, Error2)> {
        Ok(())
    }

    // Test duplicate errors are deduplicated
    #[errorset(pub(crate) mod de_mod)]
    fn dup_errors() -> Result<(), (Error1, Error1)> {
        Err(Error1 {
            data: 1,
            source: "fail".parse::<u32>().unwrap_err(),
        }
        .into())
    }

    #[test]
    fn test_dedup_enum() {
        match dup_errors() {
            Err(de_mod::DupErrorsErrors::Error1(e)) => assert_eq!(e.data, 1),
            _ => panic!("Expected Error1"),
        }
    }
}
