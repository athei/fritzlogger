use crate::xml::{get_attrib, get_child, get_child_text};
use crate::{errors::*, fetch_body};

use bitflags::bitflags;
use error_chain::bail;
use futures::Future;
use reqwest::r#async::{Client, RequestBuilder};
use roxmltree::{Document, Node};

use std::sync::Arc;

const LOCATION_AHA: &str = "/webservices/homeautoswitch.lua";
const ROOT_NAME: &str = "devicelist";

#[derive(Debug)]
pub struct Device {
    pub common: Common,
    pub temperature: Option<Temperature>,
    pub powermeter: Option<Powermeter>,
}

bitflags! {
    pub struct Functions: u16 {
        const HANFUN_DEVICE = 0b1;
        const ALARM_SENSOR = 0b1_0000;
        const RADIATOR_CONTROL = 0b100_0000;
        const ENERGY_METER = 0b1000_0000;
        const TEMPERATURE_SENSOR = 0b1_0000_0000;
        const SWITCH_SOCKET = 0b10_0000_0000;
        const AVM_DECT_REPEATER = 0b100_0000_0000;
        const MICROPHONE = 0b1000_0000_0000;
        const HANFUN_UNIT = 0b10_0000_0000_0000;
    }
}

#[derive(Debug)]
pub struct Common {
    pub unique_id: String,
    pub internal_id: u32,
    pub functions: Functions,
    pub fwversion: String,
    pub manufacturer: String,
    pub productname: String,
    pub name: String,
    pub present: bool,
}

#[derive(Debug)]
pub struct Temperature {
    pub temperature: i16,
    pub offset: i16,
}

#[derive(Debug)]
pub struct Powermeter {
    pub voltage: u32,
    pub power: u32,
    pub energy: u32,
}

impl Common {
    fn parse(node: &Node) -> Result<Self> {
        let common = Self {
            unique_id: get_attrib(node, "identifier")?.to_owned(),
            internal_id: u32::from_str_radix(get_attrib(node, "id")?, 10)
                .chain_err(|| "Cannot convert id to number")?,
            functions: Functions::from_bits_truncate(
                u16::from_str_radix(get_attrib(node, "functionbitmask")?, 10)
                    .chain_err(|| "Cannot convert funcions to number")?,
            ),
            fwversion: get_attrib(node, "fwversion")?.to_owned(),
            manufacturer: get_attrib(node, "manufacturer")?.to_owned(),
            productname: get_attrib(node, "productname")?.to_owned(),
            name: get_child_text(node, "name")?.to_owned(),
            present: match get_child_text(node, "present")? {
                "0" => false,
                "1" => true,
                _ => bail!("Present must be 0 or 1"),
            },
        };
        Ok(common)
    }
}

impl Temperature {
    fn parse(node: &Node) -> Result<Self> {
        let temp = get_child(node, "temperature")?;
        let ret = Self {
            temperature: i16::from_str_radix(get_child_text(&temp, "celsius")?, 10)
                .chain_err(|| "Cannot convert temperature to number")?,
            offset: i16::from_str_radix(get_child_text(&temp, "offset")?, 10)
                .chain_err(|| "Cannot convert offset to number")?,
        };
        Ok(ret)
    }
}

impl Powermeter {
    fn parse(node: &Node) -> Result<Self> {
        let power = get_child(node, "powermeter")?;
        let ret = Self {
            voltage: u32::from_str_radix(get_child_text(&power, "voltage")?, 10)
                .chain_err(|| "Cannot convert voltage to number")?,
            power: u32::from_str_radix(get_child_text(&power, "power")?, 10)
                .chain_err(|| "Cannot convert power to number")?,
            energy: u32::from_str_radix(get_child_text(&power, "energy")?, 10)
                .chain_err(|| "Cannot convert energy to number")?,
        };
        Ok(ret)
    }
}

impl Device {
    fn parse(node: &Node) -> Result<Self> {
        let common = Common::parse(node)?;
        let mut temperature = None;
        let mut powermeter = None;

        if common.functions.contains(Functions::TEMPERATURE_SENSOR) {
            temperature = Some(Temperature::parse(node)?);
        }

        if common.functions.contains(Functions::ENERGY_METER) {
            powermeter = Some(Powermeter::parse(node)?);
        }

        let device = Self {
            common,
            temperature,
            powermeter,
        };
        Ok(device)
    }
}

fn parse_devices(body: &str) -> Result<Arc<Vec<Device>>> {
    let doc = Document::parse(body).chain_err(|| "Cannot decode device XML")?;
    let list = get_child(&doc.root(), ROOT_NAME)?;
    let devices: Result<Vec<Device>> = list
        .children()
        .filter_map(|node| {
            let name = node.tag_name().name();
            if name != "device" && name != "group" {
                return None;
            }
            Some(Device::parse(&node))
        })
        .collect();
    Ok(Arc::new(devices?))
}

fn build_request(client: &Client, base_url: &str, sid: &str) -> RequestBuilder {
    client
        .get(&format!("{}{}", base_url, LOCATION_AHA))
        .query(&[("sid", sid), ("switchcmd", "getdevicelistinfos")])
}

pub fn devicelistinfos(
    client: &Client,
    base_url: &str,
    sid: &str,
) -> impl Future<Item = Arc<Vec<Device>>, Error = Error> {
    let request = build_request(client, base_url, sid);
    fetch_body(request).and_then(|body| parse_devices(&body))
}
