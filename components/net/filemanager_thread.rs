/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use mime_guess::guess_mime_type_opt;
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerResult};
use net_traits::filemanager_thread::{SelectedFile, FileManagerThreadError};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use util::thread::spawn_named;
use uuid::Uuid;

pub struct FileManager {
    receiver: IpcReceiver<FileManagerThreadMsg>,
    idmap: HashMap<Uuid, PathBuf>,
}

pub trait FileManagerThreadFactory {
    fn new() -> Self;
}

impl FileManagerThreadFactory for IpcSender<FileManagerThreadMsg> {
    /// Create a FileManagerThread
    fn new() -> IpcSender<FileManagerThreadMsg> {
        let (chan, recv) = ipc::channel().unwrap();

        spawn_named("FileManager".to_owned(), move || {
            FileManager::new(recv).start();
        });

        chan
    }
}


impl FileManager {
    fn new(recv: IpcReceiver<FileManagerThreadMsg>) -> FileManager {
        FileManager {
            receiver: recv,
            idmap: HashMap::new(),
        }
    }

    /// Start the file manager event loop
    fn start(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                FileManagerThreadMsg::SelectFile(sender) => self.select_file(sender),
                FileManagerThreadMsg::SelectFiles(sender) => self.select_files(sender),
                FileManagerThreadMsg::ReadFile(sender, id) => self.read_file(sender, id),
                FileManagerThreadMsg::DeleteFileID(id) => self.delete_fileid(id),
                FileManagerThreadMsg::Exit => break,
            }
        }
    }
}

impl FileManager {
    fn select_file(&mut self, sender: IpcSender<FileManagerResult<SelectedFile>>) {
        // TODO: Pull the dialog UI in and get selected
        let selected_path = Path::new("");

        match self.create_entry(selected_path) {
            Some(triple) => {
                let _ = sender.send(Ok(triple));
            },
            None => {
                let _ = sender.send(Err(FileManagerThreadError::InvalidSelection));
            }
        }
    }

    fn select_files(&mut self, sender: IpcSender<FileManagerResult<Vec<SelectedFile>>>) {
        let selected_paths = vec![Path::new("")];

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
    }

    fn create_entry(&mut self, file_path: &Path) -> Option<SelectedFile> {
        match File::open(file_path) {
            Ok(handler) => {
                let id = Uuid::new_v4();
                self.idmap.insert(id, file_path.to_path_buf());

                // Unix Epoch: https://doc.servo.org/std/time/constant.UNIX_EPOCH.html
                let epoch = handler.metadata().and_then(|metadata| metadata.modified()).map_err(|_| ())
                                              .and_then(|systime| systime.elapsed().map_err(|_| ()))
                                              .and_then(|elapsed| {
                                                    let secs = elapsed.as_secs();
                                                    let nsecs = elapsed.subsec_nanos();
                                                    let msecs = secs * 1000 + nsecs as u64 / 1000000;
                                                    Ok(msecs)
                                                });

                let filename = file_path.file_name();

                match (epoch, filename) {
                    (Ok(epoch), Some(filename)) => {
                        let filename_path = Path::new(filename);
                        let mime = guess_mime_type_opt(filename_path);
                        Some(SelectedFile {
                            id: id,
                            filename: filename_path.to_path_buf(),
                            modified: epoch,
                            type_string: match mime {
                                Some(x) => format!("{}", x),
                                None    => "".to_string(),
                            },
                        })
                    }
                    _ => None
                }
            },
            Err(_) => None
        }
    }

    fn read_file(&mut self, sender: IpcSender<FileManagerResult<Vec<u8>>>, id: Uuid) {
        match self.idmap.get(&id).and_then(|filepath| {
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
        self.idmap.remove(&id);
    }
}
