use anyhow::Result;

pub trait Validator {
    fn validate(&self) -> Result<()>;
}
