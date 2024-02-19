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

    #[error("I/O error while {context:?}: {source:?}")]
    Io { source: io::Error, context: String },

    #[error(transparent)]
    Toml(#[from] TomlError),

    #[error("Invalid manifest path: {0}")]
    InvalidManifestPath(Utf8PathBuf),

    #[error("Backup manifest already exists: {0}")]
    BackupManifestAlreadyExists(Utf8PathBuf),
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
    /// Type returned from [`with_io_context`](IoErrorContext::with_io_context).
    type Output;

    /// Provides context for the I/O error.
    ///
    /// See [trait description](IoErrorContext) for details.
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
