use std::num::ParseIntError;

use smarterr::Erroneous;
use smarterr_macro::smarterr_mod;

#[derive(Debug)]
pub struct Test {
    pub a: i32,
    pub b: i32,
}

/*#[smarterr_mod(test_err)]
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

    pub fn last(self) -> std::result::Result<i32, test_err::PlanetsError<ParseIntError>> {
        let int_str = "22".to_string();
        int_str
            .parse::<i32>()
            .throw_ctx(test_err::FledgedFailureCtx { data: "data".to_string() })
    }

    #[smarterr(
        from NewError { InitFailed },
    )]
    pub fn next(self) {
        let t = Test::new("1", "2")?;
        print!("{:?}", t);
        Ok(())
    }

    smarterr_fledged!(pub PlanetsError<E> {
        FledgedFailure<E> {pub data: String} -> "Fledged failure",
    });
}
*/

impl Test {
    pub fn new(a: &str, b: &str) -> std::result::Result<Self, test_err::NewError> {
        Ok(Self {
            a: a.parse()
                .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
            b: b.parse()
                .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
        })
    }
    pub fn next(self) -> std::result::Result<(), test_err::NextError> {
        trait ErrorHandler<T, EH, ER> {
            fn handle<F: FnOnce(EH) -> Result<T, ER>>(self, handler: F) -> Result<T, ER>;
        }
        impl From<test_err::NewError> for test_err::NextError {
            fn from(source: test_err::NewError) -> Self {
                match source {
                    test_err::NewError::InitFailed(ctx) => test_err::NextError::InitFailed(ctx),
                }
            }
        }
        let t = Test::new("1", "2")?;
        print!("{:?}", t);
        Ok(())
    }
    pub fn last(self) -> std::result::Result<i32, test_err::PlanetsError<ParseIntError>> {
        let int_str = "22".to_string();
        int_str
            .parse::<i32>()
            .throw_ctx(test_err::FledgedFailureCtx { data: "data".to_string() })
    }
}
mod test_err {
    use smarterr_macro::smarterr_fledged;
    #[derive(std::fmt::Debug)]
    pub enum NewError {
        InitFailed(InitFailed),
    }
    impl std::error::Error for NewError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                _ => None,
            }
        }
    }
    impl std::fmt::Display for NewError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NewError::InitFailed(err) => {
                    write!(f, "{}{}", err.default_message(), err)?;
                }
            }
            Ok(())
        }
    }
    #[derive(std::fmt::Debug)]
    pub struct InitFailed {
        ctx: InitFailedCtx,
    }
    impl InitFailed {
        pub fn new<ES>(_src: ES, ctx: InitFailedCtx) -> Self {
            InitFailed { ctx }
        }
        pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            None
        }
        pub fn default_message(&self) -> &'static str {
            "Init error"
        }
    }
    impl std::fmt::Display for InitFailed {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let x = format!("{:?}", self.ctx).replace("\"", "\'");
            let x = x.strip_prefix("InitFailedCtx").unwrap_or("");
            write!(f, "{}", x)?;
            Ok(())
        }
    }
    #[allow(dead_code)]
    #[derive(std::fmt::Debug)]
    pub struct InitFailedCtx {
        pub a: String,
        pub b: String,
    }
    impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<NewError, ES> for InitFailedCtx {
        fn into_error(self, source: ES) -> NewError {
            NewError::InitFailed(InitFailed::new(source, self))
        }
    }

    #[derive(std::fmt::Debug)]
    pub enum NextError {
        InitFailed(InitFailed),
    }
    impl std::error::Error for NextError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                _ => None,
            }
        }
    }
    impl std::fmt::Display for NextError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NextError::InitFailed(err) => {
                    write!(f, "{}{}", err.default_message(), err)?;
                }
            }
            Ok(())
        }
    }
    impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<NextError, ES> for InitFailedCtx {
        fn into_error(self, source: ES) -> NextError {
            NextError::InitFailed(InitFailed::new(source, self))
        }
    }

    //smarterr_fledged!(pub PlanetsError< E1 > {
    //    FledgedFailure<E1> {pub data: String} -> "Fledged failure",
    //    FledgedProblem<E2> {pub data: String} -> "Fledged problem",
    //});

    //
    // ====================================================================================================
    // fledged
    /*#[derive(std::fmt::Debug)]
    pub enum PlanetsError<ES: std::error::Error + 'static> {
        FledgedFailure(FledgedFailure<ES>),
    }
    impl<ES: std::error::Error + 'static> std::error::Error for PlanetsError<ES> {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                PlanetsError::FledgedFailure(err) => err.source().map(|e| e as _),
                _ => None,
            }
        }
    }
    impl<ES: std::error::Error + 'static> std::fmt::Display for PlanetsError<ES> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                PlanetsError::FledgedFailure(err) => {
                    //f.write_fmt(builtin #format_args("{}{}",err.default_message(),err))? ;
                }
            }
            Ok(())
        }
    }
    #[derive(std::fmt::Debug)]
    pub struct FledgedFailure<ES: std::error::Error> {
        src: ES,
        ctx: FledgedFailureCtx,
    }
    impl<ES: std::error::Error> FledgedFailure<ES> {
        pub fn new(src: ES, ctx: FledgedFailureCtx) -> Self {
            FledgedFailure { src, ctx }
        }
        pub fn source(&self) -> Option<&ES> {
            Some(&self.src)
        }
        pub fn default_message(&self) -> &'static str {
            "Fledged failure"
        }
    }
    impl<ES: std::error::Error> std::fmt::Display for FledgedFailure<ES> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            //let x = {
            //  let res = $crate::fmt::format(builtin #format_args("{:?}",self.ctx));
            //  res
            //}.replace("\"","\'");
            //let x = x.strip_prefix("FledgedFailureCtx").unwrap_or("");
            //f.write_fmt(builtin #format_args("{}, caused by: {}",x,self.src))? ;
            Ok(())
        }
    }
    #[allow(dead_code)]
    #[derive(std::fmt::Debug)]
    pub struct FledgedFailureCtx {
        pub data: String,
    }
    impl<ES: std::error::Error> smarterr::IntoError<PlanetsError<ES>, ES> for FledgedFailureCtx {
        fn into_error(self, source: ES) -> PlanetsError<ES> {
            PlanetsError::FledgedFailure(FledgedFailure::new(source, self))
        }
    }*/
}
