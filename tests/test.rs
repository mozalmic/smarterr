#[cfg(feature = "atomic_error")]
mod atomic_error {
    use smarterr::error;
    use std::num::ParseIntError;

    // define several atomic struct-based error types using the `thiserror` crate
    error!(pub Error1<ParseIntError> { pub data: String } => "Error1 [data={data}]");
    error!(pub Error2 { pub message: String } => "Error2 [msg={message}]");
    error!(pub Error3 { pub data: u32, pub message: String } => "Error3 [data={data}, msg={message}]");
    error!(Error4 { pub data: u32, pub message: String } => "Error4 [data={data}, msg={message}]");
}

#[cfg(test)]
#[cfg(feature = "atomic_error")]
mod tests {
    use smarterr::Throwable;

    fn error_producer(v: &str, inverse: bool) -> Result<usize, atomic_error::Error1> {
        let v = v
            .parse::<usize>()
            .throw(|e| atomic_error::Error1 { data: v.to_owned(), source: v })?;
        if inverse {
            v.raise(|v| atomic_error::Error2 { message: format!("Int value: {}", v) })
        } else {
            v.throw(|v| atomic_error::Error2 { message: format!("Int value: {}", v) })
        }
    }

    #[test]
    fn test_error_producer() {
        let v = "123";
        match error_producer(v, false) {
            Ok(_) => println!("No error occurred"),
            Err(e) => println!("Error: {:#?}", e),
        }
        match error_producer(v, true) {
            Ok(_) => println!("No error occurred"),
            Err(e) => println!("Error: {:#?}", e),
        }
    }
}
