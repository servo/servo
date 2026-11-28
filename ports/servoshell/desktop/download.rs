/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use egui::{Id, Modal, ProgressBar};
use rfd::AsyncFileDialog;
use servo::{DownloadCancellation, UnsupportedResponse};
use tempfile::NamedTempFile;
use winit::window::Window;

type SavePicker = Pin<Box<dyn Future<Output = Option<PathBuf>>>>;

pub(crate) enum DownloadDialog {
    Choose(PendingDownload),
    Save(SavingDownload),
    Finished(io::Result<PathBuf>),
}

pub(crate) struct PendingDownload {
    response: UnsupportedResponse,
    picker: Option<SavePicker>,
}

impl Drop for PendingDownload {
    fn drop(&mut self) {
        self.response.decline();
    }
}

pub(crate) struct SavingDownload {
    path: PathBuf,
    length: Option<u64>,
    received: Arc<AtomicU64>,
    cancellation: DownloadCancellation,
    result: Receiver<io::Result<()>>,
}

impl Drop for SavingDownload {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

impl DownloadDialog {
    pub(crate) fn new(response: UnsupportedResponse) -> Self {
        Self::Choose(PendingDownload {
            response,
            picker: None,
        })
    }

    pub(crate) fn update(&mut self, ctx: &egui::Context, window: &Window, waker: Waker) -> bool {
        match self {
            Self::Choose(pending) => {
                if pending.picker.is_none() && !pending.response.is_pending() {
                    return false;
                }
                let picker = pending.picker.get_or_insert_with(|| {
                    let mut dialog = AsyncFileDialog::new()
                        .set_parent(window)
                        .set_file_name(suggested_filename(&pending.response));
                    if let Some(downloads) = dirs::download_dir() {
                        dialog = dialog.set_directory(downloads);
                    }
                    let future = dialog.save_file();
                    Box::pin(async move { future.await.map(|handle| handle.path().to_owned()) })
                });
                match picker.as_mut().poll(&mut Context::from_waker(&waker)) {
                    Poll::Pending => return true,
                    Poll::Ready(None) => return false,
                    Poll::Ready(Some(path)) => {
                        *self = match SavingDownload::start(&mut pending.response, path, waker) {
                            Ok(transfer) => Self::Save(transfer),
                            Err(error) => Self::Finished(Err(error)),
                        };
                        ctx.request_repaint();
                    },
                }
            },
            Self::Save(transfer) => {
                match transfer.result.try_recv() {
                    Ok(Err(error)) if error.kind() == io::ErrorKind::Interrupted => return false,
                    Ok(result) => {
                        *self = Self::Finished(result.map(|()| transfer.path.clone()));
                        ctx.request_repaint();
                        return true;
                    },
                    Err(TryRecvError::Disconnected) => {
                        *self = Self::Finished(Err(io::Error::other("Download worker stopped")));
                        ctx.request_repaint();
                        return true;
                    },
                    Err(TryRecvError::Empty) => {},
                }
                let modal = Modal::new(Id::new("download-in-progress")).show(ctx, |ui| {
                    ui.heading("Downloading");
                    ui.label(
                        transfer
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                    );
                    let received = transfer.received.load(Ordering::Relaxed);
                    if let Some(total) = transfer.length.filter(|total| *total > 0) {
                        ui.add(ProgressBar::new(received as f32 / total as f32).show_percentage());
                    } else {
                        ui.label(format!("{received} bytes received"));
                    }
                    ui.add_enabled(
                        !transfer.cancellation.is_cancelled(),
                        egui::Button::new("Cancel"),
                    )
                    .clicked()
                });
                if modal.inner {
                    transfer.cancellation.cancel();
                }
            },
            Self::Finished(result) => {
                let modal = Modal::new(Id::new("download-result")).show(ctx, |ui| {
                    match result {
                        Ok(path) => {
                            ui.heading("Download complete");
                            ui.label(path.display().to_string());
                        },
                        Err(error) => {
                            ui.heading("Download failed");
                            ui.label(error.to_string());
                        },
                    }
                    ui.button("Close").clicked()
                });
                return !(modal.inner || modal.should_close());
            },
        }
        true
    }
}

impl SavingDownload {
    fn start(response: &mut UnsupportedResponse, path: PathBuf, waker: Waker) -> io::Result<Self> {
        let directory = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Choose a destination file")
        })?;
        let file = tempfile::Builder::new()
            .prefix(".servo-download-")
            .suffix(".part")
            .tempfile_in(directory)?;
        let mut stream = response.accept().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Interrupted, "The download request ended")
        })?;
        let cancellation = stream.cancellation();
        let download_cancellation = cancellation.clone();
        let length = response.content_length;
        let received = Arc::new(AtomicU64::new(0));
        let progress = received.clone();
        let destination = path.clone();
        let (sender, result) = mpsc::channel();
        std::thread::Builder::new()
            .name("servo-download".into())
            .spawn(move || {
                let mut last_repaint = Instant::now();
                let result = save_download(
                    file,
                    &destination,
                    length,
                    || {
                        stream
                            .read_chunk()
                            .map_err(|error| io::Error::other(format!("{error:?}")))
                    },
                    || download_cancellation.is_cancelled(),
                    |bytes| {
                        progress.store(bytes, Ordering::Relaxed);
                        if last_repaint.elapsed() >= Duration::from_millis(100) {
                            waker.wake_by_ref();
                            last_repaint = Instant::now();
                        }
                    },
                );
                let _ = sender.send(result);
                waker.wake();
            })?;
        Ok(Self {
            path,
            length,
            received,
            cancellation,
            result,
        })
    }
}

