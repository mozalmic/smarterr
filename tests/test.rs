use errorset::errorset;
use smarterr::Throwable;

mod atomic_error {
    use smarterr::error;
    use std::num::ParseIntError;

    #[derive(Debug)]
    pub enum Bounds {
        Lower(u32),
        Upper(u32),
    }
    impl Bounds {
        pub fn test(&self, value: u32) -> bool {
            match self {
                Bounds::Lower(bound) => value >= *bound,
                Bounds::Upper(bound) => value <= *bound,
            }
        }
    }

    // define several atomic struct-based error types using the `thiserror` crate
    error!(pub InvalidData<ParseIntError> { pub data: String } => "Invalid data provided [data={data}]");
    error!(pub OutOfBounds { pub value: u32, pub bound: Bounds } => "Value is out of bound [value={value}, bound={bound:?}]");
}

use atomic_error::*;

struct _SomeStruct {}

#[errorset(pub(crate) mod eei)]
impl _SomeStruct {
    #[errorset]
    pub fn parse_u32(&self, data: &str) -> Result<u32, (InvalidData, OutOfBounds)> {
        let min = Bounds::Lower(100);
        let max = Bounds::Upper(1000);
        let data = data.to_owned();
        let value = data.parse::<u32>().throw(|source| InvalidData { data, source })?;
        min.test(value).throw(|_| OutOfBounds { value, bound: min })?;
        max.test(value).throw(|_| OutOfBounds { value, bound: max })?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test() {
        let s = _SomeStruct {};
        let mut actual = vec![];
        for value in &["x1", "10", "100", "1000", "10000"] {
            println!("Value: {}", value);
            let t = s.parse_u32(value);
            let output = match t {
                Ok(v) => format!("-- Value: {}", v),
                Err(e) => format!("-- method_one failed: {:#}", anyhow!(e)),
            };
            println!("{}", output);
            actual.push(output);
        }
        assert_eq!(
            actual,
            vec![
                "-- method_one failed: Invalid data provided [data=x1]: invalid digit found in string",
                "-- method_one failed: Value is out of bound [value=10, bound=Lower(100)]",
                "-- Value: 100",
                "-- Value: 1000",
                "-- method_one failed: Value is out of bound [value=10000, bound=Upper(1000)]",
            ]
        );
    }
}
