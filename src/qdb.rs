use leveldb::options::Options;
use leveldb::options::WriteOptions;
use leveldb::options::ReadOptions;
use leveldb::database::Database;
use db_key::Key;
use std::path::Path;
use leveldb::error::Error;
use leveldb::kv::KV;
use leveldb::database::iterator::Iterable;
use std::iter::Iterator;

#[derive(Clone)]
struct Key64(u64);

impl<'a> From<&'a [u8]> for Key64 {
    fn from(key: &'a [u8]) -> Key64 {
        use std::intrinsics::transmute;

        let key: &Key64 = unsafe { transmute(key.as_ptr()) };
        key.clone()
    }
}

impl AsRef<[u8]> for Key64 {
    fn as_ref(&self) -> &[u8] {
        use std::intrinsics::transmute;
        use std::slice::from_raw_parts;
        use std::mem::size_of;

        unsafe { from_raw_parts(transmute(self), size_of::<Key64>()) }
    }
}

impl Key for Key64 {
    fn from_u8(key: &[u8]) -> Self {
        key.into()
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(self.as_ref())
    }
}

pub struct QDB {
    opt: WriteOptions,
    db: Database<Key64>,
}

unsafe impl Send for QDB {}

pub struct Item {
    pub id: u64,
    pub data: Vec<u8>,
}

impl QDB {
    pub fn new(path: &str, sync: bool) -> Result<QDB, Error> {
        let path = Path::new(path);
        let mut database = try!({
            let mut opts = Options::new();
            opts.create_if_missing = true;
            Database::open(path, opts)
        });
        let mut opts = WriteOptions::new();
        opts.sync = true;
        let mut qdb = QDB{ opt: opts, db: database };
        if qdb.health_check() {
            Ok(qdb)
        } else {
            Err(Error::new("DB failed health check".into()))
        }
    }

    fn health_check(&mut self) -> bool {
        true
    }

    pub fn get(&mut self, id: u64) -> Result<Item, Error> {
        let data = try!({
            self.db.get(ReadOptions::new(), Key64(id))
        });
        match data {
            Some(stuff) => Ok(Item{ id: id, data: stuff }),
            None => Err(Error::new("Got None".into())),
        }
    }

    pub fn put(&mut self, qi: Item) -> Result<(), Error> {
        self.db.put(self.opt, Key64(qi.id), qi.data.as_ref())
    }

    pub fn remove(&mut self, id: u64) -> Result<(), Error> {
        self.db.delete(self.opt, Key64(id))
    }

    pub fn next(&mut self, remove: bool) -> Result<Item, Error> {
        let data = self.db.iter(ReadOptions::new()).next();
        match data {
            Some((key, value)) => {
                if remove {
                    match self.remove(key.0) {
                        Ok(_) => {},
                        Err(err) => return Err(err),
                    };
                }
                Ok(Item { id: key.0, data: value })
            }
            None => Err(Error::new("no items in queue".into()))
        }
    }
}
