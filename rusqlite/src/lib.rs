use std::{
    cell::{RefCell, RefMut},
    slice::from_raw_parts,
};

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use rusqlite::{Connection, Row};
use stable_fs::{
    fs::{DstBuf, FdStat, FileSystem, OpenFlags, SrcBuf},
    storage::transient::TransientStorage,
};

mod wasi;
use wasi::inject_shims;
use wasi_shim::wasi::Iovec;

mod conv;
mod polyfill;

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

    let conn = Connection::open("db.sqlite3").expect("failed to open connection");

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
}

#[ic_cdk::query]
fn trigger_query() {
    let conn = Connection::open("db.sqlite3").expect("failed to open connection");

    struct Person {
        id: i32,
        name: String,
    }

    let q = "
        SELECT id, name
        FROM persons
    ";

    let mut stmt = conn.prepare(q).expect("failed to prepare statement");

    let f = |r: &Row| {
        Ok(Person {
            id: r.get(0).expect("failed to get column"),
            name: r.get(1).expect("failed to get column"),
        })
    };

    let ps = stmt.query_map([], f).expect("failed to query");
    for p in ps {
        let p = p.expect("failed to read row");

        ic_cdk::println!("{} {}", p.id, p.name);
    }
}

fn read_into_vec(mut fs: RefMut<'_, FileSystem>, path: &str) -> Vec<u8> {
    let _fd = fs
        .open(
            3,
            path,
            FdStat::default(),
            OpenFlags::empty(),
            ic_cdk::api::time(),
        )
        .expect("failed to open file");

    let mut fs = scopeguard::guard(fs, |mut fs| {
        let _ = fs.close(_fd);
    });

    let md = fs
        .metadata(_fd)
        .expect("failed to get the opened file metadata");

    let mut dst = vec![0; md.size as usize];

    let mut buf = vec![Iovec {
        buf: dst.as_mut_ptr(),
        buf_len: dst.len(),
    }];

    let bs: &[DstBuf] = unsafe {
        from_raw_parts(
            buf.as_mut_ptr() as *const DstBuf, //
            buf.len(),                         //
        )
    };

    fs.read_vec(_fd, bs).expect("failed to read file");

    dst
}

fn write_vec(mut fs: RefMut<'_, FileSystem>, path: &str, bs: &mut [u8]) {
    let _fd = fs
        .open(
            3,
            path,
            FdStat::default(),
            OpenFlags::CREATE | OpenFlags::EXCLUSIVE | OpenFlags::TRUNCATE,
            ic_cdk::api::time(),
        )
        .expect("failed to open file");

    let mut fs = scopeguard::guard(fs, |mut fs| {
        let _ = fs.close(_fd);
    });

    let mut buf = [Iovec {
        buf: bs.as_mut_ptr(),
        buf_len: bs.len(),
    }];

    let bs: &[SrcBuf] = unsafe {
        from_raw_parts(
            buf.as_mut_ptr() as *const SrcBuf, //
            buf.len(),                         //
        )
    };

    fs.write_vec(_fd, bs).expect("failed to write");
}
