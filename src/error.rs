use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use hecs::ComponentError;

#[derive(Debug)]
pub enum DRError {
    ComponentMissing(String),
    MissingEntity(String)
}

impl Display for DRError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl Error for DRError {}

impl From<ComponentError> for DRError {
    fn from(err: ComponentError) -> Self {
        DRError::ComponentMissing(err.to_string())
    }
}

pub type DRResult<T> = Result<T, DRError>;
