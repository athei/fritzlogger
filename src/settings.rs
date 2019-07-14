use crate::backend::Backend;
use crate::errors::*;

use config::{Config as CConfig, File, Value};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;
use std::sync::Mutex;

static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    let mut config = Config {
        config: CConfig::new(),
        default_settings: String::new(),
    };
    add_defaults_with_config::<Base, Base>(&mut config).expect("Failed to set base defaults");
    Mutex::new(config)
});

pub trait Named {
    fn name() -> &'static str;
}

pub trait Settings<'de, T: Named>: Deserialize<'de> + Serialize {
    fn defaults() -> Vec<(String, Value)>;
    fn section() -> &'static str {
        T::name()
    }
}

#[derive(Deserialize, Serialize)]
pub struct No<T: Named> {
    #[serde(skip_serializing, skip_deserializing)]
    _dummy: bool,
    #[serde(skip_serializing, skip_deserializing)]
    _phantom: PhantomData<T>,
}

impl<'de, T: Named> Settings<'de, T> for No<T> {
    fn defaults() -> Vec<(String, Value)> {
        vec![("dummy".into(), true.into())]
    }
}

#[derive(Deserialize, Serialize)]
pub struct Base {
    pub url: String,
    pub username: String,
    pub password: String,
    pub interval: u64,
}

impl Named for Base {
    fn name() -> &'static str {
        "Base"
    }
}

impl<'de> Settings<'de, Base> for Base {
    fn defaults() -> Vec<(String, Value)> {
        vec![
            ("url".into(), "http://fritz.box".into()),
            ("username".into(), "".into()),
            ("password".into(), "".into()),
            ("interval".into(), 60.into()),
        ]
    }
}

struct Config {
    config: CConfig,
    default_settings: String,
}

fn add_defaults_with_config<'de, T: Named, S: Settings<'de, T>>(config: &mut Config) -> Result<()> {
    for (key, value) in S::defaults() {
        let key = format!("{}.{}", S::section(), key);
        config
            .config
            .set_default(&key, value)
            .chain_err(|| "Failed to set defaults")?;
    }
    let defaults: S = get_with_config(config)?;
    let section = S::section();
    let body = toml::ser::to_string(&defaults)
        .chain_err(|| format!("Failed to serialize default values of backend: {}", section))?;

    // this backend has no settings
    if body.is_empty() {
        return Ok(());
    }

    config
        .default_settings
        .push_str(&format!("[{}]\n{}\n", section, body));
    Ok(())
}

fn get_with_config<'de, T: Named, S: Settings<'de, T>>(config: &Config) -> Result<S> {
    config
        .config
        .get(S::section())
        .chain_err(|| format!("Cannot get settings for {}", S::section()))
}

pub fn load(path: &str) -> Result<()> {
    CONFIG
        .lock()
        .unwrap()
        .config
        .merge(File::with_name(path))
        .chain_err(|| "Failed to load config file")?;
    Ok(())
}

pub fn add_defaults<'de, B: Backend<'de>>() -> Result<()> {
    add_defaults_with_config::<B, B::Settings>(&mut CONFIG.lock().unwrap())
}

pub fn get_base() -> Result<Base> {
    get_with_config::<Base, Base>(&CONFIG.lock().unwrap())
}

pub fn get_for_backend<'de, T: Backend<'de>>() -> Result<T::Settings> {
    get_with_config(&CONFIG.lock().unwrap())
}

pub fn refresh() -> Result<()> {
    CONFIG
        .lock()
        .unwrap()
        .config
        .refresh()
        .map(|_| ())
        .chain_err(|| "Failed to load backend settings.")
}

pub fn defaults() -> Result<String> {
    crate::backend::Dispatcher::register_defaults()?;
    Ok(CONFIG.lock().unwrap().default_settings.clone())
}
