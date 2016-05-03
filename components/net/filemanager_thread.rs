/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerResult, FileManagerThreadError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use util::thread::spawn_named;
use uuid::Uuid;

pub struct FileManager {
    receiver: IpcReceiver<FileManagerThreadMsg>,
    idmap: RefCell<HashMap<Uuid, String>>,
}

enum SelectedPort {
    Single(IpcSender<FileManagerResult<(Uuid, PathBuf, u64)>>),
    Multiple(IpcSender<FileManagerResult<Vec<(Uuid, PathBuf, u64)>>>)
}

impl FileManager {
    fn new(recv: IpcReceiver<FileManagerThreadMsg>) -> FileManager {
        FileManager {
            receiver: recv,
            idmap: RefCell::new(HashMap::new()),
        }
    }

    fn new_thread() -> IpcSender<FileManagerThreadMsg> {
        let (chan, recv) = ipc::channel().unwrap();

        spawn_named("FileManager".to_owned(), move || {
            FileManager::new(recv).start();
        });

        chan
    }

    /// Start the file manager event loop
    pub fn start(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                FileManagerThreadMsg::SelectFile(sender) => self.select(SelectedPort::Single(sender)),
                FileManagerThreadMsg::SelectFiles(sender) => self.select(SelectedPort::Multiple(sender)),
                FileManagerThreadMsg::ReadFile(sender, id) => self.read_file(sender, id),
                FileManagerThreadMsg::DeleteFileID(id) => self.delete_fileid(id),
            }
        }
    }
}

impl FileManager {
    fn select(&mut self, sender: SelectedPort) {
        // TODO: Pull the dialog UI in and get selected
        match sender {
            SelectedPort::Multiple(sender) => {
                let selected_paths = vec![""];

                let mut replies = vec![];

                for path in selected_paths {
                    match self.create_entry(path) {
                        Some(triple) => replies.push(triple),
                        None => {
                            let _ = sender.send(Err(FileManagerThreadError::InvalidSelection));
                            return;
                        }
                    }
                }

                let _ = sender.send(Ok(replies));
            },
            SelectedPort::Single(sender) => {
                let selected_path = "";

                match self.create_entry(selected_path) {
                    Some(triple) => {
                        let _ = sender.send(Ok(triple));
                    },
                    None => {
                        let _ = sender.send(Err(FileManagerThreadError::InvalidSelection));
                    }
                }
            }
        }
    }

    fn create_entry(&mut self, file_path: &str) -> Option<(Uuid, PathBuf, u64)> {
        match File::open(file_path) {
            Ok(handler) => {
                let id = Uuid::new_v4();
                self.idmap.borrow_mut().insert(id, file_path.to_string());

                // Unix Epoch: https://doc.servo.org/std/time/constant.UNIX_EPOCH.html
                let epoch = handler.metadata().and_then(|metadata| metadata.modified()).map_err(|_| ())
                                              .and_then(|systime| systime.elapsed().map_err(|_| ()))
                                              .and_then(|elapsed| {
                                                    let secs = elapsed.as_secs();
                                                    let nsecs = elapsed.subsec_nanos();
                                                    let msecs = secs * 1000 + nsecs as u64 / 1000000;
                                                    Ok(msecs)
                                                });

                let filename = Path::new(file_path).file_name();

                match (epoch, filename) {
                    (Ok(epoch), Some(filename)) => Some((id, Path::new(filename).to_path_buf(), epoch)),
                    _ => None
                }
            },
            Err(_) => None
        }
    }

    fn read_file(&mut self, sender: IpcSender<FileManagerResult<Vec<u8>>>, id: Uuid) {

        match self.idmap.borrow().get(&id).and_then(|filepath| {
            let mut buffer = vec![];
            match File::open(&filepath) {
                Ok(mut handler) => {
                    match handler.read_to_end(&mut buffer) {
                        Ok(_) => Some(buffer),
                        Err(_) => None,
                    }
                },
                Err(_) => None,
            }
        }) {
            Some(buffer) => {
                let _ = sender.send(Ok(buffer));
            },
            None => {
                let _ = sender.send(Err(FileManagerThreadError::ReadFileError));
            }
        };
    }

    fn delete_fileid(&mut self, id: Uuid) {
        self.idmap.borrow_mut().remove(&id);
    }
}
