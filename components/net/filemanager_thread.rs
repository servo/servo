/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::sync::atomic::{self, AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};

use embedder_traits::{EmbedderMsg, EmbedderProxy, FilterPattern};
use headers::{ContentLength, ContentType, HeaderMap, HeaderMapExt};
use http::header::{self, HeaderValue};
use ipc_channel::ipc::{self, IpcSender};
use log::warn;
use mime::{self, Mime};
use net_traits::blob_url_store::{BlobBuf, BlobURLStoreError};
use net_traits::filemanager_thread::{
    FileManagerResult, FileManagerThreadError, FileManagerThreadMsg, FileOrigin, FileTokenCheck,
    ReadFileProgress, RelativePos, SelectedFile,
};
use net_traits::http_percent_encode;
use net_traits::response::{Response, ResponseBody};
use servo_arc::Arc as ServoArc;
use servo_config::pref;
use tokio::sync::mpsc::UnboundedSender as TokioSender;
use url::Url;
use uuid::Uuid;

use crate::fetch::methods::{CancellationListener, Data, RangeRequestBounds};
use crate::resource_thread::CoreResourceThreadPool;

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
    /// UUIDs of fetch instances that acquired an interest in this file,
    /// when the url was still valid.
    outstanding_tokens: HashSet<Uuid>,
}

#[derive(Clone)]
struct FileMetaData {
    path: PathBuf,
    /// Modified time in UNIX Epoch format
    _modified: u64,
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
    thread_pool: Weak<CoreResourceThreadPool>,
}

impl FileManager {
    pub fn new(
        embedder_proxy: EmbedderProxy,
        pool_handle: Weak<CoreResourceThreadPool>,
    ) -> FileManager {
        FileManager {
            embedder_proxy,
            store: Arc::new(FileManagerStore::new()),
            thread_pool: pool_handle,
        }
    }

    pub fn read_file(
        &self,
        sender: IpcSender<FileManagerResult<ReadFileProgress>>,
        id: Uuid,
        origin: FileOrigin,
    ) {
        let store = self.store.clone();
        self.thread_pool
            .upgrade()
            .map(|pool| {
                pool.spawn(move || {
                    if let Err(e) = store.try_read_file(&sender, id, origin) {
                        let _ = sender.send(Err(FileManagerThreadError::BlobURLStoreError(e)));
                    }
                });
            })
            .unwrap_or_else(|| {
                warn!("FileManager tried to read a file after CoreResourceManager has exited.");
            });
    }

    pub fn get_token_for_file(&self, file_id: &Uuid) -> FileTokenCheck {
        self.store.get_token_for_file(file_id)
    }

    pub fn invalidate_token(&self, token: &FileTokenCheck, file_id: &Uuid) {
        self.store.invalidate_token(token, file_id);
    }

    /// Read a file for the Fetch implementation.
    /// It gets the required headers synchronously and reads the actual content
    /// in a separate thread.
    #[allow(clippy::too_many_arguments)]
    pub fn fetch_file(
        &self,
        done_sender: &mut TokioSender<Data>,
        cancellation_listener: Arc<Mutex<CancellationListener>>,
        id: Uuid,
        file_token: &FileTokenCheck,
        origin: FileOrigin,
        response: &mut Response,
        range: RangeRequestBounds,
    ) -> Result<(), BlobURLStoreError> {
        self.fetch_blob_buf(
            done_sender,
            cancellation_listener,
            &id,
            file_token,
            &origin,
            range,
            response,
        )
    }

    pub fn promote_memory(&self, id: Uuid, blob_buf: BlobBuf, set_valid: bool, origin: FileOrigin) {
        self.store.promote_memory(id, blob_buf, set_valid, origin);
    }

