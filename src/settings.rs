use crate::errors::*;

use config::{Config as CConfig, File, Value};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;
use std::sync::Mutex;

static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    let mut config = Config {
        config: CConfig::new(),
        default_settings: String::new(),
    };
    add_defaults_with_config::<Base>(&mut config).expect("Failed to set base defaults");
    Mutex::new(config)
});

pub trait Settings<'de>: Deserialize<'de> + Serialize + Debug {
    fn defaults() -> Vec<(String, Value)>;
    fn section() -> &'static str;
}

impl<'de> Settings<'de> for () {
    fn defaults() -> Vec<(String, Value)> {
        vec![]
    }

    fn section() -> &'static str {
        ""
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Base {
    pub url: String,
    pub username: String,
    pub password: String,
    pub interval: u64,
}

impl<'de> Settings<'de> for Base {
    fn defaults() -> Vec<(String, Value)> {
        vec![
            ("url".into(), "http://fritz.box".into()),
            ("username".into(), "".into()),
            ("password".into(), "".into()),
            ("interval".into(), 60.into()),
        ]
    }

    fn section() -> &'static str {
        "Base"
    }
}

struct Config {
    config: CConfig,
    default_settings: String,
}

fn add_defaults_with_config<'de, S: Settings<'de>>(config: &mut Config) -> Result<()> {
    for (key, value) in S::defaults() {
        let key = format!("{}.{}", S::section(), key);
        config
            .config
            .set_default(&key, value)
            .chain_err(|| "Failed to set defaults")?;
    }
    if S::section() == "" {
        return Ok(());
    }
    // TODO: handle unwraps
    let defaults: S = get_with_config(config).unwrap();
    let text = format!(
        "[{}]\n{}\n",
        S::section(),
        toml::ser::to_string(&defaults).unwrap()
    );
    config.default_settings.push_str(&text);
    Ok(())
}

fn get_with_config<'de, S: Settings<'de>>(config: &Config) -> Result<S> {
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

pub fn add_defaults<'de, S: Settings<'de>>() -> Result<()> {
    add_defaults_with_config::<S>(&mut CONFIG.lock().unwrap())
}

pub fn get<'de, S: Settings<'de>>() -> Result<S> {
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
