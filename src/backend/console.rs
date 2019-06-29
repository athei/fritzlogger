use super::Backend;
use crate::device::Device;
use crate::errors::*;

use std::time::Instant;

const NAME: &str = "Console";

pub struct Console {}

impl<'de> Backend<'de> for Console {
    type Settings = ();

    fn name() -> &'static str {
        NAME
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
