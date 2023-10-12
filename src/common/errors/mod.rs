use std::error;
use std::fmt::{ self, Display };

#[derive(Debug, Clone)]
pub struct NoTotalSize;

impl Display for NoTotalSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Could not find total size for request and inable to validate it against the RbxPkgManifest"
        )
    }
}
impl error::Error for NoTotalSize {}

#[derive(Debug, Clone)]
pub struct BadTotalSize;

impl Display for BadTotalSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The total size is not equal to the expected total size")
    }
}
impl error::Error for BadTotalSize {}

#[derive(Debug, Clone)]
pub struct InvalidMd5Hash;

impl Display for InvalidMd5Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The md5 hash generated is not the same as that defined in the rbxPkgManifest")
    }
}
impl error::Error for InvalidMd5Hash {}

#[derive(Debug, Clone)]
pub struct ZipExtractionError;

impl Display for ZipExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The zip has failed to be extracted")
    }
}
impl error::Error for ZipExtractionError {}

#[derive(Debug, Clone)]
pub struct FileCreationError;

impl Display for FileCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not create a file")
    }
}
impl error::Error for FileCreationError {}

#[derive(Debug, Clone)]
pub struct FileAlreadyExistError;

impl Display for FileAlreadyExistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not create a file since one already exist")
    }
}
impl error::Error for FileAlreadyExistError {}

#[derive(Debug, Clone)]
pub struct BadCliUse;

impl Display for BadCliUse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Please use the exe in the way described when running with no args")
    }
}
impl error::Error for BadCliUse {}
