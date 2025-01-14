use std::{
    slice::{from_raw_parts, from_raw_parts_mut},
    str::from_utf8_unchecked,
};

use scopeguard::defer;
use stable_fs::{
    fs::{
        self, DstBuf as StableFsDstBuf, FdFlags as StableFsFdFlags, FdStat, OpenFlags,
        SrcBuf as StableFsSrcBuf, Whence as StableFsWhence,
    },
    storage::types::{DirEntryIndex as StableFsDirEntryIndex, FileType as StableFsFileType},
};

use wasi_shim::wasi::{
    Advice, Ciovec, Clockid, Dircookie, Dirent, Errno, Event, Exitcode, Fd, Fdflags, Fdstat,
    Filedelta, Filesize, Filestat, Filetype, Fstflags, Iovec, Lookupflags, Oflags, Prestat,
    PrestatDir, PrestatU, Riflags, Rights, Roflags, Sdflags, Siflags, Signal, Size, Subscription,
    Timestamp, Whence, DIRCOOKIE_START, ERRNO_BADF, ERRNO_INVAL, ERRNO_NOTSUP, ERRNO_SUCCESS,
    FD_STDERR, FILETYPE_DIRECTORY, FILETYPE_REGULAR_FILE, FILETYPE_SYMBOLIC_LINK, FSTFLAGS_ATIM,
    FSTFLAGS_ATIM_NOW, FSTFLAGS_MTIM, FSTFLAGS_MTIM_NOW, WHENCE_CUR, WHENCE_END, WHENCE_SET,
};

const ERRNO_NOP: Errno = ERRNO_SUCCESS;

const DEVICE_ZERO: u64 = 0;
const TAG_ZERO: u8 = 0;
const DIRCOOKIE_NEG_ONE: i64 = -1;

const NANOSECOND: u64 = 1;
const MICROSECOND: u64 = 1000 * NANOSECOND;
const MILLISECOND: u64 = 1000 * MICROSECOND;
const SECOND: u64 = 1000 * MILLISECOND;

use crate::FILESYSTEM;

pub fn args_get(_argv: *mut *mut u8, _argv_buf: *mut u8) -> Errno {
    ERRNO_NOP
}

pub fn args_sizes_get(rp0: *mut Size, rp1: *mut Size) -> Errno {
    unsafe {
        *rp0 = 0;
        *rp1 = 0;
    }

    ERRNO_NOP
}

pub fn clock_res_get(_id: Clockid, rp0: *mut Timestamp) -> Errno {
    #[allow(clippy::identity_op)]
    unsafe {
        *rp0 = 1 * SECOND;
    }

    ERRNO_SUCCESS
}

pub fn clock_time_get(_id: Clockid, _precision: Timestamp, rp0: *mut Timestamp) -> Errno {
    unsafe {
        *rp0 = ic_cdk::api::time();
    }

    ERRNO_SUCCESS
}

pub fn environ_sizes_get(rp0: *mut Size, rp1: *mut Size) -> Errno {
    unsafe {
        *rp0 = 0;
        *rp1 = 0;
    }

    ERRNO_SUCCESS
}

pub fn environ_get(_environ: *mut *mut u8, _environ_buf: *mut u8) -> Errno {
    ERRNO_SUCCESS
}

