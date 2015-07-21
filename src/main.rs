extern crate rustc_serialize;
extern crate leveldb;
extern crate db_key;
extern crate iron;
extern crate router;

mod resp;
mod qdb;
mod handlers;

use iron::prelude::*;
use router::Router;
use handlers::*;
use std::sync::Mutex;
use std::sync::Arc;
use qdb::QDB;

fn main() {
    let mut router = Router::new();

    router.post("/enqueue", enqueue);
    router.get("/dequeue", dequeue);
    router.get("/statistics", stats);
    router.get("/version", version);
    router.get("/", health);

    let mut chain = Chain::new(router);
    let db = QDB::new("/tmp/db", true).unwrap();
    chain.link_before(DBMW(Arc::new(Mutex::new(db))));


    Iron::new(chain).http("localhost:8080").unwrap();
}
