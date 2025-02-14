/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::Modal;
use egui_file_dialog::{DialogState, FileDialog as EguiFileDialog};
use log::warn;
use servo::ipc_channel::ipc::IpcSender;
use servo::{FilterPattern, PromptResult};

pub enum Dialog {
    File {
        dialog: EguiFileDialog,
        multiple: bool,
        response_sender: IpcSender<Option<Vec<PathBuf>>>,
    },
    Alert {
        message: String,
        sender: IpcSender<()>,
    },
    OkCancel {
        message: String,
        sender: IpcSender<PromptResult>,
    },
    Input {
        message: String,
        input_text: String,
        sender: IpcSender<Option<String>>,
    },
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

        Dialog::File {
            dialog,
            multiple,
            response_sender,
        }
    }

    pub fn new_alert_dialog(message: String, sender: IpcSender<()>) -> Self {
        Dialog::Alert { message, sender }
    }

    pub fn new_okcancel_dialog(message: String, sender: IpcSender<PromptResult>) -> Self {
        Dialog::OkCancel { message, sender }
    }

    pub fn new_input_dialog(
        message: String,
        default: String,
        sender: IpcSender<Option<String>>,
    ) -> Self {
        Dialog::Input {
            message,
            input_text: default,
            sender,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        match self {
            Dialog::File {
                dialog,
                multiple,
                response_sender,
            } => {
                if dialog.state() == DialogState::Closed {
                    if *multiple {
                        dialog.pick_multiple();
                    } else {
                        dialog.pick_file();
                    }
                }

                let state = dialog.update(ctx).state();
                match state {
                    DialogState::Open => true,
                    DialogState::Picked(path) => {
                        if let Err(e) = response_sender.send(Some(vec![path])) {
                            warn!("Failed to send file selection response: {}", e);
                        }
                        false
                    },
                    DialogState::PickedMultiple(paths) => {
                        if let Err(e) = response_sender.send(Some(paths)) {
                            warn!("Failed to send file selection response: {}", e);
                        }
                        false
                    },
                    DialogState::Cancelled => {
                        if let Err(e) = response_sender.send(None) {
                            warn!("Failed to send cancellation response: {}", e);
                        }
                        false
                    },
                    DialogState::Closed => false,
                }
            },
            Dialog::Alert { message, sender } => {
                let mut is_open = true;
                let modal = Modal::new("alert".into());
                modal.show(ctx, |ui| {
                    make_dialog_label(message, ui, None);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Close").clicked() {
                                is_open = false;
                                if let Err(e) = sender.send(()) {
                                    warn!("Failed to send alert dialog response: {}", e);
                                }
                            }
                        },
                    );
                });
                is_open
            },
            Dialog::OkCancel { message, sender } => {
                let mut is_open = true;
                let modal = Modal::new("OkCancel".into());
                modal.show(ctx, |ui| {
                    make_dialog_label(message, ui, None);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Ok").clicked() {
                                is_open = false;
                                if let Err(e) = sender.send(PromptResult::Primary) {
                                    warn!("Failed to send alert dialog response: {}", e);
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                is_open = false;
                                if let Err(e) = sender.send(PromptResult::Secondary) {
                                    warn!("Failed to send alert dialog response: {}", e);
                                }
                            }
                        },
                    );
                });
                is_open
            },
            Dialog::Input {
                message,
                input_text,
                sender,
            } => {
                let mut is_open = true;
                Modal::new("input".into()).show(ctx, |ui| {
                    make_dialog_label(message, ui, Some(input_text));
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Ok").clicked() {
                                is_open = false;
                                if let Err(e) = sender.send(Some(input_text.clone())) {
                                    warn!("Failed to send input dialog response: {}", e);
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                is_open = false;
                                if let Err(e) = sender.send(None) {
                                    warn!("Failed to send input dialog response: {}", e);
                                }
                            }
                        },
                    );
                });
                is_open
            },
        }
    }
}

fn make_dialog_label(message: &str, ui: &mut egui::Ui, input_text: Option<&mut String>) {
    let mut frame = egui::Frame::default().inner_margin(10.0).begin(ui);
    frame.content_ui.set_min_width(150.0);
    frame.content_ui.label(message);
    if let Some(input_text) = input_text {
        frame.content_ui.text_edit_singleline(input_text);
    }
    frame.end(ui);
}
