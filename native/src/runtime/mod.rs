use vg_types::Call;

#[cfg(feature = "wasm")]
pub mod wasm;

pub type Error = Box<dyn std::error::Error>;

pub trait Runtime
where
    Self: Sized,
{
    fn load(code: &[u8]) -> Result<Self, Error>;
    fn run_tick(&mut self) -> Result<Vec<Call>, Error>;
    fn send(&mut self, value: vg_types::Response);

    fn serialize(&self) -> Result<Vec<u8>, Error>;
    fn deserialize(bytes: &[u8]) -> Result<Self, Error>;

    fn duplicate(&self) -> Result<Self, Error> {
        let bytes = self.serialize()?;
        Self::deserialize(&bytes)
    }
}
