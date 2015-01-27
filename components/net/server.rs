/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Encapsulates the notion of a "server".
//!
//! Servers maintain connections to content threads (in single-process mode) or processes (in
//! multiprocess mode) and serve them resources. Examples of servers are the image cache thread,
//! the resource thread, and the font cache thread.

use libc::c_int;
use serialize::{Decodable, Encodable};
use servo_util::ipc::{mod, IpcReceiver, IpcSender};
use servo_util::platform::unix::ipc as unix_ipc;
use servo_util::platform::unix::ipc::{POLLRDBAND, POLLRDNORM, ServoUnixSocket, pollfd};
use servo_util::sbsf::{ServoDecoder, ServoEncoder};
use servo_util::task_state;
use std::collections::HashMap;
use std::io::IoError;
use std::sync::{Arc, Mutex};
use std::task::TaskBuilder;

/// A server which maintains connections to content threads.
///
/// `M` is the type of a message from the client to the server. `R` is the type of a response from
/// the server to the client (in reply to a synchronous message).
pub struct Server<M,R> {
    /// The name of this server, for debugging.
    name: &'static str,
    /// A list of clients to be serviced.
    clients: HashMap<ClientId,ClientInfo<M,R>>,
}

/// The type of a client ID. On Unix, this is just the receiving-end file descriptor.
pub type ClientId = c_int;

/// Information that the server keeps about each client.
struct ClientInfo<M,R> {
    /// The channel on which messages can be sent to the client.
    sender: IpcSender<ClientMsg<R>>,
    /// The channel on which messages are received from the client.
    receiver: IpcReceiver<ServerMsg<M>>,
}

/// Messages sent to the clients.
#[deriving(Decodable, Encodable)]
enum ClientMsg<R> {
    /// A response to a request.
    Response(R),
}

impl<M,R> Server<M,R> where M: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                               for<'a> Encodable<ServoEncoder<'a>,IoError>,
                            R: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                               for<'a> Encodable<ServoEncoder<'a>,IoError> {
    /// Creates a new server.
    pub fn new(name: &'static str) -> Server<M,R> {
        Server {
            name: name,
            clients: HashMap::new(),
        }
    }

    /// Creates a new client and returns the proxy it uses to communicate with the server.
    pub fn create_new_client(&mut self) -> ServerProxy<M,R> {
        let (client_msg_receiver, client_msg_sender) = ipc::channel();
        let (server_msg_receiver, server_msg_sender) = ipc::channel();
        let client_id = server_msg_receiver.fd();
        self.clients.insert(client_id, ClientInfo {
            sender: client_msg_sender,
            receiver: server_msg_receiver,
        });
        ServerProxy {
            sender: server_msg_sender,
            receiver: client_msg_receiver,
        }
    }

    /// Returns the next message or messages. If `None` is returned, then all clients have exited.
    ///
    /// TODO(pcwalton): Refactor this to not be so Unix-specific. We will need real async I/O
    /// support in Rust or a library to do this.
    pub fn recv(&mut self) -> Option<Vec<(ClientId, M)>> {
        let mut result = Vec::new();
        while result.is_empty() {
            if self.clients.len() == 0 {
                return None
            }

            let mut pollfds = Vec::new();
            for (_, client) in self.clients.iter() {
                pollfds.push(pollfd {
                    fd: client.receiver.fd(),
                    events: POLLRDNORM | POLLRDBAND,
                    revents: 0,
                });
            }

            unix_ipc::poll_fds(pollfds.as_mut_slice(), None).unwrap();

            for pollfd in pollfds.iter() {
                if pollfd.revents == 0 {
                    continue
                }
                let client_id = pollfd.fd;
                match self.clients[client_id].receiver.recv() {
                    ServerMsg::Msg(msg) => result.push((client_id, msg)),
                    ServerMsg::CreateNewClient => {
                        // Create a new pair of sockets and send it to the client.
                        let (mut their_socket, my_sending_socket) =
                            ServoUnixSocket::pair().unwrap();
                        let my_receiving_socket = my_sending_socket.dup();
                        let sender = IpcSender::from_socket(my_sending_socket);
                        let receiver = IpcReceiver::from_socket(my_receiving_socket);
                        let new_client_id = receiver.fd();
                        self.clients.insert(new_client_id, ClientInfo {
                            sender: sender,
                            receiver: receiver,
                        });

                        let fds = [their_socket.fd()];
                        self.clients[client_id].sender.socket().send_fds(&fds).unwrap();
                        their_socket.forget();
                    }
                    ServerMsg::Exit => {
                        self.clients.remove(&client_id);
                    }
                }
            }
        }
        Some(result)
    }

    /// Sends a response to a client.
    pub fn send(&self, client_id: ClientId, response: R) {
        self.clients[client_id].sender.send(ClientMsg::Response(response))
    }
}

