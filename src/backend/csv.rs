use super::Backend;
use crate::device::Device;
use crate::errors::*;
use crate::settings;

use config::Value;
use csv::{Writer, WriterBuilder};
use serde::{Deserialize, Serialize};

use std::fs::{File, OpenOptions};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Serialize)]
pub struct Settings {
    outfile: String,
}

impl<'de> settings::Settings<'de, Csv> for Settings {
    fn defaults() -> Vec<(String, Value)> {
        vec![("outfile".into(), "./fritzaha.csv".into())]
    }
}

pub struct Csv {
    writer: Writer<File>,
}

#[derive(Serialize)]
struct Record<'a> {
    timestamp: u64,
    id: &'a str,
    temperature: i16,
}

impl<'de> Backend<'de> for Csv {
    type Settings = Settings;

    fn name() -> &'static str {
        "Csv"
    }

    fn new(settings: Self::Settings) -> Result<Self> {
        let ret = Self {
            writer: create_writer(&settings.outfile).chain_err(|| "Cannot open csv outfile")?,
        };
        Ok(ret)
    }

    fn log(&mut self, _: Instant, data: &[Device]) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let records = data.iter().map(|d| Record {
            timestamp,
            id: &d.common.unique_id,
            temperature: d.temperature.as_ref().unwrap().temperature,
        });

        for record in records {
            self.writer
                .serialize(record)
                .chain_err(|| "Error serializing csv record")?;
        }
        self.writer
            .flush()
            .chain_err(|| "Cannot flush out csv recors")?;

        Ok(())
    }
}

fn create_writer(path: &str) -> std::result::Result<Writer<File>, std::io::Error> {
    let mut fbuilder = OpenOptions::new();
    let file_prexists;

    fbuilder.read(false).append(true).truncate(false);
    let file = match fbuilder.create(false).open(path) {
        Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
            file_prexists = false;
            fbuilder.create(true).open(path)
        }
        x @ Ok(_) => {
            file_prexists = true;
            x
        }
        x => {
            file_prexists = false;
            x
        }
    }?;
    Ok(WriterBuilder::new()
        .has_headers(!file_prexists)
        .from_writer(file))
}
