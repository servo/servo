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
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use tinyfiledialogs;
use url::Url;
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
    refs: Cell<usize>,
}

/// File backend implementation
enum FileImpl {
    PathOnly(PathBuf),
    Memory(BlobURLStoreEntry),
    Sliced(Uuid, RelativePos),
}

struct FileManager<UI: 'static + UIProvider> {
    receiver: IpcReceiver<FileManagerThreadMsg>,
    store: HashMap<Uuid, FileStoreEntry>,
    classifier: Arc<MimeClassifier>,
    ui: &'static UI,
}

impl<UI: 'static + UIProvider> FileManager<UI> {
    fn new(recv: IpcReceiver<FileManagerThreadMsg>, ui: &'static UI) -> FileManager<UI> {
        FileManager {
            receiver: recv,
            store: HashMap::new(),
            classifier: Arc::new(MimeClassifier::new()),
            ui: ui,
        }
    }

    /// Start the file manager event loop
    fn start(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                FileManagerThreadMsg::SelectFile(filter, sender, origin) => self.select_file(filter, sender, origin),
                FileManagerThreadMsg::SelectFiles(filter, sender, origin) => self.select_files(filter, sender, origin),
                FileManagerThreadMsg::ReadFile(sender, id, origin) => {
                    match self.try_read_file(id, origin) {
                        Ok(buffer) => { let _ = sender.send(Ok(buffer)); }
                        Err(_) => { let _ = sender.send(Err(FileManagerThreadError::ReadFileError)); }
                    }
                }
                FileManagerThreadMsg::TransferMemory(entry, sender, origin) =>
                    self.transfer_memory(entry, sender, origin),
                FileManagerThreadMsg::AddSlicedEntry(id, rel_pos, sender, origin) =>
                    self.add_sliced_entry(id, rel_pos, sender, origin),
                FileManagerThreadMsg::LoadBlob(load_data, consumer) => {
                    match parse_blob_url(&load_data.url) {
                        None => {
                            let e = format!("Invalid blob URL format {:?}", load_data.url);
                            let format_err = NetworkError::Internal(e);
                            send_error(load_data.url.clone(), format_err, consumer);
                        }
                        Some((id, _fragment)) => {
                            self.process_request(&load_data, consumer, &RelativePos::full_range(), &id);
                        }
                    }
                },
                FileManagerThreadMsg::DecRef(id, origin) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        self.dec_ref(id, origin);
                    }
                }
                FileManagerThreadMsg::IncRef(id, origin) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        self.inc_ref(id, origin);
                    }
                }
                FileManagerThreadMsg::Exit => break,
            };
        }
    }

    fn inc_ref(&mut self, id: Uuid, origin_in: FileOrigin) {
        match self.store.get(&id) {
            Some(entry) => {
                if entry.origin == origin_in {
                    entry.refs.set(entry.refs.get() + 1);
                }
            }
            None => return, // Invalid UUID
        }
    }

    fn add_sliced_entry(&mut self, id: SelectedFileId, rel_pos: RelativePos,
                        sender: IpcSender<Result<SelectedFileId, BlobURLStoreError>>,
                        origin_in: FileOrigin) {
        if let Ok(id) = Uuid::parse_str(&id.0) {
            match self.store.get(&id) {
                Some(entry) => {
                    if entry.origin == origin_in {
                        // inc_ref on parent entry
                        entry.refs.set(entry.refs.get() + 1);
                    } else {
                        let _ = sender.send(Err(BlobURLStoreError::InvalidOrigin));
                        return;
                    }
                },
                None => {
                    let _ = sender.send(Err(BlobURLStoreError::InvalidFileID));
                    return;
                }
            };

            let new_id = Uuid::new_v4();
            self.store.insert(new_id, FileStoreEntry {
                origin: origin_in.clone(),
                file_impl: FileImpl::Sliced(id, rel_pos),
                refs: Cell::new(1),
            });

            let _ = sender.send(Ok(SelectedFileId(new_id.simple().to_string())));
        } else {
            let _ = sender.send(Err(BlobURLStoreError::InvalidFileID));
        }
    }

    fn select_file(&mut self, patterns: Vec<FilterPattern>,
                   sender: IpcSender<FileManagerResult<SelectedFile>>,
                   origin: FileOrigin) {
        match self.ui.open_file_dialog("", patterns) {
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

    fn select_files(&mut self, patterns: Vec<FilterPattern>,
                    sender: IpcSender<FileManagerResult<Vec<SelectedFile>>>,
                    origin: FileOrigin) {
        match self.ui.open_file_dialog_multi("", patterns) {
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

    fn create_entry(&mut self, file_path: &Path, origin: &str) -> Option<SelectedFile> {
        match File::open(file_path) {
            Ok(handler) => {
                let id = Uuid::new_v4();
                let file_impl = FileImpl::PathOnly(file_path.to_path_buf());

                self.store.insert(id, FileStoreEntry {
                    origin: origin.to_string(),
                    file_impl: file_impl,
                    refs: Cell::new(1),
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

    fn try_read_file(&self, id: SelectedFileId, origin_in: String) -> Result<Vec<u8>, ()> {
        let id = try!(Uuid::parse_str(&id.0).map_err(|_| ()));

        match self.store.get(&id) {
            Some(entry) => {
                match entry.file_impl {
                    FileImpl::PathOnly(ref filepath) => {
                        if *entry.origin == origin_in {
                            let mut buffer = vec![];
                            let mut handler = try!(File::open(filepath).map_err(|_| ()));
                            try!(handler.read_to_end(&mut buffer).map_err(|_| ()));
                            Ok(buffer)
                        } else {
                            Err(())
                        }
                    },
                    FileImpl::Memory(ref buffered) => {
                        Ok(buffered.bytes.clone())
                    },
                    FileImpl::Sliced(ref id, ref _rel_pos) => {
                        self.try_read_file(SelectedFileId(id.simple().to_string()), origin_in)
                    }
                }
            },
            None => Err(()),
        }
    }

    fn dec_ref(&mut self, id: Uuid, origin_in: FileOrigin) {
        let (is_last_ref, opt_parent_id) = match self.store.get(&id) {
            Some(entry) => {
                if *entry.origin == origin_in {
                    let r = entry.refs.get();

                    if r > 1 {
                        entry.refs.set(r - 1);
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
                    return;
                }
            }
            None => return, // Invalid UUID
        };

        if is_last_ref {
            self.store.remove(&id);

            if let Some(parent_id) = opt_parent_id {
                self.dec_ref(parent_id, origin_in);
            }
        }
    }

    fn process_request(&self, load_data: &LoadData, consumer: LoadConsumer,
                       rel_pos: &RelativePos, id: &Uuid) {
        let origin_in = load_data.url.origin().unicode_serialization();
        match self.store.get(id) {
            Some(entry) => {
                match entry.file_impl {
                    FileImpl::Memory(ref buffered) => {
                        if *entry.origin == origin_in {
                            load_blob(&load_data, consumer, self.classifier.clone(),
                                      None, rel_pos, buffered);
                        } else {
                            let e = format!("Invalid blob URL origin {:?}", origin_in);
                            send_error(load_data.url.clone(), NetworkError::Internal(e), consumer);
                        }
                    },
                    FileImpl::PathOnly(ref filepath) => {
                        let opt_filename = filepath.file_name()
                                               .and_then(|osstr| osstr.to_str())
                                               .map(|s| s.to_string());

                        if *entry.origin == origin_in {
                            let mut bytes = vec![];
                            let mut handler = File::open(filepath).unwrap();
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

                            load_blob(&load_data, consumer, self.classifier.clone(),
                                      opt_filename, rel_pos, &entry);
                        } else {
                            let e = format!("Invalid blob URL origin {:?}", origin_in);
                            send_error(load_data.url.clone(), NetworkError::Internal(e), consumer);
                        }
                    },
                    FileImpl::Sliced(ref id, ref rel_pos) => {
                        self.process_request(load_data, consumer, rel_pos, id);
                    }
                }
            }
            _ => {
                let e = format!("Invalid blob URL key {:?}", id.simple().to_string());
                send_error(load_data.url.clone(), NetworkError::Internal(e), consumer);
            }
        }
    }

    fn transfer_memory(&mut self, entry: BlobURLStoreEntry,
                       sender: IpcSender<Result<SelectedFileId, BlobURLStoreError>>,
                       origin: FileOrigin) {
        match Url::parse(&origin) { // parse to check sanity
            Ok(_) => {
                let id = Uuid::new_v4();
                self.store.insert(id, FileStoreEntry {
                    origin: origin.clone(),
                    file_impl: FileImpl::Memory(entry),
                    refs: Cell::new(1),
                });

                let _ = sender.send(Ok(SelectedFileId(id.simple().to_string())));
            }
            Err(_) => {
                let _ = sender.send(Err(BlobURLStoreError::InvalidOrigin));
            }
        }
    }
}
