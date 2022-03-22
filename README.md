# 1. Type Convert.

Take one struct convert another struct, reference from `Iterator` implementation.

```rust
trait Animal {
    fn name(&self) -> String;
}

trait IntoAnimal {
    type IntoAnimal: Animal;

    fn into_animal(self) -> Self::IntoAnimal;
}

impl<A: Animal> IntoAnimal for A {
    type IntoAnimal = A;

    fn into_animal(self) -> Self::IntoAnimal {
        self
    }
}

struct Apeman;

impl Apeman {
    pub fn new() -> Apeman {
        Apeman
    }
}

impl Animal for Apeman {
    fn name(&self) -> String {
        "猿人".into()
    }
}

struct People;

impl IntoAnimal for People {
    type IntoAnimal = Apeman;

    fn into_animal(self) -> Self::IntoAnimal {
        Apeman::new()
    }
}

#[test]
fn test() {
    let people = People {};
    let animal = people.into_animal();
    println!("{}", animal.name());
}
```

# 2. Catch Specified Error.
Reference from `Poem`
```rust
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
```

# 3. Callback
```rust
enum EventType {
    Click,
    DoubleClick,
    Touch,
    None,
}

struct Event {
    typ: EventType,
}

struct Executor<'a> {
    events: Vec<Event>,
    callback: Box<dyn Fn(&Executor) + 'a>,
}

impl<'a> Executor<'a> {
    fn set_callback(&mut self, cb: impl Fn(&Executor) + 'a) {
        self.callback = Box::new(cb);
    }

    fn process_event(&self) {
        (self.callback)(&self)
    }
}

fn handle_click(exec: &Executor) {
    println!("event len = {}", exec.events.len());
    for event in exec.events.iter() {
        match event.typ {
            EventType::Click => println!("clicked"),
            _ => println!("others"),
        }
    }
    println!("Callback")
}
```
# 4. Use defer to simulate java-like abstract method.
```rust
struct Animal;

impl Animal {
    pub fn new() -> Option<&'static Self> {
        Some(&Animal)
    }
    pub fn name(&self) -> &str {
        "动物"
    }
}

struct Monkey;

impl Deref for Monkey {
    type Target = Animal;

    fn deref(&self) -> &Self::Target {
        Animal::new().unwrap()
    }
}
```