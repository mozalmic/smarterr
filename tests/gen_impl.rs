use smarterr::{Erroneous, RawError, Throwable};
use smarterr_macro::{smarterr, smarterr_fledged, smarterr_mod};
use std::num::ParseIntError;

pub struct Test {
    pub a: i32,
    pub b: i32,
}

#[smarterr_mod(test_err)]
impl Test {
    #[smarterr(InitFailed{pub a: String, pub b: String} -> "Init error")]
    pub fn new(a: &str, b: &str) -> Self {
        Ok(Self {
            a: a.parse()
                .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
            b: b.parse()
                .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
        })
    }
}
