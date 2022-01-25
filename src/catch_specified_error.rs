use std::{
    convert::Infallible,
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
    string::FromUtf8Error,
    marker::PhantomData,
};

enum ErrorSource {
    BoxedError(Box<dyn StdError>),
    AnyHow(anyhow::Error),
}

impl Debug for ErrorSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSource::BoxedError(err) => Debug::fmt(err, f),
            ErrorSource::AnyHow(err) => Debug::fmt(err, f),
        }
    }
}

pub struct Error {
    source: ErrorSource,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("source", &self.source)
            .finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.source {
            ErrorSource::BoxedError(err) => Display::fmt(err, f),
            ErrorSource::AnyHow(err) => Display::fmt(err, f),
        }
    }
}

impl Error {
    pub fn downcast_ref<T: StdError + Send + Sync + 'static>(&self) -> Option<&T> {
        match &self.source {
            ErrorSource::BoxedError(err) => err.downcast_ref::<T>(),
            ErrorSource::AnyHow(err) => err.downcast_ref::<T>(),
        }
    }

    pub fn downcast<T: StdError + Send + Sync + 'static>(self) -> Result<T, Error> {
        match self.source {
            ErrorSource::BoxedError(err) => match err.downcast::<T>() {
                Ok(err) => Ok(*err),
                Err(err) => Err(Error {
                    source: ErrorSource::BoxedError(err),
                }),
            },
            ErrorSource::AnyHow(err) => match err.downcast::<T>() {
                Ok(err) => Ok(err),
                Err(err) => Err(Error {
                    source: ErrorSource::AnyHow(err),
                }),
            },
        }
    }

    pub fn is<T: StdError + Debug + Send + Sync + 'static>(&self) -> bool {
        match &self.source {
            ErrorSource::BoxedError(err) => err.is::<T>(),
            ErrorSource::AnyHow(err) => err.is::<T>(),
        }
    }
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

struct CatchError<E, F, ErrType> {
    inner: E,
    f: F,
    mark: PhantomData<ErrType>,
}

impl<E, F, ErrType> CatchError<E, F, ErrType> {
    fn new(inner: E, f: F) -> CatchError<E, F, ErrType> {
        Self {
            inner,
            f,
            mark: PhantomData,
        }
    }
}

trait Animal {
    fn name(&self) -> Result<String>;
}

struct People {
    name: String,
}

impl People {
    fn new() -> People {
        let p = People { name: "".to_string() };
        p
    }

    fn catch_error<F, ErrType>(self, f: F) -> CatchError<Self, F, ErrType>
        where
            F: Fn(ErrType) -> Result<String>,
            ErrType: StdError + 'static,
            Self: Sized,
    {
        CatchError::new(self, f)
    }
}

impl Animal for People {
    fn name(&self) -> Result<String> {
        Err(Error::from(NotFoundError))
    }
}

impl<E, F, ErrType> Animal for CatchError<E, F, ErrType>
    where
        E: Animal,
        F: Fn(ErrType) -> Result<String>,
        ErrType: StdError + 'static + Send + Sync,
{
    fn name(&self) -> Result<String> {
        match self.inner.name() {
            Ok(v) => Ok(v),
            Err(err) if err.is::<ErrType>() => Ok((self.f)(err.downcast::<ErrType>().unwrap()).unwrap()),
            Err(err) => Err(err),
        }
    }
}

impl<T: Animal> Animal for &T {
    fn name(&self) -> Result<String> {
        T::name(self)
    }
}

trait TestError {
    fn msg(&self) -> String;
}

#[derive(Debug, thiserror::Error, Copy, Clone, Eq, PartialEq)]
#[error("Not Found Error.")]
struct NotFoundError;

impl TestError for NotFoundError {
    fn msg(&self) -> String {
        "Not Found Error".into()
    }
}

#[derive(Debug, thiserror::Error, Copy, Clone, Eq, PartialEq)]
#[error("Another eorr.")]
struct AnotherError;

impl TestError for AnotherError {
    fn msg(&self) -> String {
        "Another".into()
    }
}

impl<A: TestError + StdError + Send + Sync + 'static> From<A> for Error {
    fn from(e: A) -> Self {
        Error {
            source: ErrorSource::BoxedError(Box::new(e))
        }
    }
}

#[test]
fn test_catch_specified_error() {
    fn custom_error(_: NotFoundError) -> Result<String> {
        Ok("捕获".into())
    }
    let c = People::new().catch_error(custom_error);
    assert_eq!(c.name().unwrap(), "捕获");
}

#[test]
fn test_uncatch_error() {
    fn uncustom_error(_: AnotherError) -> Result<String> {
        Ok("捕获".into())
    }
    let c = People::new().catch_error(uncustom_error);
    let result = c.name();
    assert_eq!(result.err().unwrap().is::<NotFoundError>(), true);
}