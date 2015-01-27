/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! IPC over Unix sockets.

use alloc::heap;
use libc::{mod, c_char, c_int, c_short, c_uint, c_void, size_t, socklen_t, ssize_t};
use std::io::{IoError, IoResult};
use std::mem;
use std::ptr;

pub struct ServoUnixSocket {
    fd: c_int,
}

impl ServoUnixSocket {
    #[inline]
    pub fn pair() -> IoResult<(ServoUnixSocket, ServoUnixSocket)> {
        let mut results = [0, 0];
        unsafe {
            if socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, &mut results[0]) >= 0 {
                Ok((ServoUnixSocket::from_fd(results[0]), ServoUnixSocket::from_fd(results[1])))
            } else {
                Err(IoError::last_error())
            }
        }
    }

    #[inline]
    pub fn from_fd(fd: c_int) -> ServoUnixSocket {
        ServoUnixSocket {
            fd: fd,
        }
    }

    #[inline]
    pub fn fd(&self) -> c_int {
        self.fd
    }

    pub fn close(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
        self.fd = -1;
    }

    pub fn forget(&mut self) {
        self.fd = -1;
    }

    pub fn dup(&self) -> ServoUnixSocket {
        unsafe {
            let new_fd = libc::dup(self.fd);
            ServoUnixSocket::from_fd(new_fd)
        }
    }

    pub fn send_fds(&self, fds: &[c_int]) -> Result<(),IoError> {
        let cmsg_len = mem::size_of::<cmsghdr>() + fds.len() * mem::size_of::<c_int>();
        let cmsg_buf = unsafe {
            heap::allocate(cmsg_len, mem::min_align_of::<cmsghdr>())
        };
        let cmsg = cmsg_buf as *mut u8 as *mut cmsghdr;
        unsafe {
            (*cmsg).cmsg_len = uint_to_cmsglen(cmsg_len);
            (*cmsg).cmsg_level = libc::SOL_SOCKET;
            (*cmsg).cmsg_type = SCM_RIGHTS;
            ptr::copy_nonoverlapping_memory(cmsg.offset(1) as *mut u8 as *mut c_int,
                                            fds.as_ptr(),
                                            fds.len());
        }

        let mut dummy_data: c_char = 0;
        let mut iovec = iovec {
            iov_base: &mut dummy_data,
            iov_len: 1,
        };

        let msghdr = msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iovec,
            msg_iovlen: 1,
            msg_control: cmsg as *mut c_void,
            msg_controllen: uint_to_msg_controllen(cmsg_len),
            msg_flags: 0,
        };

        let result;
        unsafe {
            result = sendmsg(self.fd, &msghdr, 0);
            heap::deallocate(cmsg_buf, cmsg_len, mem::min_align_of::<cmsghdr>());
        }
        match result {
            length if length > 0 => Ok(()),
            _ => {
                error!("FD send failed");
                Err(IoError::last_error())
            }
        }
    }

    pub fn recv_fds(&self, fds: &mut [c_int]) -> Result<u32,IoError> {
        let cmsg_len = mem::size_of::<cmsghdr>() + fds.len() * mem::size_of::<c_int>();
        let cmsg_buf = unsafe {
            heap::allocate(cmsg_len, mem::align_of::<cmsghdr>())
        };
        let cmsg = cmsg_buf as *mut u8 as *mut cmsghdr;

        let mut dummy_data: c_char = 0;
        let mut iovec = iovec {
            iov_base: &mut dummy_data,
            iov_len: 1,
        };

        let mut msghdr = msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iovec,
            msg_iovlen: 1,
            msg_control: cmsg as *mut c_void,
            msg_controllen: uint_to_msg_controllen(cmsg_len),
            msg_flags: 0,
        };

        unsafe {
            let result = recvmsg(self.fd, &mut msghdr, 0);
            heap::deallocate(cmsg_buf, cmsg_len, mem::min_align_of::<cmsghdr>());
            match result {
                length if length > 0 => {}
                _ => {
                    error!("FD receive failed");
                    return Err(IoError::last_error())
                }
            }

            let mut fd_count = ((*cmsg).cmsg_len as uint - mem::size_of::<cmsghdr>()) /
                mem::size_of::<c_int>();
            if fd_count > fds.len() {
                // FIXME(pcwalton): Should probably close any extraneous FDs that we got.
                fd_count = fds.len()
            }
            ptr::copy_nonoverlapping_memory(fds.as_mut_ptr(),
                                            cmsg.offset(1) as *const u8 as *const c_int,
                                            fds.len());
            Ok(fd_count as c_uint)
        }
    }
}
impl Drop for ServoUnixSocket {
    #[inline(never)]
    fn drop(&mut self) {
        self.close()
    }
}

