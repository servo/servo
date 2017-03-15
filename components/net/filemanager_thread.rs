/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use mime_guess::guess_mime_type_opt;
use net_traits::blob_url_store::{BlobBuf, BlobURLStoreError};
use net_traits::filemanager_thread::{FileManagerResult, FileManagerThreadMsg, FileOrigin, FilterPattern};
use net_traits::filemanager_thread::{FileManagerThreadError, ReadFileProgress, RelativePos, SelectedFile};
use servo_config::opts;
use servo_config::prefs::PREFS;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{self, AtomicBool, AtomicUsize, Ordering};
use std::thread;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
use tinyfiledialogs;
use url::Url;
use uuid::Uuid;

/// The provider of file-dialog UI should implement this trait.
/// It will be used to initialize a generic FileManager.
/// For example, we can choose a dummy UI for testing purpose.
pub trait UIProvider where Self: Sync {
    fn open_file_dialog(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<String>;

    fn open_file_dialog_multi(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<Vec<String>>;
}

pub struct TFDProvider;

impl UIProvider for TFDProvider {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    fn open_file_dialog(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<String> {
        if opts::get().headless {
            return None;
        }

        let mut filter = vec![];
        for p in patterns {
            let s = "*.".to_string() + &p.0;
            filter.push(s)
        }

        let filter_ref = &(filter.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[..]);

        let filter_opt = if filter.len() > 0 { Some((filter_ref, "")) } else { None };

        tinyfiledialogs::open_file_dialog("Pick a file", path, filter_opt)
    }

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    fn open_file_dialog_multi(&self, path: &str, patterns: Vec<FilterPattern>) -> Option<Vec<String>> {
        if opts::get().headless {
            return None;
        }

        let mut filter = vec![];
        for p in patterns {
            let s = "*.".to_string() + &p.0;
            filter.push(s)
        }

        let filter_ref = &(filter.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[..]);

        let filter_opt = if filter.len() > 0 { Some((filter_ref, "")) } else { None };

        tinyfiledialogs::open_file_dialog_multi("Pick files", path, filter_opt)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn open_file_dialog(&self, _path: &str, _patterns: Vec<FilterPattern>) -> Option<String> {
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn open_file_dialog_multi(&self, _path: &str, _patterns: Vec<FilterPattern>) -> Option<Vec<String>> {
        None
    }
}

/// FileManagerStore's entry
struct FileStoreEntry {
    /// Origin of the entry's "creator"
    origin: FileOrigin,
    /// Backend implementation
    file_impl: FileImpl,
    /// Number of FileID holders that the ID is used to
    /// index this entry in `FileManagerStore`.
    /// Reference holders include a FileStoreEntry or
    /// a script-side File-based Blob
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

#[derive(Clone)]
pub struct FileManager {
    store: Arc<FileManagerStore>,
}

impl FileManager {
    pub fn new() -> FileManager {
        FileManager {
            store: Arc::new(FileManagerStore::new()),
        }
    }

    pub fn read_file(&self,
                     sender: IpcSender<FileManagerResult<ReadFileProgress>>,
                     id: Uuid,
                     check_url_validity: bool,
                     origin: FileOrigin) {
        let store = self.store.clone();
        thread::Builder::new().name("read file".to_owned()).spawn(move || {
            if let Err(e) = store.try_read_file(&sender, id, check_url_validity,
                                                origin) {
                let _ = sender.send(Err(FileManagerThreadError::BlobURLStoreError(e)));
            }
        }).expect("Thread spawning failed");
    }

    pub fn promote_memory(&self,
                          blob_buf: BlobBuf,
                          set_valid: bool,
                          sender: IpcSender<Result<Uuid, BlobURLStoreError>>,
                          origin: FileOrigin) {
        let store = self.store.clone();
        thread::Builder::new().name("transfer memory".to_owned()).spawn(move || {
            store.promote_memory(blob_buf, set_valid, sender, origin);
        }).expect("Thread spawning failed");
    }

    /// Message handler
    pub fn handle<UI>(&self,
                      msg: FileManagerThreadMsg,
                      ui: &'static UI)
        where UI: UIProvider + 'static,
    {
        match msg {
            FileManagerThreadMsg::SelectFile(filter, sender, origin, opt_test_path) => {
                let store = self.store.clone();
                thread::Builder::new().name("select file".to_owned()).spawn(move || {
                    store.select_file(filter, sender, origin, opt_test_path, ui);
                }).expect("Thread spawning failed");
            }
            FileManagerThreadMsg::SelectFiles(filter, sender, origin, opt_test_paths) => {
                let store = self.store.clone();
                thread::Builder::new().name("select files".to_owned()).spawn(move || {
                    store.select_files(filter, sender, origin, opt_test_paths, ui);
                }).expect("Thread spawning failed");
            }
            FileManagerThreadMsg::ReadFile(sender, id, check_url_validity, origin) => {
                self.read_file(sender, id, check_url_validity, origin);
            }
            FileManagerThreadMsg::PromoteMemory(blob_buf, set_valid, sender, origin) => {
                self.promote_memory(blob_buf, set_valid, sender, origin);
            }
            FileManagerThreadMsg::AddSlicedURLEntry(id, rel_pos, sender, origin) =>{
                self.store.add_sliced_url_entry(id, rel_pos, sender, origin);
            }
            FileManagerThreadMsg::DecRef(id, origin, sender) => {
                let _ = sender.send(self.store.dec_ref(&id, &origin));
            }
            FileManagerThreadMsg::RevokeBlobURL(id, origin, sender) => {
                let _ = sender.send(self.store.set_blob_url_validity(false, &id, &origin));
            }
            FileManagerThreadMsg::ActivateBlobURL(id, sender, origin) => {
                let _ = sender.send(self.store.set_blob_url_validity(true, &id, &origin));
            }
        }
    }
}

/// File manager's data store. It maintains a thread-safe mapping
/// from FileID to FileStoreEntry which might have different backend implementation.
/// Access to the content is encapsulated as methods of this struct.
struct FileManagerStore {
    entries: RwLock<HashMap<Uuid, FileStoreEntry>>,
}

impl FileManagerStore {
    fn new() -> Self {
        FileManagerStore {
            entries: RwLock::new(HashMap::new()),
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
                    let is_valid = entry.is_valid_url.load(Ordering::Acquire);
                    if check_url_validity && !is_valid {
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

    fn add_sliced_url_entry(&self, parent_id: Uuid, rel_pos: RelativePos,
                            sender: IpcSender<Result<Uuid, BlobURLStoreError>>,
                            origin_in: FileOrigin) {
        match self.inc_ref(&parent_id, &origin_in) {
            Ok(_) => {
                let new_id = Uuid::new_v4();
                self.insert(new_id, FileStoreEntry {
                    origin: origin_in,
                    file_impl: FileImpl::Sliced(parent_id, rel_pos),
                    refs: AtomicUsize::new(1),
                    // Valid here since AddSlicedURLEntry implies URL creation
                    // from a BlobImpl::Sliced
                    is_valid_url: AtomicBool::new(true),
                });

                // We assume that the returned id will be held by BlobImpl::File
                let _ = sender.send(Ok(new_id));
            }
            Err(e) => {
                let _ = sender.send(Err(e));
            }
        }
    }

    fn select_file<UI>(&self,
                       patterns: Vec<FilterPattern>,
                       sender: IpcSender<FileManagerResult<SelectedFile>>,
                       origin: FileOrigin,
                       opt_test_path: Option<String>,
                       ui: &UI)
        where UI: UIProvider,
    {
        // Check if the select_files preference is enabled
        // to ensure process-level security against compromised script;
        // Then try applying opt_test_path directly for testing convenience
        let opt_s = if select_files_pref_enabled() {
            opt_test_path
        } else {
            ui.open_file_dialog("", patterns)
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

    fn select_files<UI>(&self,
                        patterns: Vec<FilterPattern>,
                        sender: IpcSender<FileManagerResult<Vec<SelectedFile>>>,
                        origin: FileOrigin,
                        opt_test_paths: Option<Vec<String>>,
                        ui: &UI)
        where UI: UIProvider,
    {
        // Check if the select_files preference is enabled
        // to ensure process-level security against compromised script;
        // Then try applying opt_test_paths directly for testing convenience
        let opt_v = if select_files_pref_enabled() {
            opt_test_paths
        } else {
            ui.open_file_dialog_multi("", patterns)
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

        let file = try!(File::open(file_path).map_err(|e| FileSystemError(e.to_string())));
        let metadata = try!(file.metadata().map_err(|e| FileSystemError(e.to_string())));
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
            id: id,
            filename: filename_path.to_path_buf(),
            modified: modified_epoch,
            size: file_size,
            type_string: type_string,
        })
    }

    fn get_blob_buf(&self, sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
                    id: &Uuid, origin_in: &FileOrigin, rel_pos: RelativePos,
                    check_url_validity: bool) -> Result<(), BlobURLStoreError> {
        let file_impl = try!(self.get_impl(id, origin_in, check_url_validity));
        match file_impl {
            FileImpl::Memory(buf) => {
                let range = rel_pos.to_abs_range(buf.size as usize);
                let buf = BlobBuf {
                    filename: None,
                    type_string: buf.type_string,
                    size: range.len() as u64,
                    bytes: buf.bytes.index(range).to_vec(),
                };

                let _ = sender.send(Ok(ReadFileProgress::Meta(buf)));
                let _ = sender.send(Ok(ReadFileProgress::EOF));

                Ok(())
            }
            FileImpl::MetaDataOnly(metadata) => {
                /* XXX: Snapshot state check (optional) https://w3c.github.io/FileAPI/#snapshot-state.
                        Concretely, here we create another file, and this file might not
                        has the same underlying file state (meta-info plus content) as the time
                        create_entry is called.
                */

                let opt_filename = metadata.path.file_name()
                                           .and_then(|osstr| osstr.to_str())
                                           .map(|s| s.to_string());

                let mime = guess_mime_type_opt(metadata.path.clone());
                let range = rel_pos.to_abs_range(metadata.size as usize);

                let mut file = try!(File::open(&metadata.path)
                                   .map_err(|e| BlobURLStoreError::External(e.to_string())));
                let seeked_start = try!(file.seek(SeekFrom::Start(range.start as u64))
                                       .map_err(|e| BlobURLStoreError::External(e.to_string())));

                if seeked_start == (range.start as u64) {
                    let type_string = match mime {
                        Some(x) => format!("{}", x),
                        None    => "".to_string(),
                    };

                    chunked_read(sender, &mut file, range.len(), opt_filename,
                                 type_string);
                    Ok(())
                } else {
                    Err(BlobURLStoreError::InvalidEntry)
                }
            }
            FileImpl::Sliced(parent_id, inner_rel_pos) => {
                // Next time we don't need to check validity since
                // we have already done that for requesting URL if necessary
                self.get_blob_buf(sender, &parent_id, origin_in,
                                  rel_pos.slice_inner(&inner_rel_pos), false)
            }
        }
    }

    // Convenient wrapper over get_blob_buf
    fn try_read_file(&self, sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
                     id: Uuid, check_url_validity: bool, origin_in: FileOrigin)
                     -> Result<(), BlobURLStoreError> {
        self.get_blob_buf(sender, &id, &origin_in, RelativePos::full_range(), check_url_validity)
    }

    fn dec_ref(&self, id: &Uuid, origin_in: &FileOrigin) -> Result<(), BlobURLStoreError> {
        let (do_remove, opt_parent_id) = match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if *entry.origin == *origin_in {
                    let old_refs = entry.refs.fetch_sub(1, Ordering::Release);

                    if old_refs > 1 {
                        // not the last reference, no need to touch parent
                        (false, None)
                    } else {
                        // last reference, and if it has a reference to parent id
                        // dec_ref on parent later if necessary
                        let is_valid = entry.is_valid_url.load(Ordering::Acquire);
                        if let FileImpl::Sliced(ref parent_id, _) = entry.file_impl {
                            (!is_valid, Some(parent_id.clone()))
                        } else {
                            (!is_valid, None)
                        }
                    }
                } else {
                    return Err(BlobURLStoreError::InvalidOrigin);
                }
            }
            None => return Err(BlobURLStoreError::InvalidFileID),
        };

        // Trigger removing if its last reference is gone and it is
        // not a part of a valid Blob URL
        if do_remove {
            atomic::fence(Ordering::Acquire);
            self.remove(id);

            if let Some(parent_id) = opt_parent_id {
                return self.dec_ref(&parent_id, origin_in);
            }
        }

        Ok(())
    }

    fn promote_memory(&self, blob_buf: BlobBuf, set_valid: bool,
                       sender: IpcSender<Result<Uuid, BlobURLStoreError>>, origin: FileOrigin) {
        match Url::parse(&origin) { // parse to check sanity
            Ok(_) => {
                let id = Uuid::new_v4();
                self.insert(id, FileStoreEntry {
                    origin: origin.clone(),
                    file_impl: FileImpl::Memory(blob_buf),
                    refs: AtomicUsize::new(1),
                    is_valid_url: AtomicBool::new(set_valid),
                });

                let _ = sender.send(Ok(id));
            }
            Err(_) => {
                let _ = sender.send(Err(BlobURLStoreError::InvalidOrigin));
            }
        }
    }

    fn set_blob_url_validity(&self, validity: bool, id: &Uuid,
                             origin_in: &FileOrigin) -> Result<(), BlobURLStoreError> {
        let (do_remove, opt_parent_id, res) = match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if *entry.origin == *origin_in {
                    entry.is_valid_url.store(validity, Ordering::Release);

                    if !validity {
                        // Check if it is the last possible reference
                        // since refs only accounts for blob id holders
                        // and store entry id holders
                        let zero_refs =  entry.refs.load(Ordering::Acquire) == 0;

                        if let FileImpl::Sliced(ref parent_id, _) = entry.file_impl {
                            (zero_refs, Some(parent_id.clone()), Ok(()))
                        } else {
                            (zero_refs, None, Ok(()))
                        }
                    } else {
                        (false, None, Ok(()))
                    }
                } else {
                    (false, None, Err(BlobURLStoreError::InvalidOrigin))
                }
            }
            None => (false, None, Err(BlobURLStoreError::InvalidFileID))
        };

        if do_remove {
            atomic::fence(Ordering::Acquire);
            self.remove(id);

            if let Some(parent_id) = opt_parent_id {
                return self.dec_ref(&parent_id, origin_in);
            }
        }
        res
    }
}

fn select_files_pref_enabled() -> bool {
    PREFS.get("dom.testing.htmlinputelement.select_files.enabled")
         .as_boolean().unwrap_or(false)
}

const CHUNK_SIZE: usize = 8192;

fn chunked_read(sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
                file: &mut File, size: usize, opt_filename: Option<String>,
                type_string: String) {
    // First chunk
    let mut buf = vec![0; CHUNK_SIZE];
    match file.read(&mut buf) {
        Ok(n) => {
            buf.truncate(n);
            let blob_buf = BlobBuf {
                filename: opt_filename,
                type_string: type_string,
                size: size as u64,
                bytes: buf,
            };
            let _ = sender.send(Ok(ReadFileProgress::Meta(blob_buf)));
        }
        Err(e) => {
            let _ = sender.send(Err(FileManagerThreadError::FileSystemError(e.to_string())));
            return;
        }
    }

    // Send the remaining chunks
    loop {
        let mut buf = vec![0; CHUNK_SIZE];
        match file.read(&mut buf) {
            Ok(0) => {
                let _ = sender.send(Ok(ReadFileProgress::EOF));
                return;
            }
            Ok(n) => {
                buf.truncate(n);
                let _ = sender.send(Ok(ReadFileProgress::Partial(buf)));
            }
            Err(e) => {
                let _ = sender.send(Err(FileManagerThreadError::FileSystemError(e.to_string())));
                return;
            }
        }
    }
}
