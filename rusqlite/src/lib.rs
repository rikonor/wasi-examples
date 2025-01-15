use std::cell::RefCell;

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
}
