use wasi_shim::wasi::{Errno, Size, ERRNO_SUCCESS};

fn environ_sizes_get(_rp0: *mut Size, _rp1: *mut Size) -> Errno {
    ERRNO_SUCCESS
}

fn random_get(_buf: *mut u8, _buf_len: Size) -> Errno {
    ERRNO_SUCCESS
}

pub(crate) fn inject_shims() {
    unsafe {
        wasi_shim::core::environ::set::environ_sizes_get(environ_sizes_get);
        wasi_shim::core::random::set::random_get(random_get);
    }
}
