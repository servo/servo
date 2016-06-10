/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_loader;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use mime_classifier::MIMEClassifier;
use mime_guess::guess_mime_type_opt;
use net_traits::blob_url_store::{BlobURLStoreEntry, BlobURLStoreError, BlobURLStoreMsg};
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerResult};
use net_traits::filemanager_thread::{SelectedFile, FileManagerThreadError, SelectedFileId};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use url::{Url, Origin};
use util::thread::spawn_named;
use uuid::Uuid;

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

struct FileManager {
    receiver: IpcReceiver<FileManagerThreadMsg>,
    idmap: HashMap<Uuid, PathBuf>,
    classifier: Arc<MIMEClassifier>,
    blob_url_store: Arc<RwLock<BlobURLStore>>,
}

impl FileManager {
    fn new(recv: IpcReceiver<FileManagerThreadMsg>) -> FileManager {
        FileManager {
            receiver: recv,
            idmap: HashMap::new(),
            classifier: Arc::new(MIMEClassifier::new()),
            blob_url_store: Arc::new(RwLock::new(BlobURLStore::new())),
        }
    }

    /// Start the file manager event loop
    fn start(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                FileManagerThreadMsg::SelectFile(sender) => self.select_file(sender),
                FileManagerThreadMsg::SelectFiles(sender) => self.select_files(sender),
                FileManagerThreadMsg::ReadFile(sender, id) => {
                    match self.try_read_file(id) {
                        Ok(buffer) => { let _ = sender.send(Ok(buffer)); }
                        Err(_) => { let _ = sender.send(Err(FileManagerThreadError::ReadFileError)); }
                    }
                }
                FileManagerThreadMsg::DeleteFileID(id) => self.delete_fileid(id),
                FileManagerThreadMsg::BlobURLStoreMsg(msg) => self.blob_url_store.write().unwrap().process(msg),
                FileManagerThreadMsg::LoadBlob(load_data, consumer) => {
                    blob_loader::load(load_data, consumer,
                                      self.blob_url_store.clone(),
                                      self.classifier.clone());
                },
                FileManagerThreadMsg::Exit => break,
            };
        }
    }
}

impl FileManager {
    fn select_file(&mut self, sender: IpcSender<FileManagerResult<SelectedFile>>) {
        // TODO: Pull the dialog UI in and get selected
        // XXX: "test.txt" is "tests/unit/net/test.txt", for temporary testing purpose
        let selected_path = Path::new("test.txt");

        match self.create_entry(selected_path) {
            Some(triple) => { let _ = sender.send(Ok(triple)); }
            None => { let _ = sender.send(Err(FileManagerThreadError::InvalidSelection)); }
        };
    }

    fn select_files(&mut self, sender: IpcSender<FileManagerResult<Vec<SelectedFile>>>) {
        let selected_paths = vec![Path::new("test.txt")];

        let mut replies = vec![];

        for path in selected_paths {
            match self.create_entry(path) {
                Some(triple) => replies.push(triple),
                None => { let _ = sender.send(Err(FileManagerThreadError::InvalidSelection)); }
            };
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
                            id: SelectedFileId(id.simple().to_string()),
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

    fn try_read_file(&mut self, id: SelectedFileId) -> Result<Vec<u8>, ()> {
        let id = try!(Uuid::parse_str(&id.0).map_err(|_| ()));

        match self.idmap.get(&id) {
            Some(filepath) => {
                let mut buffer = vec![];
                let mut handler = try!(File::open(&filepath).map_err(|_| ()));
                try!(handler.read_to_end(&mut buffer).map_err(|_| ()));
                Ok(buffer)
            },
            None => Err(())
        }
    }

    fn delete_fileid(&mut self, id: SelectedFileId) {
        if let Ok(id) = Uuid::parse_str(&id.0) {
            self.idmap.remove(&id);
        }
    }
}

pub struct BlobURLStore {
    entries: HashMap<Uuid, (Origin, BlobURLStoreEntry)>,
}

impl BlobURLStore {
    pub fn new() -> BlobURLStore {
        BlobURLStore {
            entries: HashMap::new(),
        }
    }

    fn process(&mut self, msg: BlobURLStoreMsg) {
        match msg {
            BlobURLStoreMsg::AddEntry(entry, origin_str, sender) => {
                match Url::parse(&origin_str) {
                    Ok(base_url) => {
                        let id = Uuid::new_v4();
                        self.add_entry(id, base_url.origin(), entry);

                        let _ = sender.send(Ok(id.simple().to_string()));
                    }
                    Err(_) => {
                        let _ = sender.send(Err(BlobURLStoreError::InvalidOrigin));
                    }
                }
            }
            BlobURLStoreMsg::DeleteEntry(id) => {
                if let Ok(id) = Uuid::parse_str(&id) {
                    self.delete_entry(id);
                }
            },
        }
    }

    pub fn request(&self, id: Uuid, origin: &Origin) -> Result<&BlobURLStoreEntry, BlobURLStoreError> {
        match self.entries.get(&id) {
            Some(ref pair) => {
                if pair.0 == *origin {
                    Ok(&pair.1)
                } else {
                    Err(BlobURLStoreError::InvalidOrigin)
                }
            }
            None => Err(BlobURLStoreError::InvalidKey)
        }
    }

    pub fn add_entry(&mut self, id: Uuid, origin: Origin, blob: BlobURLStoreEntry) {
        self.entries.insert(id, (origin, blob));
    }

    pub fn delete_entry(&mut self, id: Uuid) {
        self.entries.remove(&id);
    }
}

