#[derive(RustcEncodable, RustcDecodable)]
struct Stats {
    enqueues: u64,
    dequeues: u64,
    empties: u64,
}

#[derive(RustcEncodable, RustcDecodable)]
struct Resp {
    success: bool,
    data: Vec<String>,
    message: String,
}
