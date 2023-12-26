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

    smarterr_fledged!(pub PlanetsError {
        FledgedFailure<E>
    });

    #[smarterr(
        from NewError { InitFailed },
    )]
    pub fn next(self) {
        let t = Test::new("1", "2")?;
        print!("{:?}", t);
        Ok(())
    }
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

    //smarterr_fledged!(pub PlanetsError {
    //    FledgedFailure<> {pub data: String} -> "Fledged failure",
    //});

    //
    // ====================================================================================================
    // fledged
    #[derive(std::fmt::Debug)]
    pub enum PlanetsError {
        FledgedFailure(FledgedFailure),
    }
    impl std::error::Error for PlanetsError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                PlanetsError::FledgedFailure(err) => err.source(),
                _ => None,
            }
        }
    }
    impl std::fmt::Display for PlanetsError {
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
    pub struct FledgedFailure {
        src: Box<dyn std::error::Error + 'static>,
        ctx: FledgedFailureCtx,
    }
    impl FledgedFailure {
        pub fn new<ES: std::error::Error + 'static>(src: ES, ctx: FledgedFailureCtx) -> Self {
            FledgedFailure { src: Box::new(src), ctx }
        }
        pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&*self.src)
        }
        pub fn default_message(&self) -> &'static str {
            "Fledged failure"
        }
    }
    impl std::fmt::Display for FledgedFailure {
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
    impl<ES: std::error::Error + 'static> smarterr::IntoError<PlanetsError, ES> for FledgedFailureCtx {
        fn into_error(self, source: ES) -> PlanetsError {
            PlanetsError::FledgedFailure(FledgedFailure::new(source, self))
        }
    }
}
