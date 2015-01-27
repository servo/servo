/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::unix::ipc::ServoUnixSocket;
use libc::c_int;
use sbsf::{ServoDecoder, ServoEncoder};
use serialize::{Decodable, Encodable};
use std::io::{IoError, MemReader, MemWriter};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct IpcReceiver<T> {
    pipe: Arc<Mutex<ServoUnixSocket>>,
}

impl<T> Clone for IpcReceiver<T> {
    fn clone(&self) -> IpcReceiver<T> {
        IpcReceiver {
            pipe: self.pipe.clone(),
        }
    }
}

pub struct IpcSender<T> {
    pipe: Arc<Mutex<ServoUnixSocket>>,
}

impl<T> Clone for IpcSender<T> {
    fn clone(&self) -> IpcSender<T> {
        IpcSender {
            pipe: self.pipe.clone(),
        }
    }
}

/// Creates a new IPC channel and returns the receiving and sending ends of it respectively.
pub fn channel<T>() -> (IpcReceiver<T>, IpcSender<T>) {
    let (first, second) = ServoUnixSocket::pair().unwrap();
    (IpcReceiver {
        pipe: Arc::new(Mutex::new(first)),
    }, IpcSender {
        pipe: Arc::new(Mutex::new(second)),
    })
}

impl<T> IpcReceiver<T> where T: for<'a> Decodable<ServoDecoder<'a>,IoError> {
    /// Constructs one end of an IPC channel from a file descriptor.
    pub fn from_fd(fd: c_int) -> IpcReceiver<T> {
        IpcReceiver {
            pipe: Arc::new(Mutex::new(ServoUnixSocket::from_fd(fd))),
        }
    }

    /// Constructs an IPC receiver from a raw Unix socket.
    pub fn from_socket(socket: ServoUnixSocket) -> IpcReceiver<T> {
        IpcReceiver {
            pipe: Arc::new(Mutex::new(socket)),
        }
    }

    /// Returns the raw file descriptor backing this IPC receiver.
    pub fn fd(&self) -> c_int {
        self.pipe.lock().fd()
    }

    /// Returns the raw Unix socket backing this IPC receiver.
    pub fn socket<'b>(&'b self) -> MutexGuard<'b,ServoUnixSocket> {
        self.pipe.lock()
    }

    pub fn recv(&self) -> T {
        match self.recv_opt() {
            Ok(msg) => msg,
            Err(err) => panic!("failed to receive over IPC: {}", err),
        }
    }

    pub fn recv_opt(&self) -> Result<T,IoError> {
        let mut pipe = self.pipe.lock();
        let size = try!(pipe.read_le_uint());
        let bytes = try!(pipe.read_exact(size));
        let mut reader = MemReader::new(bytes);
        let mut decoder = ServoDecoder {
            reader: &mut reader,
        };
        Decodable::decode(&mut decoder)
    }
}

impl<T> IpcSender<T> where T: for<'a> Encodable<ServoEncoder<'a>,IoError> {
    /// Constructs one end of an IPC channel from a file descriptor.
    pub fn from_fd(fd: c_int) -> IpcSender<T> {
        IpcSender {
            pipe: Arc::new(Mutex::new(ServoUnixSocket::from_fd(fd))),
        }
    }

    /// Constructs an IPC sender from a raw Unix socket.
    pub fn from_socket(socket: ServoUnixSocket) -> IpcSender<T> {
        IpcSender {
            pipe: Arc::new(Mutex::new(socket)),
        }
    }

    /// Returns the raw file descriptor backing this IPC sender.
    pub fn fd(&self) -> c_int {
        self.pipe.lock().fd()
    }

    /// Returns the raw Unix socket backing this IPC sender.
    pub fn socket<'b>(&'b self) -> MutexGuard<'b,ServoUnixSocket> {
        self.pipe.lock()
    }

    pub fn send(&self, msg: T) {
        match self.send_opt(msg) {
            Ok(()) => {}
            Err(err) => panic!("failed to send over IPC: {}", err),
        }
    }

    pub fn send_opt(&self, msg: T) -> Result<(),IoError> {
        let mut writer = MemWriter::new();
        {
            let mut encoder = ServoEncoder {
                writer: &mut writer,
            };
            try!(msg.encode(&mut encoder));
        }
        let mut pipe = self.pipe.lock();
        try!(pipe.write_le_uint(writer.get_ref().len()));
        pipe.write(writer.get_ref())
    }
}