fn save_download<Chunk: AsRef<[u8]>>(
    mut file: NamedTempFile,
    path: &Path,
    expected: Option<u64>,
    mut read_chunk: impl FnMut() -> io::Result<Option<Chunk>>,
    cancelled: impl Fn() -> bool,
    mut progress: impl FnMut(u64),
) -> io::Result<()> {
    let mut received = 0;
    loop {
        let chunk = read_chunk();
        if cancelled() {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let Some(chunk) = chunk? else {
            break;
        };
        file.write_all(chunk.as_ref())?;
        received += chunk.as_ref().len() as u64;
        progress(received);
    }
    if expected.is_some_and(|expected| expected != received) {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Incomplete download",
        ));
    }
    file.as_file().sync_all()?;
    mark_quarantine(file.as_file())?;
    if cancelled() {
        return Err(io::ErrorKind::Interrupted.into());
    }
    file.persist_noclobber(path)?;
    Ok(())
}

fn suggested_filename(response: &UnsupportedResponse) -> String {
    let from_url = || {
        let name = response.url.path().rsplit('/').next().unwrap_or_default();
        percent_encoding::percent_decode_str(name)
            .decode_utf8_lossy()
            .into_owned()
    };
    let name = if response.default_filename.trim().is_empty() {
        from_url()
    } else {
        response.default_filename.clone()
    };
    sanitize_filename(&name)
}

fn sanitize_filename(name: &str) -> String {
    let name: String = name
        .chars()
        .filter(|character| {
            !character.is_control()
                && !matches!(*character,
            '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .map(|character| match character {
            '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*' => '_',
            _ => character,
        })
        .collect();
    let name = name
        .trim()
        .trim_start_matches('.')
        .trim_end_matches(['.', ' ']);
    if name.is_empty() {
        return "download".to_owned();
    }
    name.to_owned()
}

#[cfg(target_os = "macos")]
fn mark_quarantine(file: &std::fs::File) -> io::Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    use xattr::FileExt;

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let value = format!("0001;{:x};Servo;", time.as_secs());
    file.set_xattr("com.apple.quarantine", value.as_bytes())
}

#[cfg(not(target_os = "macos"))]
fn mark_quarantine(_: &std::fs::File) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_suggestions_are_single_visible_filenames() {
        assert_eq!(sanitize_filename(".."), "download");
        assert_eq!(sanitize_filename("folder/a:b\\file"), "folder_a_b_file");
        assert_eq!(sanitize_filename(".photo\u{202e}.zip"), "photo.zip");
    }

    #[test]
    fn successful_download_is_published_with_the_exact_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("hello.txt");
        let file = NamedTempFile::new_in(directory.path()).unwrap();
        let mut chunks = [b"hel".as_slice(), b"lo".as_slice()].into_iter();
        let mut counts = Vec::new();
        save_download(
            file,
            &path,
            Some(5),
            || Ok(chunks.next()),
            || false,
            |count| {
                counts.push(count);
            },
        )
        .unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"hello");
        assert_eq!(counts, [3, 5]);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        #[cfg(target_os = "macos")]
        assert!(xattr::get(&path, "com.apple.quarantine").unwrap().is_some());
    }

    #[test]
    fn cancellation_or_truncation_leaves_no_partial_file() {
        use std::cell::Cell;

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("hello.txt");
        for cancel in [false, true] {
            let file = NamedTempFile::new_in(directory.path()).unwrap();
            let mut chunks = [b"short".as_slice()].into_iter();
            let cancelled = Cell::new(false);
            let result = save_download(
                file,
                &path,
                Some(10),
                || Ok(chunks.next()),
                || cancelled.get(),
                |_| cancelled.set(cancel),
            );
            let expected = if cancel {
                io::ErrorKind::Interrupted
            } else {
                io::ErrorKind::UnexpectedEof
            };
            assert_eq!(result.unwrap_err().kind(), expected);
            assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn existing_destination_survives_a_save_failure() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("hello.txt");
        std::fs::write(&path, b"existing file").unwrap();
        let file = NamedTempFile::new_in(directory.path()).unwrap();
        let mut chunk = Some(b"new file".as_slice());
        let result = save_download(file, &path, None, || Ok(chunk.take()), || false, |_| {});
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&path).unwrap(), b"existing file");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
