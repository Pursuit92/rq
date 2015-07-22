extern crate time;
extern crate modifier;

use iron::prelude::*;
use iron::typemap;
use iron::status;
use iron::middleware::BeforeMiddleware;
use iron::modifiers::Header;
use std::fmt::{self, Debug};
use std::error::Error;
use qdb::QDB;
use qdb::Item;
use std::sync::{Arc, Mutex};
use std::io::Read;
use rustc_serialize::json::{self, ToJson, Json};
use rustc_serialize::{Encoder, Encodable};
use rustc_serialize::base64::{ToBase64, STANDARD};
use std::ops::Deref;

use self::time::precise_time_ns;
use self::modifier::Modifier;

use resp::*;

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}

pub struct DBMW(pub Arc<Mutex<QDB>>);

impl typemap::Key for DBMW { type Value = Arc<Mutex<QDB>>; }

impl BeforeMiddleware for DBMW {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<DBMW>(self.0.clone());
        Ok(())
    }
}

pub fn enqueue(req: &mut Request) -> IronResult<Response> {
    let mut db = req.extensions.get::<DBMW>().unwrap().lock().unwrap();
    let mut buf = Vec::new();
    match req.body.read_to_end(&mut buf) {
        Err(e) => return Err(IronError::new(e, status::BadRequest)),
        Ok(_) => {}
    };
    match db.put(Item{ id: precise_time_ns(), data: buf }) {
        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
        Ok(_) => {}
    };
    Ok(RqResp::new().into())
}


pub fn dequeue(req: &mut Request) -> IronResult<Response> {
    let mut db = req.extensions.get::<DBMW>().unwrap().lock().unwrap();
    match db.next(true) {
        Err(e) => Err(IronError::new(e, status::InternalServerError)),
        Ok(item) => Ok(RqResp::with(item.data).into())
    }
}

pub fn stats(req: &mut Request) -> IronResult<Response> {
    Err(IronError::new(StringError("derp".into()), (status::Forbidden, "lolnope")))
}

pub fn version(req: &mut Request) -> IronResult<Response> {
    Ok(RqVers.into())
}

pub fn health(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "1")))
}