/// Messages sent to the server. `M` is the type of the messages specific to this server.
#[deriving(Decodable, Encodable)]
pub enum ServerMsg<M> {
    /// A server-specific asynchronous or synchronous message.
    Msg(M),
    /// Requests that a new client ID be created.
    CreateNewClient,
    /// A notification that the client has exited.
    Exit,
}

/// A proxy from each client to the server.
///
/// `M` is the type of a message from the server to the client. `N` is the type of a notification
/// message from the client to the server. `R` is the type of a request from the client to the
/// server.
pub struct ServerProxy<M,R> where M: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                     for<'a> Encodable<ServoEncoder<'a>,IoError>,
                                  R: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                     for<'a> Encodable<ServoEncoder<'a>,IoError> {
    /// A channel on which messages can be sent to the server.
    sender: IpcSender<ServerMsg<M>>,
    /// A channel on which messages can be received from the server.
    receiver: IpcReceiver<ClientMsg<R>>,
}

impl<M,R> ServerProxy<M,R> where M: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                    for<'a> Encodable<ServoEncoder<'a>,IoError>,
                                 R: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                    for<'a> Encodable<ServoEncoder<'a>,IoError> {
    /// Creates a server proxy from a pair of file descriptors.
    #[inline]
    pub fn from_fds(sender_fd: c_int, receiver_fd: c_int) -> ServerProxy<M,R> {
        ServerProxy {
            sender: IpcSender::from_fd(sender_fd),
            receiver: IpcReceiver::from_fd(receiver_fd),
        }
    }

    /// Returns the raw sending and receiving file descriptors, respectively.
    #[inline]
    pub fn fds(&self) -> (c_int, c_int) {
        (self.sender.fd(), self.receiver.fd())
    }

    /// Leaks the file descriptors! Obviously, you must be careful when using this function.
    ///
    /// The only time this is used at the moment is when serializing the file descriptors over IPC.
    pub fn forget(&mut self) {
        self.sender.socket().forget();
        self.receiver.socket().forget();
    }

    /// Sends an asynchronous message to the server, without waiting for a response.
    pub fn send_async(&self, msg: M) {
        self.sender.send(ServerMsg::Msg(msg))
    }

    /// Sends a request to the server, blocks to wait for a response, and returns it.
    pub fn send_sync(&self, msg: M) -> R {
        self.sender.send(ServerMsg::Msg(msg));
        let ClientMsg::Response(response) = self.receiver.recv();
        return response
    }

    /// Creates a new client, effectively cloning this server proxy.
    pub fn create_new_client(&self) -> ServerProxy<M,R> {
        self.sender.send(ServerMsg::CreateNewClient);

        // Receive our end of the new Unix socket.
        let mut fds = [0];
        assert!(self.receiver.socket().recv_fds(&mut fds) == Ok(1));
        let new_receiver = ServoUnixSocket::from_fd(fds[0]);
        let new_sender = new_receiver.dup();
        ServerProxy {
            sender: IpcSender::from_socket(new_sender),
            receiver: IpcReceiver::from_socket(new_receiver),
        }
    }
}

/// A convenience typedef for the common case of multiple threads in a process sharing a server
/// proxy.
pub type SharedServerProxy<M,R> = Arc<Mutex<ServerProxy<M,R>>>;

#[unsafe_destructor]
impl<M,R> Drop for ServerProxy<M,R> where M: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                             for<'a> Encodable<ServoEncoder<'a>,IoError>,
                                          R: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                             for<'a> Encodable<ServoEncoder<'a>,IoError> {
    fn drop(&mut self) {
        drop(self.sender.send_opt(ServerMsg::Exit));
    }
}

/// Spawns a task with an arrangement to send a particular message to a server if the task fails.
pub fn spawn_named_with_send_to_server_on_failure<M,R>(name: &'static str,
                                                       state: task_state::TaskState,
                                                       body: proc(): Send,
                                                       failure_msg: M,
                                                       failure_dest: SharedServerProxy<M,R>)
                                                       where M: Send +
                                                                'static +
                                                                for<'a>
                                                                Decodable<ServoDecoder<'a>,
                                                                          IoError> +
                                                                for<'a>
                                                                Encodable<ServoEncoder<'a>,
                                                                          IoError>,
                                                             R: for<'a>
                                                                Decodable<ServoDecoder<'a>,
                                                                          IoError> +
                                                                for<'a>
                                                                Encodable<ServoEncoder<'a>,
                                                                          IoError> {
    let future_result = TaskBuilder::new().named(name).try_future(proc() {
        task_state::initialize(state);
        // FIXME: Find replacement for this post-runtime removal
        // rtinstrument::instrument(f);
        body();
    });

    let watched_name = name.into_string();
    let watcher_name = format!("{}Watcher", watched_name);
    TaskBuilder::new().named(watcher_name).spawn(proc() {
        if future_result.into_inner().is_err() {
            debug!("{} failed, notifying constellation", name);
            failure_dest.lock().send_async(failure_msg);
        }
    });
}