pub fn fd_advise(fd: Fd, offset: Filesize, len: Filesize, advice: Advice) -> Errno {
    let advice = match fs::Advice::try_from(advice.raw()) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL,
    };

    FILESYSTEM.with(|fs| match fs.borrow_mut().advice(fd, offset, len, advice) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_allocate(fd: Fd, offset: Filesize, len: Filesize) -> Errno {
    FILESYSTEM.with(|fs| match fs.borrow_mut().allocate(fd, offset, len) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_close(fd: Fd) -> Errno {
    FILESYSTEM.with(|fs| match fs.borrow_mut().close(fd) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_datasync(fd: Fd) -> Errno {
    FILESYSTEM.with(|fs| match fs.borrow_mut().flush(fd) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_fdstat_get(fd: Fd, rp0: *mut Fdstat) -> Errno {
    let (ftype, fdstat) = match FILESYSTEM.with(|fs| fs.borrow().get_stat(fd)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = Fdstat {
            fs_filetype: convert_filetype(ftype),
            fs_flags: fdstat.flags.bits(),
            fs_rights_base: fdstat.rights_base,
            fs_rights_inheriting: fdstat.rights_inheriting,
        };
    }

    ERRNO_SUCCESS
}

pub fn fd_fdstat_set_flags(fd: Fd, flags: Fdflags) -> Errno {
    let (_, mut fdstat) = match FILESYSTEM.with(|fs| fs.borrow().get_stat(fd)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    let flags = match StableFsFdFlags::from_bits(flags) {
        Some(v) => v,
        None => return ERRNO_INVAL,
    };

    fdstat.flags = flags;

    FILESYSTEM.with(|fs| match fs.borrow_mut().set_stat(fd, fdstat) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_fdstat_set_rights(fd: Fd, fs_rights_base: Rights, fs_rights_inheriting: Rights) -> Errno {
    let (_, mut fdstat) = match FILESYSTEM.with(|fs| fs.borrow().get_stat(fd)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    fdstat.rights_base = fs_rights_base;
    fdstat.rights_inheriting = fs_rights_inheriting;

    FILESYSTEM.with(|fs| match fs.borrow_mut().set_stat(fd, fdstat) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_filestat_get(fd: Fd, rp0: *mut Filestat) -> Errno {
    let md = match FILESYSTEM.with(|fs| fs.borrow().metadata(fd)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = Filestat {
            dev: DEVICE_ZERO,
            ino: md.node,
            filetype: convert_filetype(md.file_type),
            nlink: md.link_count,
            size: md.size,
            atim: md.times.accessed,
            mtim: md.times.modified,
            ctim: md.times.created,
        };
    }

    ERRNO_SUCCESS
}

pub fn fd_filestat_set_size(fd: Fd, size: Filesize) -> Errno {
    FILESYSTEM.with(|fs| match fs.borrow_mut().set_file_size(fd, size) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_filestat_set_times(
    fd: Fd,
    mut atim: Timestamp,
    mut mtim: Timestamp,
    fst_flags: Fstflags,
) -> Errno {
    if let Err(_err) = FILESYSTEM.with(|fs| fs.borrow().metadata(fd)) {
        return ERRNO_INVAL; // TODO
    };

    // ATIM
    if fst_flags & FSTFLAGS_ATIM > 0 {
        if fst_flags & FSTFLAGS_ATIM_NOW > 0 {
            atim = ic_cdk::api::time()
        };

        if let Err(_err) = FILESYSTEM.with(|fs| fs.borrow_mut().set_accessed_time(fd, atim)) {
            return ERRNO_INVAL; // TODO
        };
    }

    // MTIM
    if fst_flags & FSTFLAGS_MTIM > 0 {
        if fst_flags & FSTFLAGS_MTIM_NOW > 0 {
            mtim = ic_cdk::api::time()
        }

        if let Err(_err) = FILESYSTEM.with(|fs| fs.borrow_mut().set_modified_time(fd, mtim)) {
            return ERRNO_INVAL; // TODO
        };
    }

    ERRNO_SUCCESS
}

pub fn fd_pread(
    fd: Fd,
    iovs: *const Iovec,
    iovs_len: i32,
    offset: Filesize,
    rp0: *mut Size,
) -> Errno {
    if fd <= FD_STDERR {
        return ERRNO_INVAL;
    }

    let dst = unsafe {
        from_raw_parts(
            iovs as *const StableFsDstBuf, // data
            iovs_len as usize,             // len
        )
    };

    let s = match FILESYSTEM.with(|fs| fs.borrow_mut().read_vec_with_offset(fd, dst, offset)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = s as usize;
    }

    ERRNO_SUCCESS
}

pub fn fd_prestat_dir_name(fd: Fd, path: *mut u8, path_len: Size) -> Errno {
    if fd != FILESYSTEM.with(|fs| fs.borrow().root_fd()) {
        return ERRNO_BADF;
    }

    let root_path = FILESYSTEM.with(|fs| fs.borrow().root_path().to_owned());

    unsafe {
        for i in 0..path_len {
            path.add(i).write(root_path.as_bytes()[i]);
        }
    }

    ERRNO_SUCCESS
}

pub fn fd_prestat_get(fd: Fd, rp0: *mut Prestat) -> Errno {
    if fd != FILESYSTEM.with(|fs| fs.borrow().root_fd()) {
        return ERRNO_BADF;
    }

    let pr_name_len = FILESYSTEM.with(|fs| fs.borrow().root_path().len());

    unsafe {
        *rp0 = Prestat {
            tag: TAG_ZERO,
            u: PrestatU {
                dir: PrestatDir { pr_name_len },
            },
        };
    }

    ERRNO_SUCCESS
}

pub fn fd_pwrite(
    fd: Fd,
    iovs: *const Iovec,
    iovs_len: i32,
    offset: Filesize,
    rp0: *mut Size,
) -> Errno {
    if fd <= FD_STDERR {
        return ERRNO_NOP;
    }

    let src = unsafe {
        from_raw_parts(
            iovs as *const StableFsSrcBuf, // data
            iovs_len as usize,             // len
        )
    };

    let s = match FILESYSTEM.with(|fs| fs.borrow_mut().write_vec_with_offset(fd, src, offset)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = s as usize;
    }

    ERRNO_SUCCESS
}

pub fn fd_read(fd: Fd, iovs: *const Iovec, iovs_len: i32, rp0: *mut Size) -> Errno {
    if fd <= FD_STDERR {
        return ERRNO_INVAL;
    }

    let dst = unsafe {
        from_raw_parts(
            iovs as *const StableFsDstBuf, // data
            iovs_len as usize,             // len
        )
    };

    let s = match FILESYSTEM.with(|fs| fs.borrow_mut().read_vec(fd, dst)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = s as usize;
    }

    ERRNO_SUCCESS
}

pub fn fd_readdir(fd: Fd, buf: *mut u8, buf_len: Size, cookie: Dircookie, rp0: *mut Size) -> Errno {
    if cookie as i64 == DIRCOOKIE_NEG_ONE {
        unsafe {
            *rp0 = 0;
        }

        return ERRNO_SUCCESS;
    }

    FILESYSTEM.with(|fs| {
        let fs = fs.borrow();

        let md = match FILESYSTEM.with(|fs| fs.borrow().metadata(fd)) {
            Ok(v) => v,
            Err(_err) => return ERRNO_INVAL, // TODO
        };

        let mut idx = match cookie {
            DIRCOOKIE_START => md.first_dir_entry,
            _ => Some(cookie as StableFsDirEntryIndex),
        };

        let buf = unsafe {
            from_raw_parts_mut(
                buf,     // data
                buf_len, // len
            )
        };

        let mut out: usize = 0;

        while let Some(_idx) = idx {
            let e = match fs.get_direntry(fd, _idx) {
                Ok(v) => v,
                Err(_err) => return ERRNO_INVAL, // TODO
            };

            let ftype = match fs.metadata_from_node(e.node) {
                Ok(v) => v.file_type,
                Err(_err) => return ERRNO_INVAL, // TODO
            };

            let p = Dirent {
                d_next: e.next_entry.map(Into::into).unwrap_or(u64::MAX),
                d_ino: _idx as u64,
                d_namlen: e.name.length as u32,
                d_type: convert_filetype(ftype),
            };

            let p: *const Dirent = &p;
            let p: *const u8 = p as *const u8;

            let p = unsafe {
                from_raw_parts(
                    p,                   // data
                    size_of::<Dirent>(), // len
                )
            };

            let n = p.len().min(buf.len());
            buf[0..n].copy_from_slice(&p[0..n]);

            let fname = &e.name.bytes[0..e.name.length as usize];
            let m = fname.len().min(buf.len());
            buf[0..m].copy_from_slice(&fname[0..m]);

            out += n + m;

            if out >= buf_len {
                break;
            }

            idx = e.next_entry;
        }

        unsafe {
            *rp0 = out;
        }

        ERRNO_SUCCESS
    })
}

pub fn fd_renumber(fd: Fd, to: Fd) -> Errno {
    FILESYSTEM.with(|fs| match fs.borrow_mut().renumber(fd, to) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_seek(fd: Fd, offset: Filedelta, whence: Whence, rp0: *mut Filesize) -> Errno {
    if fd <= FD_STDERR {
        return ERRNO_INVAL;
    }

    let _whence = match whence {
        WHENCE_SET => StableFsWhence::SET,
        WHENCE_CUR => StableFsWhence::CUR,
        WHENCE_END => StableFsWhence::END,

        _ => return ERRNO_NOTSUP,
    };

    let s = match FILESYSTEM.with(|fs| fs.borrow_mut().seek(fd, offset, _whence)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = s;
    }

    ERRNO_SUCCESS
}

pub fn fd_sync(fd: Fd) -> Errno {
    FILESYSTEM.with(|fs| match fs.borrow_mut().flush(fd) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn fd_tell(fd: Fd, rp0: *mut Filesize) -> Errno {
    if fd <= FD_STDERR {
        return ERRNO_BADF;
    }

    let p = match FILESYSTEM.with(|fs| fs.borrow_mut().tell(fd)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = p;
    }

    ERRNO_SUCCESS
}

pub fn fd_write(fd: Fd, iovs: *const Iovec, iovs_len: i32, rp0: *mut Size) -> Errno {
    if fd <= FD_STDERR {
        return ERRNO_NOP;
    }

    let src = unsafe {
        from_raw_parts(
            iovs as *const StableFsSrcBuf, // data
            iovs_len as usize,             // len
        )
    };

    let s = match FILESYSTEM.with(|fs| fs.borrow_mut().write_vec(fd, src)) {
        Ok(v) => v,
        Err(_) => return ERRNO_INVAL, // TODO
    };

    unsafe {
        *rp0 = s as usize;
    }

    ERRNO_SUCCESS
}

pub fn path_create_directory(fd: Fd, path: *const u8, path_len: i32) -> Errno {
    let dirname = unsafe {
        from_utf8_unchecked(from_raw_parts(
            path,              // data
            path_len as usize, // len
        ))
    };

    FILESYSTEM.with(|fs| {
        match fs
            .borrow_mut()
            .mkdir(fd, dirname, FdStat::default(), ic_cdk::api::time())
        {
            Ok(_) => ERRNO_SUCCESS,
            Err(_) => ERRNO_INVAL, // TODO
        }
    })
}

pub fn path_filestat_get(
    fd: Fd,
    _flags: Lookupflags,
    path: *const u8,
    path_len: i32,
    rp0: *mut Filestat,
) -> Errno {
    let fname = unsafe {
        from_utf8_unchecked(from_raw_parts(
            path,              // data
            path_len as usize, // len
        ))
    };

    FILESYSTEM.with(|fs| {
        let mut fs = fs.borrow_mut();

        let _fd = match fs.open(
            fd,                  // parent_fd
            fname,               // path
            FdStat::default(),   // stat
            OpenFlags::empty(),  // flags
            ic_cdk::api::time(), // ctime
        ) {
            Ok(v) => v,
            Err(_) => return ERRNO_INVAL, // TODO
        };

        let fs = scopeguard::guard(fs, |mut fs| {
            let _ = fs.close(_fd);
        });

        let md = match fs.metadata(_fd) {
            Ok(v) => v,
            Err(_) => return ERRNO_INVAL, // TODO
        };

        unsafe {
            *rp0 = Filestat {
                dev: DEVICE_ZERO,
                ino: md.node,
                filetype: convert_filetype(md.file_type),
                nlink: md.link_count,
                size: md.size,
                atim: md.times.accessed,
                mtim: md.times.modified,
                ctim: md.times.created,
            };
        }

        ERRNO_SUCCESS
    })
}

pub fn path_filestat_set_times(
    fd: Fd,
    _flags: Lookupflags,
    path: *const u8,
    path_len: i32,
    mut atim: Timestamp,
    mut mtim: Timestamp,
    fst_flags: Fstflags,
) -> Errno {
    let fname = unsafe {
        from_utf8_unchecked(from_raw_parts(
            path,              // data
            path_len as usize, // len
        ))
    };

    FILESYSTEM.with(|fs| {
        let mut fs = fs.borrow_mut();

        let _fd = match fs.open(
            fd,                  // parent_fd
            fname,               // path
            FdStat::default(),   // stat
            OpenFlags::empty(),  // flags
            ic_cdk::api::time(), // ctime
        ) {
            Ok(v) => v,
            Err(_) => return ERRNO_INVAL, // TODO
        };

        let mut fs = scopeguard::guard(fs, |mut fs| {
            let _ = fs.close(_fd);
        });

        let mut md = match fs.metadata(_fd) {
            Ok(v) => v,
            Err(_) => return ERRNO_INVAL, // TODO
        };

        // ATIM
        if fst_flags & FSTFLAGS_ATIM > 0 {
            if fst_flags & FSTFLAGS_ATIM_NOW > 0 {
                atim = ic_cdk::api::time()
            };

            md.times.accessed = atim;
        }

        // MTIM
        if fst_flags & FSTFLAGS_MTIM > 0 {
            if fst_flags & FSTFLAGS_MTIM_NOW > 0 {
                mtim = ic_cdk::api::time()
            }

            md.times.modified = mtim;
        }

        if let Err(_err) = fs.set_metadata(fd, md) {
            return ERRNO_INVAL; // TODO
        }

        ERRNO_SUCCESS
    })
}

pub fn path_link(
    old_fd: Fd,
    _old_flags: Lookupflags,
    old_path: *const u8,
    old_path_len: i32,
    new_fd: Fd,
    new_path: *const u8,
    new_path_len: i32,
) -> Errno {
    let opath = unsafe {
        from_utf8_unchecked(from_raw_parts(
            old_path,              // data
            old_path_len as usize, // len
        ))
    };

    let npath = unsafe {
        from_utf8_unchecked(from_raw_parts(
            new_path,              // data
            new_path_len as usize, // len
        ))
    };

    FILESYSTEM.with(|fs| {
        let mut fs = fs.borrow_mut();

        let _fd = match fs.create_hard_link(old_fd, opath, new_fd, npath) {
            Ok(v) => v,
            Err(_) => return ERRNO_INVAL, // TODO
        };
        defer! {
            let _ = fs.close(_fd);
        }

        ERRNO_SUCCESS
    })
}

#[allow(clippy::too_many_arguments)]
pub fn path_open(
    fd: Fd,
    _dirflags: Lookupflags,
    path: *const u8,
    path_len: i32,
    oflags: Oflags,
    fs_rights_base: Rights,
    fs_rights_inheriting: Rights,
    fdflags: Fdflags,
    rp0: *mut Fd,
) -> Errno {
    let fname = unsafe {
        from_utf8_unchecked(from_raw_parts(
            path,              // data
            path_len as usize, // len
        ))
    };

    let fdstat = FdStat {
        flags: StableFsFdFlags::from_bits_truncate(fdflags),
        rights_base: fs_rights_base,
        rights_inheriting: fs_rights_inheriting,
    };

    FILESYSTEM.with(|fs| {
        let mut fs = fs.borrow_mut();

        let _fd = match fs.open(
            fd,                                    // parent_fd
            fname,                                 // file_name
            fdstat,                                // stat
            OpenFlags::from_bits_truncate(oflags), // flags
            ic_cdk::api::time(),                   // ctime
        ) {
            Ok(v) => v,
            Err(_) => return ERRNO_INVAL, // TODO
        };
        defer! {
            let _ = fs.close(_fd);
        }

        unsafe {
            *rp0 = _fd;
        }

        ERRNO_SUCCESS
    })
}

pub fn path_readlink(
    _fd: Fd,
    _path: *const u8,
    _path_len: i32,
    _buf: *mut u8,
    _buf_len: Size,
    _rp0: *mut Size,
) -> Errno {
    ERRNO_NOTSUP
}

pub fn path_remove_directory(fd: Fd, path: *const u8, path_len: i32) -> Errno {
    let fname = unsafe {
        from_utf8_unchecked(from_raw_parts(
            path,              // data
            path_len as usize, // len
        ))
    };

    FILESYSTEM.with(|fs| match fs.borrow_mut().remove_dir(fd, fname) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn path_rename(
    fd: Fd,
    old_path: *const u8,
    old_path_len: i32,
    new_fd: Fd,
    new_path: *const u8,
    new_path_len: i32,
) -> Errno {
    let opath = unsafe {
        from_utf8_unchecked(from_raw_parts(
            old_path,              // data
            old_path_len as usize, // len
        ))
    };

    let npath = unsafe {
        from_utf8_unchecked(from_raw_parts(
            new_path,              // data
            new_path_len as usize, // len
        ))
    };

    FILESYSTEM.with(|fs| {
        let mut fs = fs.borrow_mut();

        let _fd = match fs.rename(fd, opath, new_fd, npath) {
            Ok(v) => v,
            Err(_) => return ERRNO_INVAL, // TODO
        };
        defer! {
            let _ = fs.close(_fd);
        }

        ERRNO_SUCCESS
    })
}

pub fn path_symlink(
    _old_path: *const u8,
    _old_path_len: i32,
    _fd: Fd,
    _new_path: *const u8,
    _new_path_len: i32,
) -> Errno {
    ERRNO_NOTSUP
}

pub fn path_unlink_file(fd: Fd, path: *const u8, path_len: i32) -> Errno {
    let fname = unsafe {
        from_utf8_unchecked(from_raw_parts(
            path,              // data
            path_len as usize, // len
        ))
    };

    FILESYSTEM.with(|fs| match fs.borrow_mut().remove_file(fd, fname) {
        Ok(_) => ERRNO_SUCCESS,
        Err(_) => ERRNO_INVAL, // TODO
    })
}

pub fn poll_oneoff(
    _in_: *const Subscription,
    _out: *mut Event,
    _nsubscriptions: Size,
    _rp0: *mut Size,
) -> Errno {
    ERRNO_NOTSUP
}

pub fn proc_exit(_rval: Exitcode) -> ! {
    panic!("proc_exit: {_rval}")
}

pub fn proc_raise(_sig: Signal) -> Errno {
    ERRNO_NOTSUP
}

pub fn random_get(_buf: *mut u8, _buf_len: Size) -> Errno {
    ERRNO_SUCCESS
}

pub fn sched_yield() -> Errno {
    ERRNO_NOP
}

pub fn sock_accept(_fd: Fd, _flags: Fdflags, _rp0: *mut Fd) -> Errno {
    ERRNO_NOTSUP
}

pub fn sock_recv(
    _fd: Fd,
    _ri_data: *const Iovec,
    _ri_data_len: i32,
    _ri_flags: Riflags,
    _rp0: *mut Size,
    _rp1: *mut Roflags,
) -> Errno {
    ERRNO_NOTSUP
}

pub fn sock_send(
    _fd: Fd,
    _si_data: *const Ciovec,
    _si_data_len: i32,
    _si_flags: Siflags,
    _rp0: *mut Size,
) -> Errno {
    ERRNO_NOTSUP
}

pub fn sock_shutdown(_fd: Fd, _how: Sdflags) -> Errno {
    ERRNO_NOTSUP
}

fn convert_filetype(ftype: StableFsFileType) -> Filetype {
    match ftype {
        StableFsFileType::Directory => FILETYPE_DIRECTORY,
        StableFsFileType::RegularFile => FILETYPE_REGULAR_FILE,
        StableFsFileType::SymbolicLink => FILETYPE_SYMBOLIC_LINK,
    }
}
