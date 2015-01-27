/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A Linux-specific sandbox using `seccomp-bpf`.

#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]

use compositing::content_process::Zone;

use libc::{AF_INET, AF_INET6, AF_UNIX, O_NONBLOCK, O_RDONLY, c_int, c_ulong, c_ushort};

pub type rlim_t = u64;

const AF_NETLINK: c_int = 16;

const CLONE_VM: c_int = 0x00000100;
const CLONE_FS: c_int = 0x00000200;
const CLONE_FILES: c_int = 0x00000400;
const CLONE_SIGHAND: c_int = 0x00000800;
const CLONE_THREAD: c_int = 0x00010000;
const CLONE_SYSVSEM: c_int = 0x00040000;
const CLONE_SETTLS: c_int = 0x00080000;
const CLONE_PARENT_SETTID: c_int = 0x00100000;
const CLONE_CHILD_CLEARTID: c_int = 0x00200000;

const O_NOCTTY: c_int = 256;
const O_CLOEXEC: c_int = 524288;

const FIONREAD: c_int = 0x541b;
const NETLINK_ROUTE: c_int = 0;

#[repr(C)]
struct rlimit {
    rlim_cur: rlim_t,
    rlim_max: rlim_t,
}

const RLIMIT_FSIZE: c_int = 1;

const EM_X86_64: u32 = 62;

const NR_read: u32 = 0;
const NR_write: u32 = 1;
const NR_open: u32 = 2;
const NR_close: u32 = 3;
const NR_stat: u32 = 4;
const NR_fstat: u32 = 5;
const NR_poll: u32 = 7;
const NR_lseek: u32 = 8;
const NR_mmap: u32 = 9;
const NR_mprotect: u32 = 10;
const NR_munmap: u32 = 11;
const NR_brk: u32 = 12;
const NR_rt_sigreturn: u32 = 15;
const NR_ioctl: u32 = 16;
const NR_access: u32 = 21;
const NR_madvise: u32 = 28;
const NR_socket: u32 = 41;
const NR_connect: u32 = 42;
const NR_sendto: u32 = 44;
const NR_recvfrom: u32 = 45;
const NR_recvmsg: u32 = 47;
const NR_bind: u32 = 49;
const NR_getsockname: u32 = 51;
const NR_clone: u32 = 56;
const NR_exit: u32 = 60;
const NR_readlink: u32 = 89;
const NR_getuid: u32 = 102;
const NR_sigaltstack: u32 = 131;
const NR_futex: u32 = 202;
const NR_sched_getaffinity: u32 = 204;
const NR_exit_group: u32 = 231;
const NR_set_robust_list: u32 = 273;
const NR_sendmmsg: u32 = 307;
const NR_unknown_318: u32 = 318;

const __AUDIT_ARCH_64BIT: u32 = 0x80000000;
const __AUDIT_ARCH_LE: u32 = 0x40000000;
const AUDIT_ARCH_X86_64: u32 = EM_X86_64 | __AUDIT_ARCH_64BIT | __AUDIT_ARCH_LE;

const PR_SET_SECCOMP: c_int = 22;
const PR_SET_NO_NEW_PRIVS: c_int = 38;

const SECCOMP_MODE_FILTER: c_ulong = 2;

#[repr(C)]
struct sock_filter {
    code: u16,
    jt: u8,
    jf: u8,
    k: u32,
}

#[repr(C)]
struct sock_fprog {
    len: c_ushort,
    filter: *const sock_filter,
}

const BPF_LD: u16 = 0x00;
const BPF_JMP: u16 = 0x05;
const BPF_RET: u16 = 0x06;

const BPF_W: u16 = 0x00;
const BPF_ABS: u16 = 0x20;

const BPF_JA: u16 = 0x00;
const BPF_JEQ: u16 = 0x10;
const BPF_JGT: u16 = 0x20;
const BPF_JGE: u16 = 0x30;
const BPF_JSET: u16 = 0x40;

const BPF_K: u16 = 0x00;

// The syscall structure:
const SYSCALL_NR_OFFSET: u32 = 0;
const ARCH_NR_OFFSET: u32 = 4;
const IP_OFFSET: u32 = 8;
const ARG_0_OFFSET: u32 = 16;
const ARG_1_OFFSET: u32 = 24;
const ARG_2_OFFSET: u32 = 32;

const ARCH_NR: u32 = AUDIT_ARCH_X86_64;

const SECCOMP_RET_KILL: u32 = 0;
const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;

macro_rules! bpf_stmt {
    ($code:expr, $k:expr) => (
        sock_filter {
            code: $code,
            jt: 0,
            jf: 0,
            k: $k,
        }
    )
}

macro_rules! bpf_jump {
    ($code:expr, $k:expr, $jt:expr, $jf:expr) => (
        sock_filter {
            code: $code,
            jt: $jt,
            jf: $jf,
            k: $k,
        }
    )
}

const BPF_VALIDATE_ARCHITECTURE_0: sock_filter =
    bpf_stmt!(BPF_LD+BPF_W+BPF_ABS, ARCH_NR_OFFSET);
const BPF_VALIDATE_ARCHITECTURE_1: sock_filter =
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, ARCH_NR, 1, 0);
const BPF_VALIDATE_ARCHITECTURE_2: sock_filter =
    bpf_stmt!(BPF_RET+BPF_K, SECCOMP_RET_KILL);

