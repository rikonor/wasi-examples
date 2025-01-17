use std::{cell::RefMut, slice::from_raw_parts};

use stable_fs::fs::{DstBuf, FdStat, FileSystem, OpenFlags, SrcBuf};
use wasi_shim::wasi::Iovec;

pub(crate) fn read_into_vec(mut fs: RefMut<'_, FileSystem>, path: &str) -> Vec<u8> {
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

pub(crate) fn write_vec(mut fs: RefMut<'_, FileSystem>, path: &str, bs: &mut [u8]) {
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
