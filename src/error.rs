#![allow(dead_code)]

use std::error::Error as StdError;

/// Alias for `async` and `anyhow` friendly dynamic error
/// `Box<dyn std::error::Error + Send + Sync + 'static>`.
#[allow(unused)]
pub type BoxError = Box<dyn StdError + Send + Sync + 'static>;

/// Extension methods for types implementing `std::error::Error`.
pub trait StdErrorExt
where
    Self: StdError,
{
    /// Format this error as a chain of colon separated strings built from this error and all
    /// recursive sources.
    ///
    /// Can be used to log errors like this:
    ///
    /// `error!(error = error.as_chain(), "cannot do this or that");`
    fn as_chain(&self) -> String {
        let mut sources = vec![];
        sources.push(self.to_string());

        let mut source = self.source();
        while let Some(s) = source {
            sources.push(s.to_string());
            source = s.source();
        }

        sources.join(": ")
    }
}

impl<T> StdErrorExt for T where T: StdError {}

#[cfg(test)]
mod tests {
    use crate::error::StdErrorExt;
    use std::num::ParseIntError;
    use thiserror::Error;

    #[test]
    fn test_as_chain() {
        let number = "-1".parse::<u32>().map_err(Error);
        assert_eq!(
            number.unwrap_err().as_chain(),
            "error: invalid digit found in string"
        );
    }

    #[derive(Debug, Error)]
    #[error("error")]
    struct Error(#[source] ParseIntError);
}
