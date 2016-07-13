/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_loader::load_blob;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use mime_classifier::MimeClassifier;
use mime_guess::guess_mime_type_opt;
use net_traits::blob_url_store::{BlobBuf, BlobURLStoreError, parse_blob_url};
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerResult, FilterPattern, FileOrigin};
use net_traits::filemanager_thread::{SelectedFile, RelativePos, FileManagerThreadError, SelectedFileId};
use net_traits::{LoadConsumer, LoadData, NetworkError};
use resource_thread::send_error;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::sync::atomic::{self, AtomicUsize, AtomicBool, Ordering};
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

/// Trait that provider of file-dialog UI should implement.
/// It will be used to initialize a generic FileManager.
/// For example, we can choose a dummy UI for testing purpose.
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

/// FileManagerStore's entry
struct FileStoreEntry {
    /// Origin of the entry's "creator"
    origin: FileOrigin,
    /// Backend implementation
    file_impl: FileImpl,
    /// Number of `FileImpl::Sliced` entries in `FileManagerStore`
    /// that has a reference (FileID) to this entry
    refs: AtomicUsize,
    /// UUIDs only become valid blob URIs when explicitly requested
    /// by the user with createObjectURL. Validity can be revoked as well.
    /// (The UUID is the one that maps to this entry in `FileManagerStore`)
    is_valid_url: AtomicBool
}

#[derive(Clone)]
struct FileMetaData {
    path: PathBuf,
    /// Modified time in UNIX Epoch format
    modified: u64,
    size: u64,
}

/// File backend implementation
#[derive(Clone)]
enum FileImpl {
    /// Metadata of on-disk file
    MetaDataOnly(FileMetaData),
    /// In-memory Blob buffer object
    Memory(BlobBuf),
    /// A reference to parent entry in `FileManagerStore`,
    /// representing a sliced version of the parent entry data
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
                            Err(e) => {
                                let _ = sender.send(Err(FileManagerThreadError::BlobURLStoreError(e)));
                            }
                        }
                    })
                }
                FileManagerThreadMsg::PromoteMemory(blob_buf, sender, origin) => {
                    spawn_named("transfer memory".to_owned(), move || {
                        store.promote_memory(blob_buf, sender, origin);
                    })
                }
                FileManagerThreadMsg::AddSlicedURLEntry(id, rel_pos, sender, origin) =>{
                    spawn_named("add sliced URL entry".to_owned(), move || {
                        store.add_sliced_url_entry(id, rel_pos, sender, origin);
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
                            self.process_request(load_data, consumer, id);
                        }
                    }
                },
                FileManagerThreadMsg::RevokeBlobURL(id, origin, sender) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        spawn_named("revoke blob url".to_owned(), move || {
                            // Since it is revocation, unset_url_validity is true
                            let _ = sender.send(store.dec_ref(&id, &origin, true));
                        })
                    } else {
                        let _ = sender.send(Err(BlobURLStoreError::InvalidFileID));
                    }
                }
                FileManagerThreadMsg::DecRef(id, origin, sender) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        spawn_named("dec ref".to_owned(), move || {
                            // Since it is simple DecRef (possibly caused by close/drop),
                            // unset_url_validity is false
                            let _ = sender.send(store.dec_ref(&id, &origin, false));
                        })
                    } else {
                        let _ = sender.send(Err(BlobURLStoreError::InvalidFileID));
                    }
                }
                FileManagerThreadMsg::IncRef(id, origin) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        spawn_named("inc ref".to_owned(), move || {
                            let _ = store.inc_ref(&id, &origin);
                        })
                    }
                }
                FileManagerThreadMsg::ActivateBlobURL(id, sender, origin) => {
                    if let Ok(id) = Uuid::parse_str(&id.0) {
                        spawn_named("activate blob url".to_owned(), move || {
                            let _ = sender.send(store.activate_blob_url(&id, &origin));
                        });
                    } else {
                        let _ = sender.send(Err(BlobURLStoreError::InvalidFileID));
                    }
                }
                FileManagerThreadMsg::Exit => break,
            };
        }
    }

    fn process_request(&self, load_data: LoadData, consumer: LoadConsumer, id: Uuid) {
        let origin_in = load_data.url.origin().unicode_serialization();
        // check_url_validity is true since content is requested by this URL
        match self.store.get_blob_buf(&id, &origin_in, RelativePos::full_range(), true) {
            Ok(blob_buf) => {
                let classifier = self.classifier.clone();
                spawn_named("load blob".to_owned(), move || load_blob(load_data, consumer, classifier, blob_buf));
            }
            Err(e) => send_error(load_data.url.clone(), NetworkError::Internal(format!("{:?}", e)), consumer),
        }
    }
}

