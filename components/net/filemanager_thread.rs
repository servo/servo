/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::fetch::methods::{CancellationListener, Data, RangeRequestBounds};
use embedder_traits::{EmbedderMsg, EmbedderProxy, FilterPattern};
use headers_ext::{ContentLength, ContentType, HeaderMap, HeaderMapExt};
use http::header::{self, HeaderValue};
use ipc_channel::ipc::{self, IpcSender};
use mime::{self, Mime};
use mime_guess::guess_mime_type_opt;
use net_traits::blob_url_store::{BlobBuf, BlobURLStoreError};
use net_traits::filemanager_thread::{FileManagerResult, FileManagerThreadMsg, FileOrigin};
use net_traits::filemanager_thread::{
    FileManagerThreadError, ReadFileProgress, RelativePos, SelectedFile,
};
use net_traits::http_percent_encode;
use net_traits::response::{Response, ResponseBody};
use servo_arc::Arc as ServoArc;
use servo_channel;
use servo_config::prefs::PREFS;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::mem;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::sync::atomic::{self, AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use url::Url;
use uuid::Uuid;

pub const FILE_CHUNK_SIZE: usize = 32768; //32 KB

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
    is_valid_url: AtomicBool,
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
    embedder_proxy: EmbedderProxy,
    store: Arc<FileManagerStore>,
}

impl FileManager {
    pub fn new(embedder_proxy: EmbedderProxy) -> FileManager {
        FileManager {
            embedder_proxy: embedder_proxy,
            store: Arc::new(FileManagerStore::new()),
        }
    }

    pub fn read_file(
        &self,
        sender: IpcSender<FileManagerResult<ReadFileProgress>>,
        id: Uuid,
        check_url_validity: bool,
        origin: FileOrigin,
    ) {
        let store = self.store.clone();
        thread::Builder::new()
            .name("read file".to_owned())
            .spawn(move || {
                if let Err(e) = store.try_read_file(&sender, id, check_url_validity, origin) {
                    let _ = sender.send(Err(FileManagerThreadError::BlobURLStoreError(e)));
                }
            })
            .expect("Thread spawning failed");
    }

    // Read a file for the Fetch implementation.
    // It gets the required headers synchronously and reads the actual content
    // in a separate thread.
    pub fn fetch_file(
        &self,
        done_sender: &servo_channel::Sender<Data>,
        cancellation_listener: Arc<Mutex<CancellationListener>>,
        id: Uuid,
        check_url_validity: bool,
        origin: FileOrigin,
        response: &mut Response,
        range: RangeRequestBounds,
    ) -> Result<(), BlobURLStoreError> {
        self.store.fetch_blob_buf(
            done_sender,
            cancellation_listener,
            &id,
            &origin,
            range,
            check_url_validity,
            response,
        )
    }

    pub fn promote_memory(
        &self,
        blob_buf: BlobBuf,
        set_valid: bool,
        sender: IpcSender<Result<Uuid, BlobURLStoreError>>,
        origin: FileOrigin,
    ) {
        let store = self.store.clone();
        thread::Builder::new()
            .name("transfer memory".to_owned())
            .spawn(move || {
                store.promote_memory(blob_buf, set_valid, sender, origin);
            })
            .expect("Thread spawning failed");
    }

    /// Message handler
    pub fn handle(&self, msg: FileManagerThreadMsg) {
        match msg {
            FileManagerThreadMsg::SelectFile(filter, sender, origin, opt_test_path) => {
                let store = self.store.clone();
                let embedder = self.embedder_proxy.clone();
                thread::Builder::new()
                    .name("select file".to_owned())
                    .spawn(move || {
                        store.select_file(filter, sender, origin, opt_test_path, embedder);
                    })
                    .expect("Thread spawning failed");
            },
            FileManagerThreadMsg::SelectFiles(filter, sender, origin, opt_test_paths) => {
                let store = self.store.clone();
                let embedder = self.embedder_proxy.clone();
                thread::Builder::new()
                    .name("select files".to_owned())
                    .spawn(move || {
                        store.select_files(filter, sender, origin, opt_test_paths, embedder);
                    })
                    .expect("Thread spawning failed");
            },
            FileManagerThreadMsg::ReadFile(sender, id, check_url_validity, origin) => {
                self.read_file(sender, id, check_url_validity, origin);
            },
            FileManagerThreadMsg::PromoteMemory(blob_buf, set_valid, sender, origin) => {
                self.promote_memory(blob_buf, set_valid, sender, origin);
            },
            FileManagerThreadMsg::AddSlicedURLEntry(id, rel_pos, sender, origin) => {
                self.store.add_sliced_url_entry(id, rel_pos, sender, origin);
            },
            FileManagerThreadMsg::DecRef(id, origin, sender) => {
                let _ = sender.send(self.store.dec_ref(&id, &origin));
            },
            FileManagerThreadMsg::RevokeBlobURL(id, origin, sender) => {
                let _ = sender.send(self.store.set_blob_url_validity(false, &id, &origin));
            },
            FileManagerThreadMsg::ActivateBlobURL(id, sender, origin) => {
                let _ = sender.send(self.store.set_blob_url_validity(true, &id, &origin));
            },
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
    fn get_impl(
        &self,
        id: &Uuid,
        origin_in: &FileOrigin,
        check_url_validity: bool,
    ) -> Result<FileImpl, BlobURLStoreError> {
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
            },
            None => Err(BlobURLStoreError::InvalidFileID),
        }
    }

