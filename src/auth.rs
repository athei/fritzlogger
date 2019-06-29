use crate::xml::{get_child, get_child_text};
use crate::{errors::*, fetch_body};

use futures::{future, Future};
use reqwest::r#async::{Client, RequestBuilder};
use roxmltree::{Document, Node};

use std::str;

const LOCATION_LOGIN: &str = "/login_sid.lua";
const NO_SESSION: &str = "0000000000000000";

#[derive(Debug)]
struct SessionInfo {
    sid: String,
    challenge: String,
    block_time: u32,
    permissions: Vec<Permission>,
}

#[derive(Debug)]
struct Permission {
    kind: PermissionKind,
    level: PermissionLevel,
}

#[derive(Debug, PartialEq, Eq)]
enum PermissionKind {
    BoxAdmin,
    HomeAuto,
    Nas,
    App,
    Phone,
}

#[derive(Debug)]
enum PermissionLevel {
    Read,
    ReadWrite,
}

impl PermissionKind {
    fn parse(node: &Node) -> Option<Self> {
        use PermissionKind::*;
        match node.text()? {
            "BoxAdmin" => Some(BoxAdmin),
            "HomeAuto" => Some(HomeAuto),
            "NAS" => Some(Nas),
            "App" => Some(App),
            "Phone" => Some(Phone),
            _ => None,
        }
    }
}

impl PermissionLevel {
    fn parse(node: &Node) -> Option<Self> {
        use PermissionLevel::*;
        match node.text()? {
            "1" => Some(Read),
            "2" => Some(ReadWrite),
            _ => None,
        }
    }
}

impl SessionInfo {
    fn parse(body: &str) -> Result<Self> {
        let doc = Document::parse(body).chain_err(|| "Cannot decode XML")?;
        let info = get_child(&doc.root(), "SessionInfo")?;
        let sid = get_child_text(&info, "SID")?.to_owned();
        let challenge = get_child_text(&info, "Challenge")?.to_owned();
        let block_time = u32::from_str_radix(get_child_text(&info, "BlockTime")?, 10)
            .chain_err(|| "Cannot convert block_time to number")?;

        let permissions = get_child(&info, "Rights")?
            .children()
            .step_by(2)
            .filter_map(|node| {
                let perm = Permission {
                    kind: PermissionKind::parse(&node)?,
                    level: PermissionLevel::parse(&node.next_sibling()?)?,
                };
                Some(perm)
            })
            .collect();

        let session = Self {
            sid,
            challenge,
            block_time,
            permissions,
        };
        Ok(session)
    }
}

pub fn auth(
    client: &Client,
    base_url: &str,
    username: String,
    password: String,
) -> impl Future<Item = String, Error = Error> {
    let request1 = build_login_request(client, base_url);
    let mut request2 = build_login_request(client, base_url);
    fetch_body(request1)
        .and_then(|body| SessionInfo::parse(&body))
        .and_then(move |session| {
            // no login necessary
            if session.sid != NO_SESSION {
                return future::Either::A(future::ok(session.sid));
            }
            let response = create_response(&session.challenge, &password);
            request2 = request2.query(&[("username", &username), ("response", &response)]);
            future::Either::B(fetch_body(request2))
        })
        .and_then(|body| SessionInfo::parse(&body))
        .and_then(|session| {
            use PermissionKind::HomeAuto;
            if session.sid == NO_SESSION {
                return future::err("Authentication failed (wrong username/password)".into());
            }
            if session
                .permissions
                .iter()
                .find(|perm| perm.kind == HomeAuto)
                .is_none()
            {
                return future::err("User has no home automation permission".into());
            }
            future::ok(session.sid)
        })
}

fn create_response(challenge: &str, password: &str) -> String {
    let input: Vec<u8> = format!("{}-{}", challenge, password)
        .encode_utf16()
        .flat_map(|codepoint| codepoint.to_le_bytes().to_vec())
        .collect();
    format!("{}-{:x}", challenge, md5::compute(input))
}

fn build_login_request(client: &Client, base_url: &str) -> RequestBuilder {
    client.get(&format!("{}{}", base_url, LOCATION_LOGIN))
}