/// File manager's data store. It maintains a thread-safe mapping
/// from FileID to FileStoreEntry which might have different backend implementation.
/// Access to the content is encapsulated as methods of this struct.
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
    fn get_impl(&self, id: &Uuid, origin_in: &FileOrigin,
                check_url_validity: bool) -> Result<FileImpl, BlobURLStoreError> {
        match self.entries.read().unwrap().get(id) {
            Some(ref entry) => {
                if *origin_in != *entry.origin {
                    Err(BlobURLStoreError::InvalidOrigin)
                } else {
                    if check_url_validity && !entry.is_valid_url.load(Ordering::Acquire) {
                        Err(BlobURLStoreError::InvalidFileID)
                    } else {
                        Ok(entry.file_impl.clone())
                    }
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

    fn add_sliced_url_entry(&self, parent_id: SelectedFileId, rel_pos: RelativePos,
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
                        // Valid here since AddSlicedURLEntry implies URL creation
                        is_valid_url: AtomicBool::new(true),
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
                let result = self.create_entry(selected_path, &origin);
                let _ = sender.send(result);
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
                        Ok(triple) => replies.push(triple),
                        Err(e) => {
                            let _ = sender.send(Err(e));
                            return;
                        }
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

    fn create_entry(&self, file_path: &Path, origin: &str) -> Result<SelectedFile, FileManagerThreadError> {
        use net_traits::filemanager_thread::FileManagerThreadError::FileSystemError;

        let handler = try!(File::open(file_path).map_err(|e| FileSystemError(e.to_string())));
        let metadata = try!(handler.metadata().map_err(|e| FileSystemError(e.to_string())));
        let modified = try!(metadata.modified().map_err(|e| FileSystemError(e.to_string())));
        let elapsed = try!(modified.elapsed().map_err(|e| FileSystemError(e.to_string())));
        // Unix Epoch: https://doc.servo.org/std/time/constant.UNIX_EPOCH.html
        let modified_epoch = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000;
        let file_size = metadata.len();
        let file_name = try!(file_path.file_name().ok_or(FileSystemError("Invalid filepath".to_string())));

        let file_impl = FileImpl::MetaDataOnly(FileMetaData {
            path: file_path.to_path_buf(),
            modified: modified_epoch,
            size: file_size,
        });

        let id = Uuid::new_v4();

        self.insert(id, FileStoreEntry {
            origin: origin.to_string(),
            file_impl: file_impl,
            refs: AtomicUsize::new(1),
            // Invalid here since create_entry is called by file selection
            is_valid_url: AtomicBool::new(false),
        });

        let filename_path = Path::new(file_name);
        let type_string = match guess_mime_type_opt(filename_path) {
            Some(x) => format!("{}", x),
            None    => "".to_string(),
        };

        Ok(SelectedFile {
            id: SelectedFileId(id.simple().to_string()),
            filename: filename_path.to_path_buf(),
            modified: modified_epoch,
            size: file_size,
            type_string: type_string,
        })
    }

    fn get_blob_buf(&self, id: &Uuid, origin_in: &FileOrigin, rel_pos: RelativePos,
                    check_url_validity: bool) -> Result<BlobBuf, BlobURLStoreError> {
        let file_impl = try!(self.get_impl(id, origin_in, check_url_validity));
        match file_impl {
            FileImpl::Memory(buf) => {
                let range = rel_pos.to_abs_range(buf.size as usize);
                Ok(BlobBuf {
                    filename: None,
                    type_string: buf.type_string,
                    size: range.len() as u64,
                    bytes: buf.bytes.index(range).to_vec(),
                })
            }
            FileImpl::MetaDataOnly(metadata) => {
                /* XXX: Snapshot state check (optional) https://w3c.github.io/FileAPI/#snapshot-state.
                        Concretely, here we create another handler, and this handler might not
                        has the same underlying file state (meta-info plus content) as the time
                        create_entry is called.
                */

                let opt_filename = metadata.path.file_name()
                                           .and_then(|osstr| osstr.to_str())
                                           .map(|s| s.to_string());

                let mime = guess_mime_type_opt(metadata.path.clone());
                let range = rel_pos.to_abs_range(metadata.size as usize);

                let mut handler = try!(File::open(&metadata.path)
                                      .map_err(|e| BlobURLStoreError::External(e.to_string())));
                let seeked_start = try!(handler.seek(SeekFrom::Start(range.start as u64))
                                       .map_err(|e| BlobURLStoreError::External(e.to_string())));

                if seeked_start == (range.start as u64) {
                    let mut bytes = vec![0; range.len()];
                    try!(handler.read_exact(&mut bytes)
                        .map_err(|e| BlobURLStoreError::External(e.to_string())));

                    Ok(BlobBuf {
                        filename: opt_filename,
                        type_string: match mime {
                            Some(x) => format!("{}", x),
                            None    => "".to_string(),
                        },
                        size: range.len() as u64,
                        bytes: bytes,
                    })
                } else {
                    Err(BlobURLStoreError::InvalidEntry)
                }
            }
            FileImpl::Sliced(parent_id, inner_rel_pos) => {
                // Next time we don't need to check validity since
                // we have already done that for requesting URL if necessary
                self.get_blob_buf(&parent_id, origin_in, rel_pos.slice_inner(&inner_rel_pos), false)
            }
        }
    }

    fn try_read_file(&self, id: SelectedFileId, origin_in: FileOrigin) -> Result<Vec<u8>, BlobURLStoreError> {
        let id = try!(Uuid::parse_str(&id.0).map_err(|_| BlobURLStoreError::InvalidFileID));

        // No need to check URL validity in reading a file by FileReader
        let blob_buf = try!(self.get_blob_buf(&id, &origin_in, RelativePos::full_range(), false));

        Ok(blob_buf.bytes)
    }

    fn dec_ref(&self, id: &Uuid, origin_in: &FileOrigin,
               unset_url_validity: bool) -> Result<(), BlobURLStoreError> {
        let (is_last_ref, opt_parent_id) = match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if *entry.origin == *origin_in {
                    let old_refs = entry.refs.fetch_sub(1, Ordering::Release);

                    if old_refs > 1 {
                        if unset_url_validity {
                            entry.is_valid_url.store(false, Ordering::Release);
                        }

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
                // unset_url_validity for parent is false since we only need
                // to unset the initial requesting URL
                return self.dec_ref(&parent_id, origin_in, false);
            }
        }

        Ok(())
    }

    fn promote_memory(&self, blob_buf: BlobBuf,
                       sender: IpcSender<Result<SelectedFileId, BlobURLStoreError>>, origin: FileOrigin) {
        match Url::parse(&origin) { // parse to check sanity
            Ok(_) => {
                let id = Uuid::new_v4();
                self.insert(id, FileStoreEntry {
                    origin: origin.clone(),
                    file_impl: FileImpl::Memory(blob_buf),
                    refs: AtomicUsize::new(1),
                    // Valid here since PromoteMemory implies URL creation
                    is_valid_url: AtomicBool::new(true),
                });

                let _ = sender.send(Ok(SelectedFileId(id.simple().to_string())));
            }
            Err(_) => {
                let _ = sender.send(Err(BlobURLStoreError::InvalidOrigin));
            }
        }
    }

    fn activate_blob_url(&self, id: &Uuid, origin_in: &FileOrigin) -> Result<(), BlobURLStoreError> {
        match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if *entry.origin == *origin_in {
                    entry.is_valid_url.store(true, Ordering::Release);
                    Ok(())
                } else {
                    Err(BlobURLStoreError::InvalidOrigin)
                }
            }
            None => Err(BlobURLStoreError::InvalidFileID)
        }
    }
}


fn select_files_pref_enabled() -> bool {
    PREFS.get("dom.testing.htmlinputelement.select_files.enabled")
         .as_boolean().unwrap_or(false)
}
