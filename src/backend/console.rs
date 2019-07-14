use super::Backend;
use crate::device::Device;
use crate::errors::*;
use crate::settings;

use std::time::Instant;

pub struct Console {}

impl<'de> Backend<'de> for Console {
    type Settings = settings::No<Self>;

    fn name() -> &'static str {
        "Console"
    }

    fn new(_: Self::Settings) -> Result<Self> {
        let ret = Self {};
        Ok(ret)
    }

    fn log(&mut self, _: Instant, data: &[Device]) -> Result<()> {
        println!("{:#?}", data);
        Ok(())
    }
}
