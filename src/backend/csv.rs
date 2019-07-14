use super::Backend;
use crate::device::Device;
use crate::errors::*;
use crate::settings;

use config::Value;
use csv::{Writer, WriterBuilder};
use serde::{Deserialize, Serialize};

use std::fs::{File, OpenOptions};
use std::time::Duration;

#[derive(Deserialize, Serialize)]
pub struct Settings {
    out_dir: String,
}

impl<'de> settings::Settings<'de, Csv> for Settings {
    fn defaults() -> Vec<(String, Value)> {
        vec![("out_dir".into(), ".".into())]
    }
}

pub struct Csv {
    writer_temperature: Writer<File>,
    writer_energy: Writer<File>,
}

#[derive(Serialize)]
struct RecordTemperature<'a> {
    timestamp: u64,
    id: &'a str,
    temperature: i16,
    offset: i16,
}

#[derive(Serialize)]
struct RecordEnergy<'a> {
    timestamp: u64,
    id: &'a str,
    voltage: u32,
    power: u32,
}

impl<'de> Backend<'de> for Csv {
    type Settings = Settings;

    fn name() -> &'static str {
        "Csv"
    }

    fn new(settings: Self::Settings) -> Result<Self> {
        let ret = Self {
            writer_temperature: Self::create_writer(&format!(
                "{}/temperature.csv",
                &settings.out_dir
            ))
            .chain_err(|| "Cannot open temperature outfile")?,
            writer_energy: Self::create_writer(&format!("{}/energy.csv", &settings.out_dir))
                .chain_err(|| "Cannot open energy outfile")?,
        };
        Ok(ret)
    }

    fn log(&mut self, when: Duration, data: &[Device]) -> Result<()> {
        self.write_temperature(&when, data)?;
        self.write_energy(&when, data)?;
        Ok(())
    }
}

impl Csv {
    fn write_temperature(&mut self, when: &Duration, data: &[Device]) -> Result<()> {
        let timestamp = when.as_secs();
        let records = data.iter().filter_map(|d| {
            let temperature = match &d.temperature {
                Some(ref temperature) => temperature,
                _ => return None,
            };

            let record = RecordTemperature {
                timestamp,
                id: &d.common.unique_id,
                temperature: temperature.temperature,
                offset: temperature.offset,
            };
            Some(record)
        });

        for record in records {
            self.writer_temperature
                .serialize(record)
                .chain_err(|| "Error serializing csv record")?;
        }
        self.writer_temperature
            .flush()
            .chain_err(|| "Cannot flush out csv records")?;

        Ok(())
    }

    fn write_energy(&mut self, when: &Duration, data: &[Device]) -> Result<()> {
        let timestamp = when.as_secs();
        let records = data.iter().filter_map(|d| {
            let energy = match &d.powermeter {
                Some(energy) => energy,
                _ => return None,
            };

            let record = RecordEnergy {
                timestamp,
                id: &d.common.unique_id,
                voltage: energy.voltage,
                power: energy.power,
            };
            Some(record)
        });

        for record in records {
            self.writer_energy
                .serialize(record)
                .chain_err(|| "Error serializing csv record")?;
        }
        self.writer_energy
            .flush()
            .chain_err(|| "Cannot flush out csv records")?;

        Ok(())
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
}
