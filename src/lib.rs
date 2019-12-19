pub mod bindings;

use libc::pid_t;
use std::os::raw::{c_int, c_ulong};

/// The `perf_event_open` system call.
///
/// See the `perf_event_open(2)` man page for details.
///
/// Note: The `attrs` argument needs to be a `*mut` because if the `size` field
/// is too small or too large, the kernel writes the size it was expecing back
/// into that field. It might do other things as well.
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
    use crate::bindings::{self, perf_event_attr, perf_event_query_bpf};
    use std::os::raw::{c_char, c_int, c_ulong};

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
        { ENABLE, perf_event_ioctls_ENABLE, c_int }
        { DISABLE, perf_event_ioctls_DISABLE, c_int }
        { REFRESH, perf_event_ioctls_REFRESH, c_int }
        { RESET, perf_event_ioctls_RESET, c_int }
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
        libc::ioctl(fd, ioctl as c_ulong, arg)
    }
}
