use anyhow::Result;

pub trait Output {
    fn write(&mut self) -> Result<()>;
}
