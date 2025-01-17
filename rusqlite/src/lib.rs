use std::cell::RefCell;

use rusqlite::Connection;
use stable_fs::{fs::FileSystem, storage::transient::TransientStorage};

mod wasi;
use wasi::inject_shims;

mod conv;
mod polyfill;

thread_local! {
    pub static FILESYSTEM: RefCell<FileSystem> = RefCell::new({
        let s = TransientStorage::new();
        let s = Box::new(s);

        FileSystem::new(s).expect("failed to init filesystem")
    });
}

#[ic_cdk::init]
fn init_fn() {
    inject_shims();

    {
        let conn = Connection::open("db.sqlite3").expect("failed to open connection");

        let v: u8 = conn
            .query_row("SELECT 1", (), |r| r.get(0))
            .expect("failed to query");

        ic_cdk::println!("{v}");
    }
}
