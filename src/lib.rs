//! Tiny input macros.
//!
//! This crate provides three macros for receiving user input:
//! [`tiny_input!`], [`input!`] and [`raw_input!`].
//!
//! [`raw_input!`] is used for when you just need the string (while handling I/O errors):
//!
//! ```no_run
//! use tiny_input::raw_input;
//!
//! let name = raw_input!("What is your name? ").unwrap();
//!
//! println!("Hello, {name}!");
//! ```
//!
//! [`tiny_input!`] is useful for when panicking on I/O errors is fine,
//! and you only need to parse the input:
//!
//! ```no_run
//! use tiny_input::tiny_input;
//!
//! let value: u64 = tiny_input!("the square of ").unwrap();
//!
//! println!("is {}", value * value);
//! ```
//!
//! [`input!`] is when you need to handle both I/O and parsing errors:
//!
//! ```no_run
//! use tiny_input::{input, Error};
//!
//! match input!(as u64, "the square of ") {
//!     Ok(value) => println!("is {}", value * value),
//!     Err(error) => match error {
//!         Error::Fetch(fetch_error) => eprintln!("failed to fetch: {fetch_error}"),
//!         Error::Parse(parse_error) => eprintln!("failed to parse: {parse_error}"),
//!     },
//! }
//! ```
//!
//! As one might have noticed, there are two kinds of [`tiny_input!`] and [`input!`],
//! one that attempts to infer the type, and one where you can provide the type explicitly.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

use thiserror::Error;

/// Represents errors that can occur when processing inputs.
#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error<E> {
    /// Fetch error. Returned when any I/O errors occur,
    /// such as when writing to [`stdout`] and flushing it,
    /// as well as when reading from [`stdin`].
    ///
    /// [`stdin`]: std::io::stdin
    /// [`stdout`]: std::io::stdout
    Fetch(std::io::Error),
    /// Parse error, which is contrained to implement the [`Error`] trait.
    /// Returned when parsing into `T` fails; the [`T::Err`] is wrapped into this variant.
    ///
    /// [`T::Err`]: std::str::FromStr::Err
    /// [`Error`]: std::error::Error
    Parse(E),
}

/// The specialized result type to be used in this library.
pub type Result<T, E> = std::result::Result<T, Error<E>>;

/// The message used for expecting values.
pub const FETCH_ERROR: &str = "I/O error occured while fetching input";

/// Invokes [`raw_input!`], panicking on I/O errors before parsing the string.
#[macro_export]
macro_rules! tiny_input {
    (as $type: ty $(, $($token: tt)+)?) => {
        $crate::raw_input!($($($token)+)?).expect($crate::FETCH_ERROR).parse::<$type>()
    };
    ($($token: tt)*) => {
        $crate::raw_input!($($token)*).expect($crate::FETCH_ERROR).parse()
    };
}

/// Similar to [`tiny_input!`], except I/O and parse errors are wrapped into [`enum@Error<E>`].
#[macro_export]
macro_rules! input {
    (as $type: ty $(, $($token: tt)+)?) => {
        $crate::raw_input!($($($token)+)?)
            .map_err($crate::Error::Fetch)
            .and_then(|string| string.parse::<$type>().map_err($crate::Error::Parse))
    };
    ($($token: tt)*) => {
        $crate::raw_input!($($token)*)
            .map_err($crate::Error::Fetch)
            .and_then(|string| string.parse().map_err($crate::Error::Parse))
    };
}

/// Fetches raw inputs, returning the resulting [`String`] and propagating I/O errors.
#[macro_export]
macro_rules! raw_input {
    ($($token: tt)+) => {{
        use ::std::io::Write;

        let mut stdout = ::std::io::stdout().lock();

        // avoid using `?` operator here

        match write!(stdout, $($token)+) {
            // we do not really need to know the byte count
            Ok(_) => match stdout.flush() {
                Ok(_) => $crate::raw_input!(),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        }
    }};
    () => {{
        use ::std::io::BufRead;

        let mut string = ::std::string::String::new();

        match ::std::io::stdin().lock().read_line(&mut string) {
            // we do not need the byte count here
            Ok(_) => {
                string.pop();  // remove the newline character, if there is one

                Ok(string)
            },
            Err(error) => Err(error),
        }
    }};
}
