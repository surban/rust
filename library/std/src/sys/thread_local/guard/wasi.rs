//! wasm32-wasip1 has pthreads support.

use core::{mem, ptr};

use crate::cell::Cell;
use crate::ffi;
use crate::sys::thread_local::destructors;

// Add a few symbols not in upstream `libc` just yet.
mod libc {
    use crate::ffi;

    #[allow(non_camel_case_types)]
    pub type pthread_key_t = *mut ffi::c_uint;

    extern "C" {
        pub fn pthread_key_create(
            key: *mut pthread_key_t,
            destructor: unsafe extern "C" fn(*mut ffi::c_void),
        ) -> ffi::c_int;

        pub fn pthread_setspecific(key: pthread_key_t, value: *const ffi::c_void) -> ffi::c_int;
    }
}

pub fn enable() {
    #[thread_local]
    static REGISTERED: Cell<bool> = Cell::new(false);

    if !REGISTERED.replace(true) {
        unsafe {
            let mut key: libc::pthread_key_t = mem::zeroed();
            assert_eq!(libc::pthread_key_create(&mut key, run_dtors), 0);

            // We must set the value to a non-NULL pointer value so that
            // the destructor is run on thread exit. The pointer is only
            // passed to run_dtors and never dereferenced.
            assert_eq!(libc::pthread_setspecific(key, ptr::with_exposed_provenance(8)), 0);
        }
    }

    extern "C" fn run_dtors(_: *mut ffi::c_void) {
        unsafe {
            destructors::run();
            crate::rt::thread_cleanup();
        }
    }
}
