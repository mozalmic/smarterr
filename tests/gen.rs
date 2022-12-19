use smarterr::{Erroneous, RawError, Throwable};
use smarterr_macro::{smarterr, smarterr_fledged};
use std::{num::ParseIntError};

smarterr_fledged!(pub PlanetsError {
    MercuryError{} -> "Mercury error",
    pub MarsError{ind: usize} -> "Mars Error",
    SaturnError<<i32>> -> "Saturn error",
    EarthError<ParseIntError> -> "EarthError",
});

#[smarterr(
    from GreekFuncError {
        AlfaError -> "Imported Alfa error",
        BetaError<> -> "Imported Beta error",
        BetaWrappedError<std::num::ParseIntError> -> "Imported Beta Wrapped Error",
        //GammaError<<>> -> "Imported Gamma error",
        handled GammaError,
        //GammaWrappedError<<i32>> -> "Imported Gamma Wrapped error",
        handled GammaWrappedError,
    },
    from LatinFuncError {
        AlfaError, BetaError<>, BetaWrappedError<ParseIntError>, ZError<<String>>, 
        handled { GammaError, XError, YError }
    },
    FirstError{ind: i32, ext: String} -> "First error",
    SecondError{ind: i32} -> "Second error",
    ThirdError{} -> "Third Error",
)]
pub fn numeric_func(err_ind: usize) -> String {
    let g = greek_func(err_ind).handle(|h| match h {
        GreekFuncErrorHandled::GammaWrappedError(e) => 
            e.ctx.ext.clone().raise_ctx(FirstErrorCtx{ind: err_ind as i32, ext: e.ctx.ext}),
        GreekFuncErrorHandled::GammaError(e) => 
            e.ctx.ext.raise_ctx(SecondErrorCtx{ ind: err_ind as i32 }), 
    })?;

    latin_func(err_ind).handle(|h| match h {
        LatinFuncErrorHandled::XError(e)=>
            ().raise_ctx(FirstErrorCtx{ ind: err_ind as i32, ext: e.ctx.ext }),
        LatinFuncErrorHandled::YError(e)=>
            ().raise_ctx(SecondErrorCtx{ ind: e.ctx.ind }),
        LatinFuncErrorHandled::GammaError(_) => Ok(())
    })?;

    let t = ().raise_ctx(MarsErrorCtx{ind: err_ind});
    let _t:Result<(), GreekFuncError> = t.throw_ctx(BetaErrorCtx{ ind: err_ind as i32 });

    let _t = RawError::new(10);

    Ok(g)
}

#[smarterr(
    from GreekFuncError { AlfaError, BetaError<>, BetaWrappedError<ParseIntError>, GammaError<<>>, handled GammaWrappedError},
    XError{ind: i32, ext: String} -> "X error",
    YError{ind: i32} -> "Y error",
    pub ZError<<String>>{ind: usize} -> "Z Error",
)]
fn latin_func(err_ind: usize) {
    greek_func(err_ind).handle(|h| match h {
        GreekFuncErrorHandled::GammaWrappedError(data) => data.ctx.ext.throw_ctx(ZErrorCtx { ind: err_ind }),
    })?;
    Ok(())
}

#[smarterr(
    AlfaError{ind: i32, ext: String} -> "Alfa error",
    BetaError<>{ind: i32} -> "Beta error",
    BetaWrappedError<ParseIntError> -> "Beta Wrapped Error",
    GammaError<<>>{ext: String} -> "Gamma error",
    GammaWrappedError<<i32>>{ext: String} -> "Gamma Wrapped error",
)]
pub fn greek_func(err_ind: usize) -> String {
    let ok_str = "All is ok".to_string();
    let err_str = "Error raised".to_string();
    let ext = "ext".to_string();
    match err_ind {
        0 => Ok(ok_str),
        1 => err_str.raise_ctx(AlfaErrorCtx { ind: -1, ext }),
        2 => "z12".parse::<i32>().throw_ctx(BetaErrorCtx { ind: -2 }).map(|_| ok_str),
        3 => "z12".parse::<i32>().throw_ctx(BetaWrappedErrorCtx {}).map(|_| ok_str),
        4 => err_str.raise_ctx(GammaErrorCtx { ext }),
        5 => 5000000.throw_ctx(GammaWrappedErrorCtx { ext }).map(|_| ok_str),
        _ => Ok(ok_str),
    }
}

fn raw_throwable(val: i32) -> Result<i32, RawError<i32>> {
    val.throw()
    //val.throw_with("raw error")
}

#[test]
pub fn test_throwable()  {
    assert_eq!(raw_throwable(0).unwrap(), 0);
    assert_eq!(raw_throwable(10).is_err(), true);
    assert_eq!(format!("{}", raw_throwable(10).unwrap_err()), 
        "raw error { value: 10 }"
    );
}

smarterr_fledged!(DomainErrors{
    DomainError<<i32>> -> "Domain error"
});

fn raw_erroneous(val: i32) -> Result<i32, RawError<i32>> {
    val.throw_err(RawError::new_with(val, "raw error"))
}

fn raw_erroneous_then(val: i32) -> Result<i32, RawError<i32>> {
    val.throw_then(|v| RawError::new_with(v, "raw error"))
}

fn raw_erroneous_ctx(val: i32) -> Result<i32, DomainErrors> {
    val.throw_ctx(DomainErrorCtx{})
}

#[test]
pub fn test_erroneous()  {
    assert_eq!(raw_erroneous(0).unwrap(), 0);
    assert_eq!(raw_erroneous_then(10).is_err(), true);
    assert_eq!(format!("{}", raw_erroneous_then(10).unwrap_err()), 
        "raw error { value: 10 }"
    );
    assert_eq!(format!("{}", raw_erroneous_ctx(10).unwrap_err()), 
        "Domain error, caused by: raw error { value: 10 }"
    );
}

#[test]
pub fn test() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        format!("{}", greek_func(1).unwrap_err()),
        "Alfa error { ind: -1, ext: 'ext' }"
    );
    assert_eq!(
        format!("{}", greek_func(2).unwrap_err()),
        "Beta error { ind: -2 }, caused by: invalid digit found in string"
    );
    assert_eq!(
        format!("{}", greek_func(3).unwrap_err()),
        "Beta Wrapped Error, caused by: invalid digit found in string"
    );
    assert_eq!(
        format!("{}", greek_func(4).unwrap_err()),
        "Gamma error { ext: 'ext' }, caused by: raw error 'Error raised'"
    );
    assert_eq!(
        format!("{}", greek_func(5).unwrap_err()),
        "Gamma Wrapped error { ext: 'ext' }, caused by: raw error 5000000"
    );

    numeric_func(0)?;
    Ok(latin_func(0)?)
}
