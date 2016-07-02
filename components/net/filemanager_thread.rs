/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_loader;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use mime_classifier::MimeClassifier;
use mime_guess::guess_mime_type_opt;
use net_traits::blob_url_store::{BlobURLStoreEntry, BlobURLStoreError, BlobURLStoreMsg};
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerResult, FilterPattern};
use net_traits::filemanager_thread::{SelectedFile, FileManagerThreadError, SelectedFileId};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use tinyfiledialogs;
use url::{Url, Origin};
use util::thread::spawn_named;
use uuid::Uuid;

pub trait FileManagerThreadFactory<UI: 'static + UIProvider> {
    fn new(&'static UI) -> Self;
}

pub trait UIProvider where Self: Sync {
    fn open_file_dialog(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<String>;

    fn open_file_dialog_multi(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<Vec<String>>;
}

pub struct TFDProvider;

impl UIProvider for TFDProvider {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn open_file_dialog(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<String> {
        let mut filter = vec![];
        for p in patterns {
            let s = "*.".to_string() + &p.0;
            filter.push(s)
        }

        let filter_ref = &(filter.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[..]);

        let filter_opt = if filter.len() > 0 { Some((filter_ref, "")) } else { None };

        tinyfiledialogs::open_file_dialog("Pick a file", path, filter_opt)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn open_file_dialog_multi(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<Vec<String>> {
        let mut filter = vec![];
        for p in patterns {
            let s = "*.".to_string() + &p.0;
            filter.push(s)
        }

        let filter_ref = &(filter.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[..]);

        let filter_opt = if filter.len() > 0 { Some((filter_ref, "")) } else { None };

        tinyfiledialogs::open_file_dialog_multi("Pick files", path, filter_opt)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn open_file_dialog(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<String> {
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn open_file_dialog_multi(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<Vec<String>> {
        None
    }
}

impl<UI: 'static + UIProvider> FileManagerThreadFactory<UI> for IpcSender<FileManagerThreadMsg> {
    /// Create a FileManagerThread
    fn new(ui: &'static UI) -> IpcSender<FileManagerThreadMsg> {
        let (chan, recv) = ipc::channel().unwrap();

        spawn_named("FileManager".to_owned(), move || {
            FileManager::new(recv, ui).start();
        });

        chan
    }
}

struct FileManager<UI: 'static + UIProvider> {
    receiver: IpcReceiver<FileManagerThreadMsg>,
    idmap: HashMap<Uuid, PathBuf>,
    classifier: Arc<MimeClassifier>,
    blob_url_store: Arc<RwLock<BlobURLStore>>,
    ui: &'static UI,
}

impl<UI: 'static + UIProvider> FileManager<UI> {
    fn new(recv: IpcReceiver<FileManagerThreadMsg>, ui: &'static UI) -> FileManager<UI> {
        FileManager {
            receiver: recv,
            idmap: HashMap::new(),
            classifier: Arc::new(MimeClassifier::new()),
            blob_url_store: Arc::new(RwLock::new(BlobURLStore::new())),
            ui: ui
        }
    }

    /// Start the file manager event loop
    fn start(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                FileManagerThreadMsg::SelectFile(filter, sender) => self.select_file(filter, sender),
                FileManagerThreadMsg::SelectFiles(filter, sender) => self.select_files(filter, sender),
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

    fn select_file(&mut self, patterns: Vec<FilterPattern>,
                   sender: IpcSender<FileManagerResult<SelectedFile>>) {
        match self.ui.open_file_dialog("", patterns) {
            Some(s) => {
                let selected_path = Path::new(&s);

                match self.create_entry(selected_path) {
                    Some(triple) => { let _ = sender.send(Ok(triple)); }
                    None => { let _ = sender.send(Err(FileManagerThreadError::InvalidSelection)); }
                };
            }
            None => {
                let _ = sender.send(Err(FileManagerThreadError::UserCancelled));
                return;
            }
        }
    }

    fn select_files(&mut self, patterns: Vec<FilterPattern>,
                    sender: IpcSender<FileManagerResult<Vec<SelectedFile>>>) {
        match self.ui.open_file_dialog_multi("", patterns) {
            Some(v) => {
                let mut selected_paths = vec![];

                for s in &v {
                    selected_paths.push(Path::new(s));
                }

                let mut replies = vec![];

                for path in selected_paths {
                    match self.create_entry(path) {
                        Some(triple) => replies.push(triple),
                        None => { let _ = sender.send(Err(FileManagerThreadError::InvalidSelection)); }
                    };
                }

                let _ = sender.send(Ok(replies));
            }
            None => {
                let _ = sender.send(Err(FileManagerThreadError::UserCancelled));
                return;
            }
        }
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