const BPF_EXAMINE_SYSCALL: sock_filter =
    bpf_stmt!(BPF_LD+BPF_W+BPF_ABS, SYSCALL_NR_OFFSET);
const BPF_EXAMINE_ARG_0: sock_filter =
    bpf_stmt!(BPF_LD+BPF_W+BPF_ABS, ARG_0_OFFSET);
const BPF_EXAMINE_ARG_1: sock_filter =
    bpf_stmt!(BPF_LD+BPF_W+BPF_ABS, ARG_1_OFFSET);
const BPF_EXAMINE_ARG_2: sock_filter =
    bpf_stmt!(BPF_LD+BPF_W+BPF_ABS, ARG_2_OFFSET);

macro_rules! bpf_allow_syscall_if {
    ($id:ident) => (
        bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, $id, 0, 1)
    )
}

const BPF_ALLOW_SYSCALL: sock_filter =
    bpf_stmt!(BPF_RET+BPF_K, SECCOMP_RET_ALLOW);

const BPF_KILL_PROCESS: sock_filter =
    bpf_stmt!(BPF_RET+BPF_K, SECCOMP_RET_KILL);

// TODO(pcwalton): When the resource task is rewritten, remove network access.
static FILTER: [sock_filter, ..93] = [
    BPF_VALIDATE_ARCHITECTURE_0,
    BPF_VALIDATE_ARCHITECTURE_1,
    BPF_VALIDATE_ARCHITECTURE_2,

    // Special handling for open(2): only allow file reading.
    BPF_EXAMINE_SYSCALL,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, NR_open, 0, 3),
    BPF_EXAMINE_ARG_1,
    bpf_jump!(BPF_JMP+BPF_JSET+BPF_K,
              !(O_RDONLY | O_CLOEXEC | O_NOCTTY | O_NONBLOCK) as u32,
              1,
              0),
    BPF_ALLOW_SYSCALL,

    // Special handling for socket(2): only allow `AF_UNIX`, `AF_INET`, `AF_INET6`, or
    // `PF_NETLINK` with `NETLINK_ROUTE` protocol.
    BPF_EXAMINE_SYSCALL,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, NR_socket, 0, 11),
    BPF_EXAMINE_ARG_0,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, AF_UNIX as u32, 0, 1),
    BPF_ALLOW_SYSCALL,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, AF_INET as u32, 0, 1),
    BPF_ALLOW_SYSCALL,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, AF_INET6 as u32, 0, 1),
    BPF_ALLOW_SYSCALL,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, AF_NETLINK as u32, 0, 3),
    BPF_EXAMINE_ARG_2,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, NETLINK_ROUTE as u32, 0, 1),
    BPF_ALLOW_SYSCALL,

    // Special handling for ioctl(2): only allow `FIONREAD`.
    BPF_EXAMINE_SYSCALL,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, NR_ioctl, 0, 3),
    BPF_EXAMINE_ARG_1,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, FIONREAD as u32, 0, 1),
    BPF_ALLOW_SYSCALL,

    // Special handling for clone(2): only allow normal threads to be created.
    BPF_EXAMINE_SYSCALL,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K, NR_clone, 0, 3),
    BPF_EXAMINE_ARG_0,
    bpf_jump!(BPF_JMP+BPF_JEQ+BPF_K,
              (CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD | CLONE_SYSVSEM |
               CLONE_SETTLS | CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID) as u32,
              0,
              1),
    BPF_ALLOW_SYSCALL,

    BPF_EXAMINE_SYSCALL,
    bpf_allow_syscall_if!(NR_rt_sigreturn),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_exit_group),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_exit),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_read),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_write),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_mmap),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_mprotect),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_set_robust_list),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_close),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_sigaltstack),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_futex),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_recvmsg),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_sched_getaffinity),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_munmap),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_recvfrom),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_readlink),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_stat),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_madvise),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_fstat),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_lseek),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_sendto),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_unknown_318),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_brk),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_bind),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_getsockname),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_connect),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_access),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_poll),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_sendmmsg),
    BPF_ALLOW_SYSCALL,
    bpf_allow_syscall_if!(NR_getuid),
    BPF_ALLOW_SYSCALL,
    BPF_KILL_PROCESS,
];

/// Enters the Linux sandbox.
///
/// The Zone doesn't do anything here, because I don't know any way to restrict which files can be
/// opened on Linux without `chroot`, and that's a privileged syscall.
pub fn enter(_: Zone) {
    unsafe {
        // Disallow writing by setting the max writable size to 0.
        let rlimit = rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if setrlimit(RLIMIT_FSIZE, &rlimit) != 0 {
            panic!("setrlimit(RLIMIT_FSIZE) failed")
        }

        if prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
            panic!("prctl(PR_SET_NO_NEW_PRIVS) failed")
        }

        let program = sock_fprog {
            len: FILTER.len() as c_ushort,
            filter: FILTER.as_ptr(),
        };
        if prctl(PR_SET_SECCOMP,
                 SECCOMP_MODE_FILTER,
                 &program as *const sock_fprog as uint as c_ulong,
                 -1,
                 0) != 0 {
            panic!("prctl(PR_SET_SECCOMP) failed")
        }
    }
}

extern {
    fn prctl(option: c_int, arg2: c_ulong, arg3: c_ulong, arg4: c_ulong, arg5: c_ulong) -> c_int;
    fn setrlimit(resource: c_int, rlim: *const rlimit) -> c_int;
}

