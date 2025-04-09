#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod fs {
    use std::fs as real_fs;
    use std::io;
    use std::path::Path;

    #[cfg_attr(test, mockall::concretize)]
    pub fn copy<P, Q>(from: P, to: Q) -> io::Result<u64>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        real_fs::copy(from, to)
    }

    #[cfg_attr(test, mockall::concretize)]
    pub fn rename<P, Q>(from: P, to: Q) -> io::Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        real_fs::rename(from, to)
    }
}
