extern crate time;

use iron::prelude::*;
use iron::typemap;
use iron::status;
use iron::middleware::BeforeMiddleware;
use std::fmt::{self, Debug};
use std::error::Error;
use qdb::QDB;
use qdb::Item;
use std::sync::{Arc, Mutex};
use std::io::Read;

use self::time::precise_time_ns;

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

static versionStr: &'static str = "1.0.0";

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
    Ok(Response::with("lol"))
}


pub fn dequeue(req: &mut Request) -> IronResult<Response> {
    let mut db = req.extensions.get::<DBMW>().unwrap().lock().unwrap();
    match db.next(true) {
        Err(e) => Err(IronError::new(e, status::InternalServerError)),
        Ok(item) => Ok(Response::with(item.data))
    }
}

pub fn stats(req: &mut Request) -> IronResult<Response> {
    Err(IronError::new(StringError("derp".into()), (status::Forbidden, "lolnope")))
}

pub fn version(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with(versionStr))
}

pub fn health(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "1")))
}
