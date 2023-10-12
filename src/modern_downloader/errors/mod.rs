use std::{
    error,
    fmt::{self, Display},
};

#[derive(Debug, Clone)]
pub struct NoCdnError;

impl Display for NoCdnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The cdn could not be found")
    }
}
impl error::Error for NoCdnError {}
