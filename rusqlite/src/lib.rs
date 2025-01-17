use std::cell::RefCell;

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use rusqlite::{Connection, Row};
use rw::{read_into_vec, write_vec};
use stable_fs::{fs::FileSystem, storage::transient::TransientStorage};

mod wasi;
use wasi::inject_shims;

mod conv;
mod polyfill;
mod rw;

thread_local! {
    pub static FILESYSTEM: RefCell<FileSystem> = RefCell::new({
        let s = TransientStorage::new();
        let s = Box::new(s);

        FileSystem::new(s).expect("failed to init filesystem")
    });
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = {
        let mm = DefaultMemoryImpl::default();
        let mm = MemoryManager::init(mm);

        RefCell::new(mm)
    };

    static BACKUP: RefCell<StableBTreeMap<(), Vec<u8>, VirtualMemory<DefaultMemoryImpl>>> = {
        let m = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)));
        let v = StableBTreeMap::init(m);

        RefCell::new(v)
    }
}

thread_local! {
    pub static CONN: RefCell<Connection> = RefCell::new({
        Connection::open("db.sqlite3").expect("failed to open connection")
    });
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade_fn() {
    let bs = FILESYSTEM.with(|fs| {
        read_into_vec(
            fs.borrow_mut(), // fs
            "db.sqlite3",    // path
        )
    });

    BACKUP.with(|m| {
        m.borrow_mut().insert(
            (), // k
            bs, // v
        )
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade_fn() {
    inject_shims();

    let mut bs = BACKUP
        .with(|m| m.borrow().get(&()))
        .expect("no database backup");

    FILESYSTEM.with(|fs| {
        write_vec(
            fs.borrow_mut(), // fs
            "db.sqlite3",    // path
            &mut bs,         // bs
        );
    });
}

#[ic_cdk::init]
fn init_fn() {
    inject_shims();

    CONN.with(|conn| {
        let conn = conn.borrow_mut();

        let q = "
            CREATE TABLE persons (
                id   INTEGER PRIMARY KEY,
                name TEXT    NOT NULL
            )
        ";

        conn.execute(q, []).expect("failed to execute");

        let q = "INSERT INTO persons (name) VALUES (?1)";

        conn.execute(q, ["Or"]).expect("failed to execute");
        conn.execute(q, ["Laura"]).expect("failed to execute");
        conn.execute(q, ["Jacob"]).expect("failed to execute");
        conn.execute(q, ["Sadie"]).expect("failed to execute");
    });
}

#[ic_cdk::query]
fn trigger_query() {
    CONN.with(|conn| {
        let conn = conn.borrow_mut();

        let mut stmt = conn
            .prepare("SELECT id, name FROM persons")
            .expect("failed to prepare statement");

        let f = |r: &Row| {
            Ok((
                r.get(0).expect("failed to get column"), // id
                r.get(1).expect("failed to get column"), // name
            ))
        };

        let ps = stmt.query_map([], f).expect("failed to query");
        for p in ps {
            let (id, name): (i32, String) = p.expect("failed to read row");
            ic_cdk::println!("{} {}", id, name);
        }
    });
}
