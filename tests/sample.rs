extern crate std;
use smarterr::Erroneous;
use std::num::ParseIntError;
pub fn numeric_func(err_ind: usize) -> std::result::Result<String, NumericFuncError> {
    trait ErrorHandler<T, EH, ER> {
        fn raise_filter<F: FnOnce(EH) -> Result<T, ER>>(self, handler: F) -> Result<T, ER>;
    }
    enum GreekFuncErrorHandled {
        GammaError(GammaError),
        GammaWrappedError(GammaWrappedError),
    }
    impl<T> ErrorHandler<T, GreekFuncErrorHandled, NumericFuncError> for Result<T, GreekFuncError> {
        fn raise_filter<F: FnOnce(GreekFuncErrorHandled) -> Result<T, NumericFuncError>>(
            self,
            handler: F,
        ) -> Result<T, NumericFuncError> {
            match self {
                Ok(v) => Ok(v),
                Err(e) => match e {
                    GreekFuncError::AlfaError(e) => Err(NumericFuncError::AlfaError(e)),
                    GreekFuncError::BetaError(e) => Err(NumericFuncError::BetaError(e)),
                    GreekFuncError::BetaWrappedError(e) => Err(NumericFuncError::BetaWrappedError(e)),
                    GreekFuncError::GammaError(e) => handler(GreekFuncErrorHandled::GammaError(e)),
                    GreekFuncError::GammaWrappedError(e) => handler(GreekFuncErrorHandled::GammaWrappedError(e)),
                },
            }
        }
    }
    enum LatinFuncErrorHandled {
        GammaError(GammaError),
        XError(XError),
        YError(YError),
    }
    impl<T> ErrorHandler<T, LatinFuncErrorHandled, NumericFuncError> for Result<T, LatinFuncError> {
        fn raise_filter<F: FnOnce(LatinFuncErrorHandled) -> Result<T, NumericFuncError>>(
            self,
            handler: F,
        ) -> Result<T, NumericFuncError> {
            match self {
                Ok(v) => Ok(v),
                Err(e) => match e {
                    LatinFuncError::AlfaError(e) => Err(NumericFuncError::AlfaError(e)),
                    LatinFuncError::BetaError(e) => Err(NumericFuncError::BetaError(e)),
                    LatinFuncError::BetaWrappedError(e) => Err(NumericFuncError::BetaWrappedError(e)),
                    LatinFuncError::ZError(e) => Err(NumericFuncError::ZError(e)),
                    LatinFuncError::GammaError(e) => handler(LatinFuncErrorHandled::GammaError(e)),
                    LatinFuncError::XError(e) => handler(LatinFuncErrorHandled::XError(e)),
                    LatinFuncError::YError(e) => handler(LatinFuncErrorHandled::YError(e)),
                },
            }
        }
    }
    let t = greek_func(err_ind).raise_filter(|h| match h {
        GreekFuncErrorHandled::GammaWrappedError(e) => e
            .ctx
            .ext
            .clone()
            .raise_ctx(FirstErrorCtx { ind: err_ind as i32, ext: e.ctx.ext }),
        GreekFuncErrorHandled::GammaError(e) => e.ctx.ext.raise_ctx(SecondErrorCtx { ind: err_ind as i32 }),
    })?;
    let _ = latin_func(err_ind);
    Ok(t)
}
pub enum NumericFuncError {
    AlfaError(AlfaError),
    BetaError(BetaError),
    BetaWrappedError(BetaWrappedError),
    ZError(ZError),
    FirstError(FirstError),
    SecondError(SecondError),
    ThirdError(ThirdError),
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for NumericFuncError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&NumericFuncError::AlfaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "AlfaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&NumericFuncError::BetaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "BetaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&NumericFuncError::BetaWrappedError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "BetaWrappedError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&NumericFuncError::ZError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "ZError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&NumericFuncError::FirstError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "FirstError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&NumericFuncError::SecondError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "SecondError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&NumericFuncError::ThirdError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "ThirdError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
        }
    }
}
impl std::error::Error for NumericFuncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NumericFuncError::BetaError(err) => err.source(),
            NumericFuncError::BetaWrappedError(err) => err.source(),
            NumericFuncError::ZError(err) => err.source(),
            _ => None,
        }
    }
}
impl std::fmt::Display for NumericFuncError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericFuncError::AlfaError(_err) => {}
            NumericFuncError::BetaError(_err) => {}
            NumericFuncError::BetaWrappedError(_err) => {}
            NumericFuncError::ZError(_err) => {}
            NumericFuncError::FirstError(_err) => {}
            NumericFuncError::SecondError(_err) => {}
            NumericFuncError::ThirdError(_err) => {}
        }
        Ok(())
    }
}
#[allow(dead_code)]
pub struct FirstError {
    ctx: FirstErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for FirstError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            FirstError { ctx: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "FirstError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl FirstError {
    pub fn new<ES>(_src: ES, ctx: FirstErrorCtx) -> Self {
        FirstError { ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    pub fn default_message(&self) -> &'static str {
        "First error"
    }
}
impl std::fmt::Display for FirstError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct FirstErrorCtx {
    ind: i32,
    ext: String,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for FirstErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            FirstErrorCtx { ind: ref __self_0_0, ext: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "FirstErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ind", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ext", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
pub struct SecondError {
    ctx: SecondErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for SecondError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            SecondError { ctx: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "SecondError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl SecondError {
    pub fn new<ES>(_src: ES, ctx: SecondErrorCtx) -> Self {
        SecondError { ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    pub fn default_message(&self) -> &'static str {
        "Second error"
    }
}
impl std::fmt::Display for SecondError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct SecondErrorCtx {
    ind: i32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for SecondErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            SecondErrorCtx { ind: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "SecondErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ind", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
pub struct ThirdError {
    ctx: ThirdErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for ThirdError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            ThirdError { ctx: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "ThirdError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl ThirdError {
    pub fn new<ES>(_src: ES, ctx: ThirdErrorCtx) -> Self {
        ThirdError { ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    pub fn default_message(&self) -> &'static str {
        "Third Error"
    }
}
impl std::fmt::Display for ThirdError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct ThirdErrorCtx {}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for ThirdErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            ThirdErrorCtx {} => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "ThirdErrorCtx");
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<NumericFuncError, ES> for AlfaErrorCtx {
    fn into_error(self, source: ES) -> NumericFuncError {
        NumericFuncError::AlfaError(AlfaError::new(source, self))
    }
}
impl<ES: std::error::Error + 'static> smarterr::IntoError<NumericFuncError, ES> for BetaErrorCtx {
    fn into_error(self, source: ES) -> NumericFuncError {
        NumericFuncError::BetaError(BetaError::new(source, self))
    }
}
impl smarterr::IntoError<NumericFuncError, std::num::ParseIntError> for BetaWrappedErrorCtx {
    fn into_error(self, source: std::num::ParseIntError) -> NumericFuncError {
        NumericFuncError::BetaWrappedError(BetaWrappedError::new(source, self))
    }
}
impl smarterr::IntoError<NumericFuncError, String> for ZErrorCtx {
    fn into_error(self, source: String) -> NumericFuncError {
        NumericFuncError::ZError(ZError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<NumericFuncError, ES> for FirstErrorCtx {
    fn into_error(self, source: ES) -> NumericFuncError {
        NumericFuncError::FirstError(FirstError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<NumericFuncError, ES> for SecondErrorCtx {
    fn into_error(self, source: ES) -> NumericFuncError {
        NumericFuncError::SecondError(SecondError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<NumericFuncError, ES> for ThirdErrorCtx {
    fn into_error(self, source: ES) -> NumericFuncError {
        NumericFuncError::ThirdError(ThirdError::new(source, self))
    }
}
fn latin_func(err_ind: usize) -> std::result::Result<String, LatinFuncError> {
    trait ErrorHandler<T, EH, ER> {
        fn raise_filter<F: FnOnce(EH) -> Result<T, ER>>(self, handler: F) -> Result<T, ER>;
    }
    enum GreekFuncErrorHandled {
        GammaWrappedError(GammaWrappedError),
    }
    impl<T> ErrorHandler<T, GreekFuncErrorHandled, LatinFuncError> for Result<T, GreekFuncError> {
        fn raise_filter<F: FnOnce(GreekFuncErrorHandled) -> Result<T, LatinFuncError>>(
            self,
            handler: F,
        ) -> Result<T, LatinFuncError> {
            match self {
                Ok(v) => Ok(v),
                Err(e) => match e {
                    GreekFuncError::AlfaError(e) => Err(LatinFuncError::AlfaError(e)),
                    GreekFuncError::BetaError(e) => Err(LatinFuncError::BetaError(e)),
                    GreekFuncError::BetaWrappedError(e) => Err(LatinFuncError::BetaWrappedError(e)),
                    GreekFuncError::GammaError(e) => Err(LatinFuncError::GammaError(e)),
                    GreekFuncError::GammaWrappedError(e) => handler(GreekFuncErrorHandled::GammaWrappedError(e)),
                },
            }
        }
    }
    greek_func(err_ind).raise_filter(|h| match h {
        GreekFuncErrorHandled::GammaWrappedError(data) => data.ctx.ext.throw_ctx(ZErrorCtx { ind: err_ind }),
    })
}
enum LatinFuncError {
    AlfaError(AlfaError),
    BetaError(BetaError),
    BetaWrappedError(BetaWrappedError),
    GammaError(GammaError),
    XError(XError),
    YError(YError),
    ZError(ZError),
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for LatinFuncError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&LatinFuncError::AlfaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "AlfaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&LatinFuncError::BetaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "BetaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&LatinFuncError::BetaWrappedError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "BetaWrappedError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&LatinFuncError::GammaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "GammaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&LatinFuncError::XError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "XError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&LatinFuncError::YError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "YError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&LatinFuncError::ZError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "ZError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
        }
    }
}
impl std::error::Error for LatinFuncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LatinFuncError::BetaError(err) => err.source(),
            LatinFuncError::BetaWrappedError(err) => err.source(),
            LatinFuncError::GammaError(err) => err.source(),
            LatinFuncError::ZError(err) => err.source(),
            _ => None,
        }
    }
}
impl std::fmt::Display for LatinFuncError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LatinFuncError::AlfaError(_err) => {}
            LatinFuncError::BetaError(_err) => {}
            LatinFuncError::BetaWrappedError(_err) => {}
            LatinFuncError::GammaError(_err) => {}
            LatinFuncError::XError(_err) => {}
            LatinFuncError::YError(_err) => {}
            LatinFuncError::ZError(_err) => {}
        }
        Ok(())
    }
}
#[allow(dead_code)]
struct XError {
    ctx: XErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for XError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            XError { ctx: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "XError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
impl XError {
    pub fn new<ES>(_src: ES, ctx: XErrorCtx) -> Self {
        XError { ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    pub fn default_message(&self) -> &'static str {
        "X error"
    }
}
impl std::fmt::Display for XError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
struct XErrorCtx {
    ind: i32,
    ext: String,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for XErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            XErrorCtx { ind: ref __self_0_0, ext: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "XErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ind", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ext", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
struct YError {
    ctx: YErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for YError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            YError { ctx: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "YError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
impl YError {
    pub fn new<ES>(_src: ES, ctx: YErrorCtx) -> Self {
        YError { ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    pub fn default_message(&self) -> &'static str {
        "Y error"
    }
}
impl std::fmt::Display for YError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
struct YErrorCtx {
    ind: i32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for YErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            YErrorCtx { ind: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "YErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ind", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
pub struct ZError {
    src: smarterr::RawError<String>,
    ctx: ZErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for ZError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            ZError { src: ref __self_0_0, ctx: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "ZError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "src", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl ZError {
    pub fn new(src: String, ctx: ZErrorCtx) -> Self {
        ZError { src: smarterr::RawError::new(src), ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.src as _)
    }
    pub fn default_message(&self) -> &'static str {
        "Z Error"
    }
}
impl std::fmt::Display for ZError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct ZErrorCtx {
    ind: usize,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for ZErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            ZErrorCtx { ind: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "ZErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ind", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<LatinFuncError, ES> for AlfaErrorCtx {
    fn into_error(self, source: ES) -> LatinFuncError {
        LatinFuncError::AlfaError(AlfaError::new(source, self))
    }
}
impl<ES: std::error::Error + 'static> smarterr::IntoError<LatinFuncError, ES> for BetaErrorCtx {
    fn into_error(self, source: ES) -> LatinFuncError {
        LatinFuncError::BetaError(BetaError::new(source, self))
    }
}
impl smarterr::IntoError<LatinFuncError, ParseIntError> for BetaWrappedErrorCtx {
    fn into_error(self, source: ParseIntError) -> LatinFuncError {
        LatinFuncError::BetaWrappedError(BetaWrappedError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<LatinFuncError, ES> for GammaErrorCtx {
    fn into_error(self, source: ES) -> LatinFuncError {
        LatinFuncError::GammaError(GammaError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<LatinFuncError, ES> for XErrorCtx {
    fn into_error(self, source: ES) -> LatinFuncError {
        LatinFuncError::XError(XError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<LatinFuncError, ES> for YErrorCtx {
    fn into_error(self, source: ES) -> LatinFuncError {
        LatinFuncError::YError(YError::new(source, self))
    }
}
impl smarterr::IntoError<LatinFuncError, String> for ZErrorCtx {
    fn into_error(self, source: String) -> LatinFuncError {
        LatinFuncError::ZError(ZError::new(source, self))
    }
}
pub fn greek_func(err_ind: usize) -> std::result::Result<String, GreekFuncError> {
    trait ErrorHandler<T, EH, ER> {
        fn raise_filter<F: FnOnce(EH) -> Result<T, ER>>(self, handler: F) -> Result<T, ER>;
    }
    let ok_str = "All is ok".to_string();
    let err_str = "Error raised".to_string();
    let ext = "ext".to_string();
    match err_ind {
        0 => Ok(ok_str),
        1 => err_str.raise_ctx(AlfaErrorCtx { ind: -1, ext }),
        2 => "z12".parse::<i32>().throw_ctx(BetaErrorCtx { ind: -2 }).map(|_| ok_str),
        3 => "z12".parse::<i32>().throw_ctx(BetaWrappedErrorCtx {}).map(|_| ok_str),
        4 => err_str.throw_ctx(GammaErrorCtx { ext }),
        5 => 5000000.throw_ctx(GammaWrappedErrorCtx { ext }).map(|_| ok_str),
        _ => Ok(ok_str),
    }
}
pub enum GreekFuncError {
    AlfaError(AlfaError),
    BetaError(BetaError),
    BetaWrappedError(BetaWrappedError),
    GammaError(GammaError),
    GammaWrappedError(GammaWrappedError),
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for GreekFuncError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&GreekFuncError::AlfaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "AlfaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&GreekFuncError::BetaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "BetaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&GreekFuncError::BetaWrappedError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "BetaWrappedError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&GreekFuncError::GammaError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "GammaError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&GreekFuncError::GammaWrappedError(ref __self_0),) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "GammaWrappedError");
                let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
        }
    }
}
impl std::error::Error for GreekFuncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GreekFuncError::BetaError(err) => err.source(),
            GreekFuncError::BetaWrappedError(err) => err.source(),
            GreekFuncError::GammaError(err) => err.source(),
            GreekFuncError::GammaWrappedError(err) => err.source(),
            _ => None,
        }
    }
}
impl std::fmt::Display for GreekFuncError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GreekFuncError::AlfaError(_err) => {}
            GreekFuncError::BetaError(_err) => {}
            GreekFuncError::BetaWrappedError(_err) => {}
            GreekFuncError::GammaError(_err) => {}
            GreekFuncError::GammaWrappedError(_err) => {}
        }
        Ok(())
    }
}
#[allow(dead_code)]
pub struct AlfaError {
    ctx: AlfaErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for AlfaError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            AlfaError { ctx: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "AlfaError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl AlfaError {
    pub fn new<ES>(_src: ES, ctx: AlfaErrorCtx) -> Self {
        AlfaError { ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    pub fn default_message(&self) -> &'static str {
        "Alfa error"
    }
}
impl std::fmt::Display for AlfaError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct AlfaErrorCtx {
    ind: i32,
    ext: String,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for AlfaErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            AlfaErrorCtx { ind: ref __self_0_0, ext: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "AlfaErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ind", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ext", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
pub struct BetaError {
    src: Box<dyn std::error::Error>,
    ctx: BetaErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for BetaError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            BetaError { src: ref __self_0_0, ctx: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "BetaError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "src", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl BetaError {
    pub fn new<ES: std::error::Error + 'static>(src: ES, ctx: BetaErrorCtx) -> Self {
        BetaError { src: Box::new(src), ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.src)
    }
    pub fn default_message(&self) -> &'static str {
        "Beta error"
    }
}
impl std::fmt::Display for BetaError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct BetaErrorCtx {
    ind: i32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for BetaErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            BetaErrorCtx { ind: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "BetaErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ind", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[allow(dead_code)]
pub struct BetaWrappedError {
    src: ParseIntError,
    ctx: BetaWrappedErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for BetaWrappedError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            BetaWrappedError { src: ref __self_0_0, ctx: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "BetaWrappedError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "src", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl BetaWrappedError {
    pub fn new(src: ParseIntError, ctx: BetaWrappedErrorCtx) -> Self {
        BetaWrappedError { src: src, ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.src as _)
    }
    pub fn default_message(&self) -> &'static str {
        "Beta Wrapped Error"
    }
}
impl std::fmt::Display for BetaWrappedError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct BetaWrappedErrorCtx {}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for BetaWrappedErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            BetaWrappedErrorCtx {} => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "BetaWrappedErrorCtx");
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
pub struct GammaError {
    src: smarterr::RawError<Box<dyn std::fmt::Debug + 'static>>,
    ctx: GammaErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for GammaError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            GammaError { src: ref __self_0_0, ctx: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "GammaError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "src", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl GammaError {
    pub fn new<ES: std::fmt::Debug + 'static>(src: ES, ctx: GammaErrorCtx) -> Self {
        GammaError { src: smarterr::RawError::new(Box::new(src)), ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.src as _)
    }
    pub fn default_message(&self) -> &'static str {
        "Gamma error"
    }
}
impl std::fmt::Display for GammaError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct GammaErrorCtx {
    ext: String,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for GammaErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            GammaErrorCtx { ext: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "GammaErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ext", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
pub struct GammaWrappedError {
    src: smarterr::RawError<i32>,
    ctx: GammaWrappedErrorCtx,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for GammaWrappedError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            GammaWrappedError { src: ref __self_0_0, ctx: ref __self_0_1 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "GammaWrappedError");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "src", &&(*__self_0_0));
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ctx", &&(*__self_0_1));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl GammaWrappedError {
    pub fn new(src: i32, ctx: GammaWrappedErrorCtx) -> Self {
        GammaWrappedError { src: smarterr::RawError::new(src), ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.src as _)
    }
    pub fn default_message(&self) -> &'static str {
        "Gamma Wrapped error"
    }
}
impl std::fmt::Display for GammaWrappedError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
#[allow(dead_code)]
pub struct GammaWrappedErrorCtx {
    ext: String,
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(dead_code)]
impl ::core::fmt::Debug for GammaWrappedErrorCtx {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            GammaWrappedErrorCtx { ext: ref __self_0_0 } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "GammaWrappedErrorCtx");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "ext", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<GreekFuncError, ES> for AlfaErrorCtx {
    fn into_error(self, source: ES) -> GreekFuncError {
        GreekFuncError::AlfaError(AlfaError::new(source, self))
    }
}
impl<ES: std::error::Error + 'static> smarterr::IntoError<GreekFuncError, ES> for BetaErrorCtx {
    fn into_error(self, source: ES) -> GreekFuncError {
        GreekFuncError::BetaError(BetaError::new(source, self))
    }
}
impl smarterr::IntoError<GreekFuncError, ParseIntError> for BetaWrappedErrorCtx {
    fn into_error(self, source: ParseIntError) -> GreekFuncError {
        GreekFuncError::BetaWrappedError(BetaWrappedError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<GreekFuncError, ES> for GammaErrorCtx {
    fn into_error(self, source: ES) -> GreekFuncError {
        GreekFuncError::GammaError(GammaError::new(source, self))
    }
}
impl smarterr::IntoError<GreekFuncError, i32> for GammaWrappedErrorCtx {
    fn into_error(self, source: i32) -> GreekFuncError {
        GreekFuncError::GammaWrappedError(GammaWrappedError::new(source, self))
    }
}
