/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_loader::load_blob;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use mime_classifier::MimeClassifier;
use mime_guess::guess_mime_type_opt;
use net_traits::blob_url_store::{BlobURLStoreEntry, BlobURLStoreError, parse_blob_url};
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerResult, FilterPattern, FileOrigin};
use net_traits::filemanager_thread::{SelectedFile, RelativePos, FileManagerThreadError, SelectedFileId};
use net_traits::{LoadConsumer, LoadData, NetworkError};
use resource_thread::send_error;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::sync::atomic::{self, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use tinyfiledialogs;
use url::Url;
use util::prefs::PREFS;
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

struct FileStoreEntry {
    /// Origin of the entry's "creator"
    origin: FileOrigin,
    /// Backend implementation
    file_impl: FileImpl,
    /// Reference counting
    refs: AtomicUsize,
}

/// File backend implementation
#[derive(Clone)]
enum FileImpl {
    PathOnly(PathBuf),
    Memory(BlobURLStoreEntry),
    Sliced(Uuid, RelativePos),
}

struct FileManager<UI: 'static + UIProvider> {
    receiver: IpcReceiver<FileManagerThreadMsg>,
    store: Arc<FileManagerStore<UI>>,
    classifier: Arc<MimeClassifier>,
}

impl<UI: 'static + UIProvider> FileManager<UI> {
    fn new(recv: IpcReceiver<FileManagerThreadMsg>, ui: &'static UI) -> FileManager<UI> {
        FileManager {
            receiver: recv,
            store: Arc::new(FileManagerStore::new(ui)),
            classifier: Arc::new(MimeClassifier::new()),
        }
    }

    /// Start the file manager event loop
    fn start(&mut self) {
        loop {
            let store = self.store.clone();
            match self.receiver.recv().unwrap() {
                FileManagerThreadMsg::SelectFile(filter, sender, origin, opt_test_path) => {
                    spawn_named("select file".to_owned(), move || {
                        store.select_file(filter, sender, origin, opt_test_path);
                    });
                }
                FileManagerThreadMsg::SelectFiles(filter, sender, origin, opt_test_paths) => {
                    spawn_named("select files".to_owned(), move || {
                        store.select_files(filter, sender, origin, opt_test_paths);
                    })
                }
                FileManagerThreadMsg::ReadFile(sender, id, origin) => {
                    spawn_named("read file".to_owned(), move || {
                        match store.try_read_file(id, origin) {
                            Ok(buffer) => { let _ = sender.send(Ok(buffer)); }
                            Err(_) => { let _ = sender.send(Err(FileManagerThreadError::ReadFileError)); }
                        }
                    })
                }
                FileManagerThreadMsg::TransferMemory(entry, sender, origin) => {
                    spawn_named("transfer memory".to_owned(), move || {
                        store.transfer_memory(entry, sender, origin);
                    })
                }
                FileManagerThreadMsg::AddSlicedEntry(id, rel_pos, sender, origin) =>{
                    spawn_named("add sliced entry".to_owned(), move || {
                        store.add_sliced_entry(id, rel_pos, sender, origin);
                    })
                }
                FileManagerThreadMsg::LoadBlob(load_data, consumer) => {
                    match parse_blob_url(&load_data.url.clone()) {
                        None => {
                            let e = format!("Invalid blob URL format {:?}", load_data.url);
                            let format_err = NetworkError::Internal(e);
                            send_error(load_data.url.clone(), format_err, consumer);
                        }
                        Some((id, _fragment)) => {
                            self.process_request(load_data, consumer, RelativePos::full_range(), id);
                        }
                    }
                },
                FileManagerThreadMsg::DecRef(id, origin, sender) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        spawn_named("dec ref".to_owned(), move || {
                            let _ = sender.send(store.dec_ref(&id, &origin));
                        })
                    }
                }
                FileManagerThreadMsg::IncRef(id, origin) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        spawn_named("inc ref".to_owned(), move || {
                            let _ = store.inc_ref(&id, &origin);
                        })
                    }
                }
                FileManagerThreadMsg::Exit => break,
            };
        }
    }

    fn process_request(&self, load_data: LoadData, consumer: LoadConsumer,
                       rel_pos: RelativePos, id: Uuid) {
        let origin_in = load_data.url.origin().unicode_serialization();
        match self.store.get_impl(&id, &origin_in) {
            Ok(file_impl) => {
                match file_impl {
                    FileImpl::Memory(buffered) => {
                        let classifier = self.classifier.clone();
                        spawn_named("load blob".to_owned(), move ||
                            load_blob(load_data, consumer, classifier,
                                      None, rel_pos, buffered));
                    }
                    FileImpl::PathOnly(filepath) => {
                        let opt_filename = filepath.file_name()
                                                   .and_then(|osstr| osstr.to_str())
                                                   .map(|s| s.to_string());

                        let mut bytes = vec![];
                        let mut handler = File::open(&filepath).unwrap();
                        let mime = guess_mime_type_opt(filepath);
                        let size = handler.read_to_end(&mut bytes).unwrap();

                        let entry = BlobURLStoreEntry {
                            type_string: match mime {
                                Some(x) => format!("{}", x),
                                None    => "".to_string(),
                            },
                            size: size as u64,
                            bytes: bytes,
                        };
                        let classifier = self.classifier.clone();
                        spawn_named("load blob".to_owned(), move ||
                            load_blob(load_data, consumer, classifier,
                                      opt_filename, rel_pos, entry));
                    },
                    FileImpl::Sliced(id, rel_pos) => {
                        self.process_request(load_data, consumer, rel_pos, id);
                    }
                }
            }
            Err(e) => {
                send_error(load_data.url.clone(), NetworkError::Internal(format!("{:?}", e)), consumer);
            }
        }
    }
}