impl Writer for ServoUnixSocket {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        unsafe {
            let result = libc::send(self.fd, buf.as_ptr() as *const c_void, buf.len() as u64, 0);
            if result == buf.len() as i64 {
                Ok(())
            } else {
                Err(IoError::last_error())
            }
        }
    }
}


impl Reader for ServoUnixSocket {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        unsafe {
            match libc::recv(self.fd, buf.as_mut_ptr() as *mut c_void, buf.len() as u64, 0) {
                length if length > 0 => Ok(length as uint),
                0 => Err(IoError::from_errno(54, false)),
                _ => {
                    error!("read failed!");
                    Err(IoError::last_error())
                }
            }
        }
    }
}

/// Polls the given set of file descriptors, exactly as `poll(2)` does.
pub fn poll_fds(pollfds: &mut [pollfd], timeout: Option<c_int>) -> Result<(),IoError> {
    unsafe {
        if poll(pollfds.as_mut_ptr(), pollfds.len() as c_uint, timeout.unwrap_or(-1)) < -1 {
            Err(IoError::last_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(target_os="macos")]
fn uint_to_cmsglen(cmsglen: uint) -> c_uint {
    cmsglen as c_uint
}
#[cfg(target_os="linux")]
fn uint_to_cmsglen(cmsglen: uint) -> size_t {
    cmsglen as size_t
}
#[cfg(target_os="macos")]
fn uint_to_msg_controllen(msg_controllen: uint) -> socklen_t {
    msg_controllen as socklen_t
}
#[cfg(target_os="linux")]
fn uint_to_msg_controllen(msg_controllen: uint) -> size_t {
    msg_controllen as size_t
}

// FFI stuff follows:

extern {
    fn poll(fds: *mut pollfd, nfds: c_uint, timeout: c_int) -> c_int;
    fn recvmsg(socket: c_int, message: *mut msghdr, flags: c_int) -> ssize_t;
    fn sendmsg(socket: c_int, message: *const msghdr, flags: c_int) -> ssize_t;
    fn socketpair(domain: c_int, socket_type: c_int, protocol: c_int, sv: *mut c_int) -> c_int;
}

pub const POLLRDNORM: c_short = 0x0040;
pub const POLLRDBAND: c_short = 0x0080;
const SCM_RIGHTS: c_int = 0x01;

#[cfg(target_os="macos")]
#[repr(C)]
struct msghdr {
    msg_name: *mut c_void,
    msg_namelen: socklen_t,
    msg_iov: *mut iovec,
    msg_iovlen: c_int,
    msg_control: *mut c_void,
    msg_controllen: socklen_t,
    msg_flags: c_int,
}

#[cfg(target_os="linux")]
#[repr(C)]
struct msghdr {
    msg_name: *mut c_void,
    msg_namelen: socklen_t,
    msg_iov: *mut iovec,
    msg_iovlen: size_t,
    msg_control: *mut c_void,
    msg_controllen: size_t,
    msg_flags: c_int,
}

#[repr(C)]
struct iovec {
    iov_base: *mut c_char,
    iov_len: size_t,
}

#[cfg(target_os="macos")]
#[repr(C)]
struct cmsghdr {
    cmsg_len: c_uint,
    cmsg_level: c_int,
    cmsg_type: c_int,
}

#[cfg(target_os="linux")]
#[repr(C)]
struct cmsghdr {
    cmsg_len: size_t,
    cmsg_level: c_int,
    cmsg_type: c_int,
}

#[repr(C)]
pub struct pollfd {
    pub fd: c_int,
    pub events: c_short,
    pub revents: c_short,
}

