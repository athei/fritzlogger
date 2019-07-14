use crate::backend::console::Console;
use crate::backend::csv::Csv;
use crate::device::Device;
use crate::errors::*;
use crate::print_errors;
use crate::settings;

use once_cell::sync::Lazy;
use tokio::prelude::*;

use std::sync::{Arc, Mutex};
use std::time::Instant;

mod console;
mod csv;

static DISPATCHER: Lazy<Dispatcher> =
    Lazy::new(|| Dispatcher::new().expect("Failed to initiate Backends"));

pub trait Backend<'de>: settings::Named + Sized {
    type Settings: settings::Settings<'de, Self>;

    fn name() -> &'static str;
    fn new(settings: Self::Settings) -> Result<Self>;
    fn log(&mut self, when: Instant, data: &[Device]) -> Result<()>;
    fn from_settings() -> Result<Self> {
        Self::new(settings::get_for_backend::<Self>()?)
    }
    fn add_defaults() -> Result<()> {
        settings::add_defaults::<Self>()
    }
}

impl<'de, T: Backend<'de>> settings::Named for T {
    fn name() -> &'static str {
        <Self as Backend>::name()
    }
}

pub struct Dispatcher {
    console: Mutex<Console>,
    csv: Mutex<Csv>,
}

impl Dispatcher {
    fn new() -> Result<Self> {
        Self::register_defaults()?;
        settings::refresh()?;
        let ret = Self {
            console: Mutex::new(Console::from_settings()?),
            csv: Mutex::new(Csv::from_settings()?),
        };
        Ok(ret)
    }

    fn call_backend<'de, B: Backend<'de> + Send>(
        t: Instant,
        devices: Arc<Vec<Device>>,
        backend: &'static Mutex<B>,
    ) {
        tokio::spawn(future::lazy(move || {
            backend.lock().unwrap().log(t, &devices).map_err(|e| {
                let err = Error::with_chain(e, format!("Backend {} failed", Console::name()));
                print_errors(&err);
            })
        }));
    }

    pub fn dispatch(time: Instant, devices: &Arc<Vec<Device>>) {
        Self::call_backend(time, devices.clone(), &DISPATCHER.console);
        Self::call_backend(time, devices.clone(), &DISPATCHER.csv);
    }

    pub fn register_defaults() -> Result<()> {
        Console::add_defaults()?;
        Csv::add_defaults()?;

        Ok(())
    }
}
