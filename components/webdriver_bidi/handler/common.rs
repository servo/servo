use std::{collections::HashSet, path::PathBuf};

use embedder_traits::webdriver_bidi::WebDriverBidiToEmbedderMsg;

use crate::{error::WebDriverBidiError, handler::Handler};

impl Handler {
    /// <https://fs.spec.whatwg.org/#locating-an-entry>
    pub(super) async fn locate_entry<T: LocateEntryOutput>(
        &self,
        _locator: &T,
    ) -> Option<T::Output> {
        unimplemented!()
    }
}

impl Handler {
    pub(super) fn send_message_to_embedder(
        &self,
        msg: WebDriverBidiToEmbedderMsg,
    ) -> Result<(), WebDriverBidiError> {
        self.0.embedder_sender.send(msg)?;
        self.0.event_loop_waker.wake();
        Ok(())
    }
}

pub(super) struct FileLocator {
    pub path: PathBuf,
    pub root: String,
}

pub(super) struct DirectoryLocator {
    pub path: PathBuf,
    pub root: String,
}

/// <https://fs.spec.whatwg.org/#file-system-locator>
pub(super) enum FileSystemLocator {
    File(FileLocator),
    Directory(DirectoryLocator),
}

pub(super) enum FileSystemLocatorKind {
    File,
    Directory,
}

pub(super) enum FileSystemEntry {
    File(FileEntry),
    Directory(DirectoryEntry),
}

pub(super) struct FileEntry {
    pub binary_data: Vec<u8>,
    pub modification_timestamp: (),
    pub lock: (),
}

pub(super) struct DirectoryEntry {
    pub children: HashSet<FileSystemEntry>,
}

pub(super) trait LocateEntryOutput {
    type Output;
}

impl LocateEntryOutput for FileLocator {
    type Output = FileEntry;
}

impl LocateEntryOutput for DirectoryLocator {
    type Output = DirectoryEntry;
}

impl LocateEntryOutput for FileSystemLocator {
    type Output = FileSystemEntry;
}