struct FileManagerStore<UI: 'static + UIProvider> {
    entries: RwLock<HashMap<Uuid, FileStoreEntry>>,
    ui: &'static UI,
}

impl <UI: 'static + UIProvider> FileManagerStore<UI> {
    fn new(ui: &'static UI) -> Self {
        FileManagerStore {
            entries: RwLock::new(HashMap::new()),
            ui: ui,
        }
    }

    /// Copy out the file backend implementation content
    fn get_impl(&self, id: &Uuid, origin_in: &FileOrigin) -> Result<FileImpl, BlobURLStoreError> {
        match self.entries.read().unwrap().get(id) {
            Some(ref e) => {
                if *origin_in != *e.origin {
                    Err(BlobURLStoreError::InvalidOrigin)
                } else {
                    Ok(e.file_impl.clone())
                }
            }
            None => Err(BlobURLStoreError::InvalidFileID),
        }
    }

    fn insert(&self, id: Uuid, entry: FileStoreEntry) {
        self.entries.write().unwrap().insert(id, entry);
    }

    fn remove(&self, id: &Uuid) {
        self.entries.write().unwrap().remove(id);
    }

    fn inc_ref(&self, id: &Uuid, origin_in: &FileOrigin) -> Result<(), BlobURLStoreError>{
        match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if entry.origin == *origin_in {
                    entry.refs.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                } else {
                    Err(BlobURLStoreError::InvalidOrigin)
                }
            }
            None => Err(BlobURLStoreError::InvalidFileID),
        }
    }

    fn add_sliced_entry(&self, parent_id: SelectedFileId, rel_pos: RelativePos,
                        sender: IpcSender<Result<SelectedFileId, BlobURLStoreError>>,
                        origin_in: FileOrigin) {
        if let Ok(parent_id) = Uuid::parse_str(&parent_id.0) {
            match self.inc_ref(&parent_id, &origin_in) {
                Ok(_) => {
                    let new_id = Uuid::new_v4();
                    self.insert(new_id, FileStoreEntry {
                        origin: origin_in,
                        file_impl: FileImpl::Sliced(parent_id, rel_pos),
                        refs: AtomicUsize::new(1),
                    });

                    let _ = sender.send(Ok(SelectedFileId(new_id.simple().to_string())));
                }
                Err(e) => {
                    let _ = sender.send(Err(e));
                }
            }
        } else {
            let _ = sender.send(Err(BlobURLStoreError::InvalidFileID));
        }
    }

    fn select_file(&self, patterns: Vec<FilterPattern>,
                   sender: IpcSender<FileManagerResult<SelectedFile>>,
                   origin: FileOrigin, opt_test_path: Option<String>) {
        // Check if the select_files preference is enabled
        // to ensure process-level security against compromised script;
        // Then try applying opt_test_path directly for testing convenience
        let opt_s = if select_files_pref_enabled() {
            opt_test_path
        } else {
            self.ui.open_file_dialog("", patterns)
        };

        match opt_s {
            Some(s) => {
                let selected_path = Path::new(&s);

                match self.create_entry(selected_path, &origin) {
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

    fn select_files(&self, patterns: Vec<FilterPattern>,
                    sender: IpcSender<FileManagerResult<Vec<SelectedFile>>>,
                    origin: FileOrigin, opt_test_paths: Option<Vec<String>>) {
        // Check if the select_files preference is enabled
        // to ensure process-level security against compromised script;
        // Then try applying opt_test_paths directly for testing convenience
        let opt_v = if select_files_pref_enabled() {
            opt_test_paths
        } else {
            self.ui.open_file_dialog_multi("", patterns)
        };

        match opt_v {
            Some(v) => {
                let mut selected_paths = vec![];

                for s in &v {
                    selected_paths.push(Path::new(s));
                }

                let mut replies = vec![];

                for path in selected_paths {
                    match self.create_entry(path, &origin) {
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

    fn create_entry(&self, file_path: &Path, origin: &str) -> Option<SelectedFile> {
        match File::open(file_path) {
            Ok(handler) => {
                let id = Uuid::new_v4();
                let file_impl = FileImpl::PathOnly(file_path.to_path_buf());

                self.insert(id, FileStoreEntry {
                    origin: origin.to_string(),
                    file_impl: file_impl,
                    refs: AtomicUsize::new(1),
                });

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

    fn try_read_file(&self, id: SelectedFileId, origin_in: FileOrigin) -> Result<Vec<u8>, BlobURLStoreError> {
        let id = try!(Uuid::parse_str(&id.0).map_err(|_| BlobURLStoreError::InvalidFileID));

        match self.get_impl(&id, &origin_in) {
            Ok(file_impl) => {
                match file_impl {
                    FileImpl::PathOnly(filepath) => {
                        let mut buffer = vec![];
                        let mut handler = try!(File::open(filepath)
                                              .map_err(|_| BlobURLStoreError::InvalidEntry));
                        try!(handler.read_to_end(&mut buffer)
                            .map_err(|_| BlobURLStoreError::External));
                        Ok(buffer)
                    },
                    FileImpl::Memory(buffered) => {
                        Ok(buffered.bytes)
                    },
                    FileImpl::Sliced(id, rel_pos) => {
                        self.try_read_file(SelectedFileId(id.simple().to_string()), origin_in)
                            .map(|bytes| bytes.index(rel_pos.to_abs_range(bytes.len())).to_vec())
                    }
                }
            },
            Err(e) => Err(e),
        }
    }

    fn dec_ref(&self, id: &Uuid, origin_in: &FileOrigin) -> Result<(), BlobURLStoreError> {
        let (is_last_ref, opt_parent_id) = match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if *entry.origin == *origin_in {
                    let old_refs = entry.refs.fetch_sub(1, Ordering::Release);

                    if old_refs > 1 {
                        (false, None)
                    } else {
                        if let FileImpl::Sliced(ref parent_id, _) = entry.file_impl {
                            // if it has a reference to parent id, dec_ref on parent later
                            (true, Some(parent_id.clone()))
                        } else {
                            (true, None)
                        }
                    }
                } else { // Invalid origin
                    return Err(BlobURLStoreError::InvalidOrigin);
                }
            }
            None => return Err(BlobURLStoreError::InvalidFileID),
        };

        if is_last_ref {
            atomic::fence(Ordering::Acquire);
            self.remove(id);

            if let Some(parent_id) = opt_parent_id {
                return self.dec_ref(&parent_id, origin_in);
            }
        }

        Ok(())
    }

    fn transfer_memory(&self, entry: BlobURLStoreEntry,
                       sender: IpcSender<Result<SelectedFileId, BlobURLStoreError>>, origin: FileOrigin) {
        match Url::parse(&origin) { // parse to check sanity
            Ok(_) => {
                let id = Uuid::new_v4();
                self.insert(id, FileStoreEntry {
                    origin: origin.clone(),
                    file_impl: FileImpl::Memory(entry),
                    refs: AtomicUsize::new(1),
                });

                let _ = sender.send(Ok(SelectedFileId(id.simple().to_string())));
            }
            Err(_) => {
                let _ = sender.send(Err(BlobURLStoreError::InvalidOrigin));
            }
        }
    }
}


fn select_files_pref_enabled() -> bool {
    PREFS.get("dom.testing.htmlinputelement.select_files.enabled")
         .as_boolean().unwrap_or(false)
}