    fn insert(&self, id: Uuid, entry: FileStoreEntry) {
        self.entries.write().unwrap().insert(id, entry);
    }

    fn remove(&self, id: &Uuid) {
        self.entries.write().unwrap().remove(id);
    }

    fn inc_ref(&self, id: &Uuid, origin_in: &FileOrigin) -> Result<(), BlobURLStoreError> {
        match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if entry.origin == *origin_in {
                    entry.refs.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                } else {
                    Err(BlobURLStoreError::InvalidOrigin)
                }
            },
            None => Err(BlobURLStoreError::InvalidFileID),
        }
    }

    fn add_sliced_url_entry(
        &self,
        parent_id: Uuid,
        rel_pos: RelativePos,
        sender: IpcSender<Result<Uuid, BlobURLStoreError>>,
        origin_in: FileOrigin,
    ) {
        match self.inc_ref(&parent_id, &origin_in) {
            Ok(_) => {
                let new_id = Uuid::new_v4();
                self.insert(
                    new_id,
                    FileStoreEntry {
                        origin: origin_in,
                        file_impl: FileImpl::Sliced(parent_id, rel_pos),
                        refs: AtomicUsize::new(1),
                        // Valid here since AddSlicedURLEntry implies URL creation
                        // from a BlobImpl::Sliced
                        is_valid_url: AtomicBool::new(true),
                    },
                );

                // We assume that the returned id will be held by BlobImpl::File
                let _ = sender.send(Ok(new_id));
            },
            Err(e) => {
                let _ = sender.send(Err(e));
            },
        }
    }

    fn query_files_from_embedder(
        &self,
        patterns: Vec<FilterPattern>,
        multiple_files: bool,
        embedder_proxy: EmbedderProxy,
    ) -> Option<Vec<String>> {
        let (ipc_sender, ipc_receiver) = ipc::channel().expect("Failed to create IPC channel!");
        let msg = (
            None,
            EmbedderMsg::SelectFiles(patterns, multiple_files, ipc_sender),
        );

        embedder_proxy.send(msg);
        match ipc_receiver.recv() {
            Ok(result) => result,
            Err(e) => {
                warn!("Failed to receive files from embedder ({}).", e);
                None
            },
        }
    }

    fn select_file(
        &self,
        patterns: Vec<FilterPattern>,
        sender: IpcSender<FileManagerResult<SelectedFile>>,
        origin: FileOrigin,
        opt_test_path: Option<String>,
        embedder_proxy: EmbedderProxy,
    ) {
        // Check if the select_files preference is enabled
        // to ensure process-level security against compromised script;
        // Then try applying opt_test_path directly for testing convenience
        let opt_s = if select_files_pref_enabled() {
            opt_test_path
        } else {
            self.query_files_from_embedder(patterns, false, embedder_proxy)
                .and_then(|mut x| x.pop())
        };

        match opt_s {
            Some(s) => {
                let selected_path = Path::new(&s);
                let result = self.create_entry(selected_path, &origin);
                let _ = sender.send(result);
            },
            None => {
                let _ = sender.send(Err(FileManagerThreadError::UserCancelled));
                return;
            },
        }
    }

    fn select_files(
        &self,
        patterns: Vec<FilterPattern>,
        sender: IpcSender<FileManagerResult<Vec<SelectedFile>>>,
        origin: FileOrigin,
        opt_test_paths: Option<Vec<String>>,
        embedder_proxy: EmbedderProxy,
    ) {
        // Check if the select_files preference is enabled
        // to ensure process-level security against compromised script;
        // Then try applying opt_test_paths directly for testing convenience
        let opt_v = if select_files_pref_enabled() {
            opt_test_paths
        } else {
            self.query_files_from_embedder(patterns, true, embedder_proxy)
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
                        },
                    };
                }

                let _ = sender.send(Ok(replies));
            },
            None => {
                let _ = sender.send(Err(FileManagerThreadError::UserCancelled));
                return;
            },
        }
    }

    fn create_entry(
        &self,
        file_path: &Path,
        origin: &str,
    ) -> Result<SelectedFile, FileManagerThreadError> {
        use net_traits::filemanager_thread::FileManagerThreadError::FileSystemError;

        let file = File::open(file_path).map_err(|e| FileSystemError(e.to_string()))?;
        let metadata = file
            .metadata()
            .map_err(|e| FileSystemError(e.to_string()))?;
        let modified = metadata
            .modified()
            .map_err(|e| FileSystemError(e.to_string()))?;
        let elapsed = modified
            .elapsed()
            .map_err(|e| FileSystemError(e.to_string()))?;
        // Unix Epoch: https://doc.servo.org/std/time/constant.UNIX_EPOCH.html
        let modified_epoch = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000;
        let file_size = metadata.len();
        let file_name = file_path
            .file_name()
            .ok_or(FileSystemError("Invalid filepath".to_string()))?;

        let file_impl = FileImpl::MetaDataOnly(FileMetaData {
            path: file_path.to_path_buf(),
            modified: modified_epoch,
            size: file_size,
        });

        let id = Uuid::new_v4();

        self.insert(
            id,
            FileStoreEntry {
                origin: origin.to_string(),
                file_impl: file_impl,
                refs: AtomicUsize::new(1),
                // Invalid here since create_entry is called by file selection
                is_valid_url: AtomicBool::new(false),
            },
        );

        let filename_path = Path::new(file_name);
        let type_string = match guess_mime_type_opt(filename_path) {
            Some(x) => format!("{}", x),
            None => "".to_string(),
        };

        Ok(SelectedFile {
            id: id,
            filename: filename_path.to_path_buf(),
            modified: modified_epoch,
            size: file_size,
            type_string: type_string,
        })
    }

    fn get_blob_buf(
        &self,
        sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
        id: &Uuid,
        origin_in: &FileOrigin,
        rel_pos: RelativePos,
        check_url_validity: bool,
    ) -> Result<(), BlobURLStoreError> {
        let file_impl = self.get_impl(id, origin_in, check_url_validity)?;
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
            },
            FileImpl::MetaDataOnly(metadata) => {
                /* XXX: Snapshot state check (optional) https://w3c.github.io/FileAPI/#snapshot-state.
                        Concretely, here we create another file, and this file might not
                        has the same underlying file state (meta-info plus content) as the time
                        create_entry is called.
                */

                let opt_filename = metadata
                    .path
                    .file_name()
                    .and_then(|osstr| osstr.to_str())
                    .map(|s| s.to_string());

                let mime = guess_mime_type_opt(metadata.path.clone());
                let range = rel_pos.to_abs_range(metadata.size as usize);

                let mut file = File::open(&metadata.path)
                    .map_err(|e| BlobURLStoreError::External(e.to_string()))?;
                let seeked_start = file
                    .seek(SeekFrom::Start(range.start as u64))
                    .map_err(|e| BlobURLStoreError::External(e.to_string()))?;

                if seeked_start == (range.start as u64) {
                    let type_string = match mime {
                        Some(x) => format!("{}", x),
                        None => "".to_string(),
                    };

                    read_file_in_chunks(sender, &mut file, range.len(), opt_filename, type_string);
                    Ok(())
                } else {
                    Err(BlobURLStoreError::InvalidEntry)
                }
            },
            FileImpl::Sliced(parent_id, inner_rel_pos) => {
                // Next time we don't need to check validity since
                // we have already done that for requesting URL if necessary
                self.get_blob_buf(
                    sender,
                    &parent_id,
                    origin_in,
                    rel_pos.slice_inner(&inner_rel_pos),
                    false,
                )
            },
        }
    }

    // Convenient wrapper over get_blob_buf
    fn try_read_file(
        &self,
        sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
        id: Uuid,
        check_url_validity: bool,
        origin_in: FileOrigin,
    ) -> Result<(), BlobURLStoreError> {
        self.get_blob_buf(
            sender,
            &id,
            &origin_in,
            RelativePos::full_range(),
            check_url_validity,
        )
    }

    fn fetch_blob_buf(
        &self,
        done_sender: &servo_channel::Sender<Data>,
        cancellation_listener: Arc<Mutex<CancellationListener>>,
        id: &Uuid,
        origin_in: &FileOrigin,
        range: RangeRequestBounds,
        check_url_validity: bool,
        response: &mut Response,
    ) -> Result<(), BlobURLStoreError> {
        let file_impl = self.get_impl(id, origin_in, check_url_validity)?;
        match file_impl {
            FileImpl::Memory(buf) => {
                let range = match range.get_final(Some(buf.size)) {
                    Ok(range) => range,
                    Err(_) => {
                        return Err(BlobURLStoreError::InvalidRange);
                    },
                };

                let range = range.to_abs_range(buf.size as usize);
                let len = range.len() as u64;

                set_headers(
                    &mut response.headers,
                    len,
                    buf.type_string.parse().unwrap_or(mime::TEXT_PLAIN),
                    /* filename */ None,
                );

                let mut bytes = vec![];
                bytes.extend_from_slice(buf.bytes.index(range));

                let _ = done_sender.send(Data::Payload(bytes));
                let _ = done_sender.send(Data::Done);

                Ok(())
            },
            FileImpl::MetaDataOnly(metadata) => {
                /* XXX: Snapshot state check (optional) https://w3c.github.io/FileAPI/#snapshot-state.
                        Concretely, here we create another file, and this file might not
                        has the same underlying file state (meta-info plus content) as the time
                        create_entry is called.
                */

                let file = File::open(&metadata.path)
                    .map_err(|e| BlobURLStoreError::External(e.to_string()))?;

                let range = match range.get_final(Some(metadata.size)) {
                    Ok(range) => range,
                    Err(_) => {
                        return Err(BlobURLStoreError::InvalidRange);
                    },
                };

                let mut reader = BufReader::with_capacity(FILE_CHUNK_SIZE, file);
                if reader.seek(SeekFrom::Start(range.start as u64)).is_err() {
                    return Err(BlobURLStoreError::External(
                        "Unexpected method for blob".into(),
                    ));
                }

                let filename = metadata
                    .path
                    .file_name()
                    .and_then(|osstr| osstr.to_str())
                    .map(|s| s.to_string());

                set_headers(
                    &mut response.headers,
                    metadata.size,
                    guess_mime_type_opt(metadata.path).unwrap_or(mime::TEXT_PLAIN),
                    filename,
                );

                fetch_file_in_chunks(
                    done_sender.clone(),
                    reader,
                    response.body.clone(),
                    cancellation_listener,
                    range,
                );

                Ok(())
            },
            FileImpl::Sliced(parent_id, inner_rel_pos) => {
                // Next time we don't need to check validity since
                // we have already done that for requesting URL if necessary.
                return self.fetch_blob_buf(
                    done_sender,
                    cancellation_listener,
                    &parent_id,
                    origin_in,
                    RangeRequestBounds::Final(
                        RelativePos::full_range().slice_inner(&inner_rel_pos),
                    ),
                    false,
                    response,
                );
            },
        }
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
            },
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

    fn promote_memory(
        &self,
        blob_buf: BlobBuf,
        set_valid: bool,
        sender: IpcSender<Result<Uuid, BlobURLStoreError>>,
        origin: FileOrigin,
    ) {
        match Url::parse(&origin) {
            // parse to check sanity
            Ok(_) => {
                let id = Uuid::new_v4();
                self.insert(
                    id,
                    FileStoreEntry {
                        origin: origin.clone(),
                        file_impl: FileImpl::Memory(blob_buf),
                        refs: AtomicUsize::new(1),
                        is_valid_url: AtomicBool::new(set_valid),
                    },
                );

                let _ = sender.send(Ok(id));
            },
            Err(_) => {
                let _ = sender.send(Err(BlobURLStoreError::InvalidOrigin));
            },
        }
    }

    fn set_blob_url_validity(
        &self,
        validity: bool,
        id: &Uuid,
        origin_in: &FileOrigin,
    ) -> Result<(), BlobURLStoreError> {
        let (do_remove, opt_parent_id, res) = match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if *entry.origin == *origin_in {
                    entry.is_valid_url.store(validity, Ordering::Release);

                    if !validity {
                        // Check if it is the last possible reference
                        // since refs only accounts for blob id holders
                        // and store entry id holders
                        let zero_refs = entry.refs.load(Ordering::Acquire) == 0;

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
            },
            None => (false, None, Err(BlobURLStoreError::InvalidFileID)),
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
    PREFS
        .get("dom.testing.htmlinputelement.select_files.enabled")
        .as_boolean()
        .unwrap_or(false)
}

fn read_file_in_chunks(
    sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
    file: &mut File,
    size: usize,
    opt_filename: Option<String>,
    type_string: String,
) {
    // First chunk
    let mut buf = vec![0; FILE_CHUNK_SIZE];
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
        },
        Err(e) => {
            let _ = sender.send(Err(FileManagerThreadError::FileSystemError(e.to_string())));
            return;
        },
    }

    // Send the remaining chunks
    loop {
        let mut buf = vec![0; FILE_CHUNK_SIZE];
        match file.read(&mut buf) {
            Ok(0) => {
                let _ = sender.send(Ok(ReadFileProgress::EOF));
                return;
            },
            Ok(n) => {
                buf.truncate(n);
                let _ = sender.send(Ok(ReadFileProgress::Partial(buf)));
            },
            Err(e) => {
                let _ = sender.send(Err(FileManagerThreadError::FileSystemError(e.to_string())));
                return;
            },
        }
    }
}

