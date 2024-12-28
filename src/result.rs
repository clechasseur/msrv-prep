use std::io;

use cargo_metadata::camino::Utf8PathBuf;
use toml_edit::TomlError;

/// Result type used for our crate. Uses our [`Error`] type by default.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error type used by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("I/O error while {context}: {source:?}")]
    Io { source: io::Error, context: String },

    #[error(transparent)]
    Toml(#[from] TomlError),

    #[error("invalid path: {0}")]
    InvalidPath(Utf8PathBuf),

    #[error("backup file already exists: {0}")]
    BackupFileAlreadyExists(Utf8PathBuf),
}

/// Trait used to provide context for I/O errors.
///
/// # Example
///
/// ```no_run
/// use std::fs;
///
/// use cargo_msrv_prep::result::IoErrorContext;
///
/// # fn read_file_content() -> cargo_msrv_prep::Result<()> {
/// let file_content =
///     fs::read_to_string("some_file").with_io_context(|| "reading data from 'some_file'")?;
/// # Ok(())
/// # }
/// ```
pub trait IoErrorContext {
    /// Type returned from [`with_io_context`](Self::with_io_context).
    type Output;

    /// Provides context for the I/O error.
    ///
    /// See [trait description](Self) for details.
    fn with_io_context<C, F>(self, context: F) -> Self::Output
    where
        C: Into<String>,
        F: FnOnce() -> C;
}

impl IoErrorContext for io::Error {
    type Output = Error;

    fn with_io_context<C, F>(self, context: F) -> Self::Output
    where
        C: Into<String>,
        F: FnOnce() -> C,
    {
        Error::Io { source: self, context: context().into() }
    }
}

impl<T, E> IoErrorContext for core::result::Result<T, E>
where
    E: IoErrorContext<Output = Error>,
{
    type Output = Result<T>;

    fn with_io_context<C, F>(self, context: F) -> Self::Output
    where
        C: Into<String>,
        F: FnOnce() -> C,
    {
        self.map_err(|err| err.with_io_context(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod impl_io_error_context_for_io_error {
        use assert_matches::assert_matches;

        use super::*;

        #[test]
        fn with_io_context() {
            let error = io::Error::other("oh no").with_io_context(|| "doing something");

            assert_matches!(error, Error::Io { source, context } => {
                assert_eq!(io::ErrorKind::Other, source.kind());
                assert_eq!("doing something", context);
            });
        }
    }

    mod impl_io_error_context_for_core_result {
        use super::*;

        mod with_io_context {
            use assert_matches::assert_matches;

            use super::*;

            #[test]
            fn for_ok() {
                let result = Ok::<_, io::Error>(()).with_io_context(|| "doing something");

                assert_matches!(result, Ok(()));
            }

            #[test]
            fn for_error() {
                let result =
                    Err::<(), _>(io::Error::other("oh no")).with_io_context(|| "doing something");

                assert_matches!(result, Err(Error::Io { source, context }) => {
                    assert_eq!(io::ErrorKind::Other, source.kind());
                    assert_eq!("doing something", context);
                });
            }
        }
    }
}
