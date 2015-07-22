extern crate modifier;

use rustc_serialize::{Encodable, Encoder};
use rustc_serialize::base64::{ToBase64, STANDARD};
use rustc_serialize::json;

use iron::headers::{ContentType};
use iron::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use iron::modifiers::Header;

use std::ops::Deref;

use iron::Response;

use self::modifier::{Modifier,Set};

static VERSION: &'static str = "1.0.0";

pub struct RqVers;

impl Modifier<Response> for RqVers {
    fn modify(self, resp: &mut Response) {
        resp.set_mut::<String>(format!(r#"{{"version": {}}}"#, VERSION).into());
        resp.set_mut(json_type());
    }
}

#[derive(RustcEncodable, RustcDecodable)]
struct Stats {
    enqueues: u64,
    dequeues: u64,
    empties: u64,
}

#[derive(Clone)]
struct RqData(Vec<u8>);

#[derive(RustcEncodable, Clone)]
pub struct RqResp {
    success: bool,
    data: Option<Vec<RqData>>,
    message: String,
}

impl Encodable for RqData {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
       s.emit_str(self.0.to_base64(STANDARD).deref())
    }
}

impl Modifier<Response> for RqResp {
    fn modify(self, resp: &mut Response) {
        resp.set_mut(json::encode(&self).unwrap());
        resp.set_mut(json_type());
    }
}

fn json_type() -> Header<ContentType> {
    Header(
        ContentType(
            Mime(TopLevel::Application, SubLevel::Json,
                 vec![(Attr::Charset, Value::Utf8)])))
}

impl RqResp {
    pub fn new() -> RqResp {
        RqResp{ message: "worked".into(), success: true, data: None}
    }

    pub fn with<T>(m: T) -> RqResp
        where T: Modifier<RqResp> {
            let mut resp = RqResp::new();
            m.modify(&mut resp);
            resp
        }
}

impl<'a> Modifier<RqResp> for &'a str {
    fn modify(self, resp: &mut RqResp) {
        resp.message = self.into();
    }
}

impl Modifier<RqResp> for Vec<u8> {
    fn modify(self, resp: &mut RqResp) {
        match resp.data {
            None => resp.data = Some(vec![RqData(self)]),
            Some(ref mut v) => v.push(RqData(self)),
        }
    }
}

impl Modifier<RqResp> for bool {
    fn modify(self, resp: &mut RqResp) {
        resp.success = self;
    }
}

impl From<RqResp> for Response {
    fn from(r: RqResp) -> Response {
        Response::with((r, Header(
            ContentType(
                Mime(TopLevel::Application, SubLevel::Json,
                     vec![(Attr::Charset, Value::Utf8)])))))
    }
}

impl From<RqVers> for Response {
    fn from(r: RqVers) -> Response {
        Response::with((r, Header(
            ContentType(
                Mime(TopLevel::Application, SubLevel::Json,
                     vec![(Attr::Charset, Value::Utf8)])))))
    }
}
