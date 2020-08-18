//! Direct, unsafe bindings for Linux [`perf_event_open`][man] and friends.
//!
//! Linux's `perf_event_open` system call provides access to the processor's
//! performance measurement counters (things like instructions retired, cache
//! misses, and so on), kernel counters (context switches, page faults), and
//! many other sources of performance information.
//!
//! You can't get the `perf_event_open` function from the `libc` crate, as you
//! would any other system call. The Linux standard C library does not provide a
//! binding for this function or its associated types and constants.
//!
//! Rust analogs to the C types and constants from `<linux/perf_event.h>` and
//! `<linux/hw_breakpoint.h>`, generated with `bindgen`, are available in the
//! [`bindings`] module.
//!
//! There are several ioctls for use with `perf_event_open` file descriptors;
//! see the [`ioctls`] module for those.
//!
//! ## Using the raw API
//!
//! All the struct and union types from the [`bindings`] module implement the
//! `Default` trait by zeroing the entire struct. This works nicely with Linux
//! system call conventions. Over time, as a kernel interface evolves, its
//! structs get new fields added to them. As a general principle, a newly added
//! field is always placed at the end of the struct, and is defined to have no
//! effect if its value is zero. So, using this crate, if you produce a struct
//! using its `Default::default` method and then initialize only the fields you
//! need, your code should continue to compile even as newer versions of this
//! crate are updated for newer versions of the kernel interface.
//!
//! For example:
//!
//! ```
//! use perf_event_open_sys as sys;
//!
//! let mut attrs = sys::bindings::perf_event_attr::default();
//!
//! attrs.size = std::mem::size_of::<sys::bindings::perf_event_attr>() as u32;
//! attrs.type_ = sys::bindings::perf_type_id_PERF_TYPE_HARDWARE;
//! attrs.config = sys::bindings::perf_hw_id_PERF_COUNT_HW_INSTRUCTIONS as u64;
//! attrs.set_disabled(1);
//! attrs.set_exclude_kernel(1);
//! attrs.set_exclude_hv(1);
//!
//! let result = unsafe {
//!     sys::perf_event_open(&mut attrs, 0, -1, -1, 0)
//! };
//!
//! if result < 0 {
//!     // ... handle error
//! }
//!
//! // ... use `result` as a raw file descriptor
//! ```
//!
//! You can find one example of using `perf_event_open` in the [`perf_event`]
//! crate, which provides a safe interface to a subset of `perf_event_open`'s
//! functionality.
//!
//! ### Kernel versions
//!
//! The bindings in this crate are generated from the Linux kernel headers
//! packaged by Fedora as `kernel-headers-5.6.11-100.fc30.x86_64`, which
//! corresponds to `PERF_EVENT_ATTR_SIZE_VER6`.
//!
//! It should always be acceptable (again, bugs aside) to regenerate this
//! crate's bindings from a newer kernel. As explained above, bugs aside, it is
//! not necessary to use the version of these structures that matches the kernel
//! you want to run under. The system call interface is designed so that older
//! kernels can handle newer structs, and vice versa. The system call fails only
//! if the structure requests functionality that the running kernel does not
//! actually support.
//!
//! Users of this crate should be using the `default` method to initialize
//! structs, as documented above, so new fields should not break properly
//! written code.
//!
//! If you need features available only in a more recent version of the type
//! than this crate provides, please file an issue.
//!
//! [`bindings`]: bindings/index.html
//! [`ioctls`]: ioctls/index.html
//! [man]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
//! [`perf_event`]: https://crates.io/crates/perf_event

pub mod bindings;

use libc::pid_t;
use std::os::raw::{c_int, c_ulong};

/// The `perf_event_open` system call.
///
/// See the [`perf_event_open(2) man page`][man] for details.
///
/// On error, this returns a negated raw OS error value. The C `errno` value is
/// not changed.
///
/// Note: The `attrs` argument needs to be a `*mut` because if the `size` field
/// is too small or too large, the kernel writes the size it was expecing back
/// into that field. It might do other things as well.
///
/// [man]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
pub unsafe fn perf_event_open(
    attrs: *mut bindings::perf_event_attr,
    pid: pid_t,
    cpu: c_int,
    group_fd: c_int,
    flags: c_ulong,
) -> c_int {
    libc::syscall(
        bindings::__NR_perf_event_open as libc::c_long,
        attrs as *const bindings::perf_event_attr,
        pid,
        cpu,
        group_fd,
        flags,
    ) as c_int
}

#[allow(dead_code, non_snake_case)]
pub mod ioctls {
    //! Ioctls for use with `perf_event_open` file descriptors.
    //!
    //! See the [`perf_event_open(2)`][man] man page for details.
    //!
    //! On error, these return `-1` and set the C `errno` value.
    //!
    //! [man]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    use crate::bindings::{self, perf_event_attr, perf_event_query_bpf};
    use std::os::raw::{c_char, c_int, c_uint, c_ulong};

    macro_rules! define_ioctls {
        ( $( $args:tt )* ) => {
            $(
                define_ioctl!($args);
            )*
        }
    }

    macro_rules! define_ioctl {
        ({ $name:ident, $ioctl:ident, $arg_type:ty }) => {
            pub unsafe fn $name(fd: c_int, arg: $arg_type) -> c_int {
                untyped_ioctl(fd, bindings::$ioctl, arg)
            }
        };
    }

    define_ioctls! {
        { ENABLE, perf_event_ioctls_ENABLE, c_uint }
        { DISABLE, perf_event_ioctls_DISABLE, c_uint }
        { REFRESH, perf_event_ioctls_REFRESH, c_int }
        { RESET, perf_event_ioctls_RESET, c_uint }
        { PERIOD, perf_event_ioctls_PERIOD, u64 }
        { SET_OUTPUT, perf_event_ioctls_SET_OUTPUT, c_int }
        { SET_FILTER, perf_event_ioctls_SET_FILTER, *mut c_char }
        { ID, perf_event_ioctls_ID, *mut u64 }
        { SET_BPF, perf_event_ioctls_SET_BPF, u32 }
        { PAUSE_OUTPUT, perf_event_ioctls_PAUSE_OUTPUT, u32 }
        { QUERY_BPF, perf_event_ioctls_QUERY_BPF, *mut perf_event_query_bpf }
        { MODIFY_ATTRIBUTES, perf_event_ioctls_MODIFY_ATTRIBUTES, *mut perf_event_attr }
    }

    unsafe fn untyped_ioctl<A>(
        fd: c_int,
        ioctl: bindings::perf_event_ioctls,
        arg: A,
    ) -> c_int {
        #[cfg(target_env = "musl")]
        return libc::ioctl(fd, ioctl as c_int, arg);

        #[cfg(not(target_env = "musl"))]
        libc::ioctl(fd, ioctl as c_ulong, arg)
    }
}
