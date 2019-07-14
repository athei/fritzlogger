use crate::backend::console::Console;
use crate::backend::csv::Csv;
use crate::device::Device;
use crate::errors::*;
use crate::print_errors;
use crate::settings;

use error_chain::bail;
use once_cell::sync::OnceCell;
use tokio::prelude::*;

use std::sync::{Arc, Mutex};
use std::time::Duration;

mod console;
mod csv;

static DISPATCHER: OnceCell<Dispatcher> = OnceCell::new();

pub trait Backend<'de>: settings::Named + Sized {
    type Settings: settings::Settings<'de, Self>;

    fn name() -> &'static str;
    fn new(settings: Self::Settings) -> Result<Self>;
    fn log(&mut self, when: Duration, data: &[Device]) -> Result<()>;
    fn from_settings() -> Result<Self> {
        Self::new(settings::get_for_backend::<Self>()?)
    }
    fn register(list: &mut Vec<String>) -> Result<()> {
        settings::add_defaults::<Self>()?;
        list.push(<Self as Backend>::name().to_owned());
        Ok(())
    }
}

impl<'de, T: Backend<'de>> settings::Named for T {
    fn name() -> &'static str {
        <Self as Backend>::name()
    }
}

struct ToggleBackend<T>(Option<Mutex<T>>);

impl<'de, T> ToggleBackend<T>
where
    T: Backend<'de>,
{
    fn new(backends: &[String]) -> Result<Self> {
        let enabled = backends
            .iter()
            .any(|x| x.as_str() == <T as settings::Named>::name());
        let backend = if enabled {
            Some(Mutex::new(T::from_settings()?))
        } else {
            None
        };
        Ok(Self(backend))
    }
}

pub struct Dispatcher {
    console: ToggleBackend<Console>,
    csv: ToggleBackend<Csv>,
}

impl Dispatcher {
    fn new(enabled_backends: &[String]) -> Result<Self> {
        let backends = Self::register_backends()?;
        settings::refresh()?;

        for backend in enabled_backends {
            if !backends.iter().any(|x| x == backend) {
                bail!(
                    "Backend \"{}\" does not exist. These we do know: {:?}",
                    backend,
                    backends
                );
            }
        }

        let ret = Self {
            console: ToggleBackend::new(enabled_backends)?,
            csv: ToggleBackend::new(enabled_backends)?,
        };
        Ok(ret)
    }

    fn call_backend<'de, B: Backend<'de> + Send>(
        t: Duration,
        devices: Arc<Vec<Device>>,
        backend: &'static ToggleBackend<B>,
    ) {
        // Backend disabled?
        let backend = match backend {
            ToggleBackend(Some(back)) => back,
            _ => return,
        };

        tokio::spawn(future::lazy(move || {
            backend.lock().unwrap().log(t, &devices).map_err(|e| {
                let err = Error::with_chain(e, format!("Backend {} failed", Console::name()));
                print_errors(&err);
            })
        }));
    }

    fn get() -> &'static Self {
        DISPATCHER.get().expect("Dispatcher not initialized.")
    }

    pub fn init(backends: &[String]) -> Result<()> {
        let dispatcher = Self::new(backends)?;
        DISPATCHER
            .set(dispatcher)
            .map_err(|_| "Dispatcher can only be initialized once".into())
    }

    pub fn dispatch(time: Duration, devices: &Arc<Vec<Device>>) {
        let dispatcher = Self::get();
        Self::call_backend(time, devices.clone(), &dispatcher.console);
        Self::call_backend(time, devices.clone(), &dispatcher.csv);
    }

    pub fn register_backends() -> Result<Vec<String>> {
        let mut backends = Vec::with_capacity(2);

        Console::register(&mut backends)?;
        Csv::register(&mut backends)?;

        Ok(backends)
    }
}
