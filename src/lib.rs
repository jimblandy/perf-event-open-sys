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
//! As the kernel interface evolves, the struct and union types from the
//! [`bindings`] module may acquire new fields. To ensure that your code will
//! continue to compile against newer versions of this crate, you should
//! construct values of these types by calling their `Default` implementations,
//! which return zero-filled values, and then assigning to the fields you care
//! about. For example:
//!
//! ```
//! use perf_event_open_sys as sys;
//!
//! // Construct a zero-filled `perf_event_attr`.
//! let mut attrs = sys::bindings::perf_event_attr::default();
//!
//! // Populate the fields we need.
//! attrs.size = std::mem::size_of::<sys::bindings::perf_event_attr>() as u32;
//! attrs.type_ = sys::bindings::perf_type_id_PERF_TYPE_HARDWARE;
//! attrs.config = sys::bindings::perf_hw_id_PERF_COUNT_HW_INSTRUCTIONS as u64;
//! attrs.set_disabled(1);
//! attrs.set_exclude_kernel(1);
//! attrs.set_exclude_hv(1);
//!
//! // Make the system call.
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
//! It is not necessary to adjust `size` to what the running kernel expects:
//! older kernels can accept newer `perf_event_attr` structs, and vice versa.
//! The kernel simply checks that any fields it doesn't know about are zero, and
//! pads smaller-than-expected structs with zeros.
//!
//! If the `size` field was properly initialized, an error result of `E2BIG`
//! indicates that the `attrs` structure has requested behavior the kernel is
//! too old to support. When this error code is returned, the kernel writes the
//! size it expected back to the `size` field of the `attrs` struct.
//!
//! This approach works nicely with Linux system call conventions. As a general
//! principle, old fields are never removed from a struct, for backwards
//! compatibility. New fields are always added to the end, and are defined to
//! have no effect when their value is zero. Each system call indicates, through
//! one means or another, the size of the struct being passed (in the case of
//! `perf_event_attr`, the `size` field does this), and this size is used to
//! indicate which version of the API userspace thinks it's using. There are
//! three cases:
//!
//! -   The kernel's own definition of the struct has the same size as the
//!     struct passed from userspace. This means that userspace is using the
//!     same version of the header files as the kernel, so all is well.
//!
//! -   The kernel's struct is larger than the one passed from userspace. This
//!     means the kernel is newer than the userspace program. The kernel copies
//!     the userspace data into the initial bytes of its own struct, and zeros
//!     the remaining bytes. Since zeroed fields have no effect, the resulting
//!     struct properly reflects the user's intent.
//!
//! -   The kernel's struct is smaller than the one passed from userspace. This
//!     means that an executable built on a newer kernel is running on an older
//!     kernel. The kernel checks that the excess bytes in the userspace struct
//!     are all zero; if they are not, the system call returns `E2BIG`,
//!     indicating that userspace has requested a feature the kernel doesn't
//!     support. If they are all zero, then the kernel initializes its own
//!     struct with the bytes from the start of the userspace struct, and drops
//!     the rest. Since the dropped bytes were all zero, they did not affect the
//!     requested behavior, and the resulting struct reflects the user's intent.
//!
//! This approach is explained by the kernel comments for
//! `copy_struct_from_user` in `include/linux/uaccess.h`. The upshot is that
//! older code can run against newer kernels and vice versa, and errors are only
//! returned when the call actually requests functionality that the kernel
//! doesn't support.
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
//! As explained above, bugs aside, it is not necessary to use the version of
//! these structures that matches the kernel you want to run under, so it should
//! always be acceptable to use the latest version of this crate, even if you
//! want to support older kernels.
//!
//! This crate's `README.md` file includes instructions on regenerating the
//! bindings from newer kernel headers. However, this can be a breaking change
//! for users that have not followed the advice above, so regeneration should
//! cause a major version increment.
//!
//! If you need features that are available only in a more recent version of the
//! types than this crate provides, please file an issue.
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