    /// Message handler
    pub fn handle(&self, msg: FileManagerThreadMsg) {
        match msg {
            FileManagerThreadMsg::SelectFile(filter, sender, origin, opt_test_path) => {
                let store = self.store.clone();
                let embedder = self.embedder_proxy.clone();
                self.thread_pool
                    .upgrade()
                    .map(|pool| {
                        pool.spawn(move || {
                            store.select_file(filter, sender, origin, opt_test_path, embedder);
                        });
                    })
                    .unwrap_or_else(|| {
                        warn!(
                            "FileManager tried to select a file after CoreResourceManager has exited."
                        );
                    });
            },
            FileManagerThreadMsg::SelectFiles(filter, sender, origin, opt_test_paths) => {
                let store = self.store.clone();
                let embedder = self.embedder_proxy.clone();
                self.thread_pool
                    .upgrade()
                    .map(|pool| {
                        pool.spawn(move || {
                            store.select_files(filter, sender, origin, opt_test_paths, embedder);
                        });
                    })
                    .unwrap_or_else(|| {
                        warn!(
                            "FileManager tried to select multiple files after CoreResourceManager has exited."
                        );
                    });
            },
            FileManagerThreadMsg::ReadFile(sender, id, origin) => {
                self.read_file(sender, id, origin);
            },
            FileManagerThreadMsg::PromoteMemory(id, blob_buf, set_valid, origin) => {
                self.promote_memory(id, blob_buf, set_valid, origin);
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

    pub fn fetch_file_in_chunks(
        &self,
        done_sender: &mut TokioSender<Data>,
        mut reader: BufReader<File>,
        res_body: ServoArc<Mutex<ResponseBody>>,
        cancellation_listener: Arc<Mutex<CancellationListener>>,
        range: RelativePos,
    ) {
        let done_sender = done_sender.clone();
        self.thread_pool
            .upgrade()
            .map(|pool| {
                pool.spawn(move || {
                    loop {
                        if cancellation_listener.lock().unwrap().cancelled() {
                            *res_body.lock().unwrap() = ResponseBody::Done(vec![]);
                            let _ = done_sender.send(Data::Cancelled);
                            return;
                        }
                        let length = {
                            let buffer = reader.fill_buf().unwrap().to_vec();
                            let mut buffer_len = buffer.len();
                            if let ResponseBody::Receiving(ref mut body) = *res_body.lock().unwrap()
                            {
                                let offset = usize::min(
                                    {
                                        if let Some(end) = range.end {
                                            // HTTP Range requests are specified with closed ranges,
                                            // while Rust uses half-open ranges. We add +1 here so
                                            // we don't skip the last requested byte.
                                            let remaining_bytes =
                                                end as usize - range.start as usize - body.len() +
                                                    1;
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
                                ResponseBody::Receiving(ref mut body) => std::mem::take(body),
                                _ => vec![],
                            };
                            *body = ResponseBody::Done(completed_body);
                            let _ = done_sender.send(Data::Done);
                            break;
                        }
                        reader.consume(length);
                    }
                });
            })
            .unwrap_or_else(|| {
                warn!("FileManager tried to fetch a file in chunks after CoreResourceManager has exited.");
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_blob_buf(
        &self,
        done_sender: &mut TokioSender<Data>,
        cancellation_listener: Arc<Mutex<CancellationListener>>,
        id: &Uuid,
        file_token: &FileTokenCheck,
        origin_in: &FileOrigin,
        range: RangeRequestBounds,
        response: &mut Response,
    ) -> Result<(), BlobURLStoreError> {
        let file_impl = self.store.get_impl(id, file_token, origin_in)?;
        match file_impl {
            FileImpl::Memory(buf) => {
                let range = range
                    .get_final(Some(buf.size))
                    .map_err(|_| BlobURLStoreError::InvalidRange)?;

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

                let range = range
                    .get_final(Some(metadata.size))
                    .map_err(|_| BlobURLStoreError::InvalidRange)?;

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
                    mime_guess::from_path(metadata.path)
                        .first()
                        .unwrap_or(mime::TEXT_PLAIN),
                    filename,
                );

                self.fetch_file_in_chunks(
                    &mut done_sender.clone(),
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
                self.fetch_blob_buf(
                    done_sender,
                    cancellation_listener,
                    &parent_id,
                    file_token,
                    origin_in,
                    RangeRequestBounds::Final(
                        RelativePos::full_range().slice_inner(&inner_rel_pos),
                    ),
                    response,
                )
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
    pub fn get_impl(
        &self,
        id: &Uuid,
        file_token: &FileTokenCheck,
        origin_in: &FileOrigin,
    ) -> Result<FileImpl, BlobURLStoreError> {
        match self.entries.read().unwrap().get(id) {
            Some(entry) => {
                if *origin_in != *entry.origin {
                    Err(BlobURLStoreError::InvalidOrigin)
                } else {
                    match file_token {
                        FileTokenCheck::NotRequired => Ok(entry.file_impl.clone()),
                        FileTokenCheck::Required(token) => {
                            if entry.outstanding_tokens.contains(token) {
                                return Ok(entry.file_impl.clone());
                            }
                            Err(BlobURLStoreError::InvalidFileID)
                        },
                        FileTokenCheck::ShouldFail => Err(BlobURLStoreError::InvalidFileID),
                    }
                }
            },
            None => Err(BlobURLStoreError::InvalidFileID),
        }
    }

    pub fn invalidate_token(&self, token: &FileTokenCheck, file_id: &Uuid) {
        if let FileTokenCheck::Required(token) = token {
            let mut entries = self.entries.write().unwrap();
            if let Some(entry) = entries.get_mut(file_id) {
                entry.outstanding_tokens.remove(token);

                // Check if there are references left.
                let zero_refs = entry.refs.load(Ordering::Acquire) == 0;

                // Check if no other fetch has acquired a token for this file.
                let no_outstanding_tokens = entry.outstanding_tokens.is_empty();

                // Check if there is still a blob URL outstanding.
                let valid = entry.is_valid_url.load(Ordering::Acquire);

                // Can we remove this file?
                let do_remove = zero_refs && no_outstanding_tokens && !valid;

                if do_remove {
                    entries.remove(file_id);
                }
            }
        }
    }

    pub fn get_token_for_file(&self, file_id: &Uuid) -> FileTokenCheck {
        let mut entries = self.entries.write().unwrap();
        let parent_id = match entries.get(file_id) {
            Some(entry) => {
                if let FileImpl::Sliced(ref parent_id, _) = entry.file_impl {
                    Some(*parent_id)
                } else {
                    None
                }
            },
            None => return FileTokenCheck::ShouldFail,
        };
        let file_id = match parent_id.as_ref() {
            Some(id) => id,
            None => file_id,
        };
        if let Some(entry) = entries.get_mut(file_id) {
            if !entry.is_valid_url.load(Ordering::Acquire) {
                return FileTokenCheck::ShouldFail;
            }
            let token = Uuid::new_v4();
            entry.outstanding_tokens.insert(token);
            return FileTokenCheck::Required(token);
        }
        FileTokenCheck::ShouldFail
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
                        outstanding_tokens: Default::default(),
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
                warn!("Failed to receive files from embedder ({:?}).", e);
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
        let opt_s = if pref!(dom.testing.html_input_element.select_files.enabled) {
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
        let opt_v = if pref!(dom.testing.html_input_element.select_files.enabled) {
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
            _modified: modified_epoch,
            size: file_size,
        });

        let id = Uuid::new_v4();

        self.insert(
            id,
            FileStoreEntry {
                origin: origin.to_string(),
                file_impl,
                refs: AtomicUsize::new(1),
                // Invalid here since create_entry is called by file selection
                is_valid_url: AtomicBool::new(false),
                outstanding_tokens: Default::default(),
            },
        );

        let filename_path = Path::new(file_name);
        let type_string = match mime_guess::from_path(filename_path).first() {
            Some(x) => format!("{}", x),
            None => "".to_string(),
        };

        Ok(SelectedFile {
            id,
            filename: filename_path.to_path_buf(),
            modified: modified_epoch,
            size: file_size,
            type_string,
        })
    }

    fn get_blob_buf(
        &self,
        sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
        id: &Uuid,
        file_token: &FileTokenCheck,
        origin_in: &FileOrigin,
        rel_pos: RelativePos,
    ) -> Result<(), BlobURLStoreError> {
        let file_impl = self.get_impl(id, file_token, origin_in)?;
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

                let mime = mime_guess::from_path(metadata.path.clone()).first();
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
                    file_token,
                    origin_in,
                    rel_pos.slice_inner(&inner_rel_pos),
                )
            },
        }
    }

    // Convenient wrapper over get_blob_buf
    fn try_read_file(
        &self,
        sender: &IpcSender<FileManagerResult<ReadFileProgress>>,
        id: Uuid,
        origin_in: FileOrigin,
    ) -> Result<(), BlobURLStoreError> {
        self.get_blob_buf(
            sender,
            &id,
            &FileTokenCheck::NotRequired,
            &origin_in,
            RelativePos::full_range(),
        )
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

                        // Check if no fetch has acquired a token for this file.
                        let no_outstanding_tokens = entry.outstanding_tokens.is_empty();

                        // Can we remove this file?
                        let do_remove = !is_valid && no_outstanding_tokens;

                        if let FileImpl::Sliced(ref parent_id, _) = entry.file_impl {
                            (do_remove, Some(*parent_id))
                        } else {
                            (do_remove, None)
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

    fn promote_memory(&self, id: Uuid, blob_buf: BlobBuf, set_valid: bool, origin: FileOrigin) {
        // parse to check sanity
        if Url::parse(&origin).is_err() {
            return;
        }
        self.insert(
            id,
            FileStoreEntry {
                origin,
                file_impl: FileImpl::Memory(blob_buf),
                refs: AtomicUsize::new(1),
                is_valid_url: AtomicBool::new(set_valid),
                outstanding_tokens: Default::default(),
            },
        );
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

                        // Check if no fetch has acquired a token for this file.
                        let no_outstanding_tokens = entry.outstanding_tokens.is_empty();

                        // Can we remove this file?
                        let do_remove = zero_refs && no_outstanding_tokens;

                        if let FileImpl::Sliced(ref parent_id, _) = entry.file_impl {
                            (do_remove, Some(*parent_id), Ok(()))
                        } else {
                            (do_remove, None, Ok(()))
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
                type_string,
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