pub fn fetch_file_in_chunks(
    done_sender: servo_channel::Sender<Data>,
    mut reader: BufReader<File>,
    res_body: ServoArc<Mutex<ResponseBody>>,
    cancellation_listener: Arc<Mutex<CancellationListener>>,
    range: RelativePos,
) {
    thread::Builder::new()
        .name("fetch file worker thread".to_string())
        .spawn(move || {
            loop {
                if cancellation_listener.lock().unwrap().cancelled() {
                    *res_body.lock().unwrap() = ResponseBody::Done(vec![]);
                    let _ = done_sender.send(Data::Cancelled);
                    return;
                }
                let length = {
                    let buffer = reader.fill_buf().unwrap().to_vec();
                    let mut buffer_len = buffer.len();
                    if let ResponseBody::Receiving(ref mut body) = *res_body.lock().unwrap() {
                        let offset = usize::min(
                            {
                                if let Some(end) = range.end {
                                    let remaining_bytes =
                                        end as usize - range.start as usize - body.len() + 1;
                                    if remaining_bytes <= FILE_CHUNK_SIZE {
                                        // This is the last chunk so we set buffer
                                        // len to 0 to break the reading loop.
                                        buffer_len = 0;
                                        remaining_bytes
                                    } else {
                                        FILE_CHUNK_SIZE
                                    }
                                } else {
                                    FILE_CHUNK_SIZE
                                }
                            },
                            buffer.len(),
                        );
                        let chunk = &buffer[0..offset];
                        body.extend_from_slice(chunk);
                        let _ = done_sender.send(Data::Payload(chunk.to_vec()));
                    }
                    buffer_len
                };
                if length == 0 {
                    let mut body = res_body.lock().unwrap();
                    let completed_body = match *body {
                        ResponseBody::Receiving(ref mut body) => mem::replace(body, vec![]),
                        _ => vec![],
                    };
                    *body = ResponseBody::Done(completed_body);
                    let _ = done_sender.send(Data::Done);
                    break;
                }
                reader.consume(length);
            }
        })
        .expect("Failed to create fetch file worker thread");
}

fn set_headers(headers: &mut HeaderMap, content_length: u64, mime: Mime, filename: Option<String>) {
    headers.typed_insert(ContentLength(content_length));
    headers.typed_insert(ContentType::from(mime.clone()));
    let name = match filename {
        Some(name) => name,
        None => return,
    };
    let charset = mime.get_param(mime::CHARSET);
    let charset = charset
        .map(|c| c.as_ref().into())
        .unwrap_or("us-ascii".to_owned());
    // TODO(eijebong): Replace this once the typed header is there
    //                 https://github.com/hyperium/headers/issues/8
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_bytes(
            format!(
                "inline; {}",
                if charset.to_lowercase() == "utf-8" {
                    format!(
                        "filename=\"{}\"",
                        String::from_utf8(name.as_bytes().into()).unwrap()
                    )
                } else {
                    format!(
                        "filename*=\"{}\"''{}",
                        charset,
                        http_percent_encode(name.as_bytes())
                    )
                }
            )
            .as_bytes(),
        )
        .unwrap(),
    );
}
