/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui_file_dialog::{DialogState, FileDialog as EguiFileDialog};
use log::warn;
use servo::ipc_channel::ipc::IpcSender;
use servo::FilterPattern;

#[derive(Debug)]
pub struct FileDialog {
    dialog: EguiFileDialog,
    multiple: bool,
    response_sender: IpcSender<Option<Vec<PathBuf>>>,
}

#[derive(Debug)]
pub enum Dialog {
    File(FileDialog),
}

impl Dialog {
    pub fn new_file_dialog(
        multiple: bool,
        response_sender: IpcSender<Option<Vec<PathBuf>>>,
        patterns: Vec<FilterPattern>,
    ) -> Self {
        let mut dialog = EguiFileDialog::new();
        if !patterns.is_empty() {
            dialog = dialog
                .add_file_filter(
                    "All Supported Types",
                    Arc::new(move |path: &Path| {
                        path.extension()
                            .and_then(|e| e.to_str())
                            .map_or(false, |ext| {
                                let ext = ext.to_lowercase();
                                patterns.iter().any(|pattern| ext == pattern.0)
                            })
                    }),
                )
                .default_file_filter("All Supported Types");
        }

        let dialog = FileDialog {
            dialog,
            multiple,
            response_sender,
        };

        Dialog::File(dialog)
    }

    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        match self {
            Dialog::File(dialog) => {
                if dialog.dialog.state() == DialogState::Closed {
                    if dialog.multiple {
                        dialog.dialog.pick_multiple();
                    } else {
                        dialog.dialog.pick_file();
                    }
                }

                let state = dialog.dialog.update(ctx).state();

                match state {
                    DialogState::Open => true,
                    DialogState::Selected(path) => {
                        if let Err(e) = dialog.response_sender.send(Some(vec![path])) {
                            warn!("Failed to send file selection response: {}", e);
                        }
                        false
                    },
                    DialogState::SelectedMultiple(paths) => {
                        if let Err(e) = dialog.response_sender.send(Some(paths)) {
                            warn!("Failed to send file selection response: {}", e);
                        }
                        false
                    },
                    DialogState::Cancelled => {
                        if let Err(e) = dialog.response_sender.send(None) {
                            warn!("Failed to send cancellation response: {}", e);
                        }
                        false
                    },
                    DialogState::Closed => false,
                }
            },
        }
    }
}
