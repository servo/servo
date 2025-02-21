/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::Modal;
use egui_file_dialog::{DialogState, FileDialog as EguiFileDialog};
use log::warn;
use servo::ipc_channel::ipc::IpcSender;
use servo::{
    AlertResponse, AuthenticationRequest, ConfirmResponse, FilterPattern, PermissionRequest,
    PromptResponse, SimpleDialog,
};

pub enum Dialog {
    File {
        dialog: EguiFileDialog,
        multiple: bool,
        response_sender: IpcSender<Option<Vec<PathBuf>>>,
    },
    #[allow(clippy::enum_variant_names, reason = "spec terminology")]
    SimpleDialog(SimpleDialog),
    Authentication {
        username: String,
        password: String,
        request: Option<AuthenticationRequest>,
    },
    Permission {
        message: String,
        request: Option<PermissionRequest>,
    },
    SelectDevice {
        devices: Vec<String>,
        selected_device_index: usize,
        response_sender: IpcSender<Option<String>>,
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
                            .is_some_and(|ext| {
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

    pub fn new_simple_dialog(dialog: SimpleDialog) -> Self {
        Self::SimpleDialog(dialog)
    }

    pub fn new_authentication_dialog(authentication_request: AuthenticationRequest) -> Self {
        Dialog::Authentication {
            username: String::new(),
            password: String::new(),
            request: Some(authentication_request),
        }
    }

    pub fn new_permission_request_dialog(permission_request: PermissionRequest) -> Self {
        let message = format!(
            "Do you want to grant permission for {:?}?",
            permission_request.feature()
        );
        Dialog::Permission {
            message,
            request: Some(permission_request),
        }
    }

    pub fn new_device_selection_dialog(
        devices: Vec<String>,
        response_sender: IpcSender<Option<String>>,
    ) -> Self {
        Dialog::SelectDevice {
            devices,
            selected_device_index: 0,
            response_sender,
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
            Dialog::SimpleDialog(SimpleDialog::Alert {
                message,
                response_sender,
            }) => {
                let mut is_open = true;
                let modal = Modal::new("Alert".into());
                modal.show(ctx, |ui| {
                    make_dialog_label(message, ui, None);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Close").clicked() {
                                is_open = false;
                                if let Err(e) = response_sender.send(AlertResponse::Ok) {
                                    warn!("Failed to send alert dialog response: {}", e);
                                }
                            }
                        },
                    );
                });
                is_open
            },
            Dialog::SimpleDialog(SimpleDialog::Confirm {
                message,
                response_sender,
            }) => {
                let mut is_open = true;
                let modal = Modal::new("Confirm".into());
                modal.show(ctx, |ui| {
                    make_dialog_label(message, ui, None);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Ok").clicked() {
                                is_open = false;
                                if let Err(e) = response_sender.send(ConfirmResponse::Ok) {
                                    warn!("Failed to send alert dialog response: {}", e);
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                is_open = false;
                                if let Err(e) = response_sender.send(ConfirmResponse::Cancel) {
                                    warn!("Failed to send alert dialog response: {}", e);
                                }
                            }
                        },
                    );
                });
                is_open
            },
            Dialog::SimpleDialog(SimpleDialog::Prompt {
                message,
                // The `default` field gets reused as the input buffer.
                default: input,
                response_sender,
            }) => {
                let mut is_open = true;
                Modal::new("Prompt".into()).show(ctx, |ui| {
                    make_dialog_label(message, ui, Some(input));
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Ok").clicked() {
                                is_open = false;
                                if let Err(e) =
                                    response_sender.send(PromptResponse::Ok(input.clone()))
                                {
                                    warn!("Failed to send input dialog response: {}", e);
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                is_open = false;
                                if let Err(e) = response_sender.send(PromptResponse::Cancel) {
                                    warn!("Failed to send input dialog response: {}", e);
                                }
                            }
                        },
                    );
                });
                is_open
            },
            Dialog::Authentication {
                username,
                password,
                ref mut request,
            } => {
                let mut is_open = true;
                Modal::new("authentication".into()).show(ctx, |ui| {
                    let mut frame = egui::Frame::default().inner_margin(10.0).begin(ui);
                    frame.content_ui.set_min_width(150.0);

                    if let Some(ref request) = request {
                        let url =
                            egui::RichText::new(request.url().origin().unicode_serialization());
                        frame.content_ui.heading(url);
                    }

                    frame.content_ui.add_space(10.0);

                    frame
                        .content_ui
                        .label("This site is asking you to sign in.");
                    frame.content_ui.add_space(10.0);

                    frame.content_ui.label("Username:");
                    frame.content_ui.text_edit_singleline(username);
                    frame.content_ui.add_space(10.0);

                    frame.content_ui.label("Password:");
                    frame
                        .content_ui
                        .add(egui::TextEdit::singleline(password).password(true));

                    frame.end(ui);

                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Sign in").clicked() {
                                let request =
                                    request.take().expect("non-None until dialog is closed");
                                request.authenticate(username.clone(), password.clone());
                                is_open = false;
                            }
                            if ui.button("Cancel").clicked() {
                                is_open = false;
                            }
                        },
                    );
                });
                is_open
            },
            Dialog::Permission { message, request } => {
                let mut is_open = true;
                let modal = Modal::new("permission".into());
                modal.show(ctx, |ui| {
                    make_dialog_label(message, ui, None);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Allow").clicked() {
                                let request =
                                    request.take().expect("non-None until dialog is closed");
                                request.allow();
                                is_open = false;
                            }
                            if ui.button("Deny").clicked() {
                                let request =
                                    request.take().expect("non-None until dialog is closed");
                                request.deny();
                                is_open = false;
                            }
                        },
                    );
                });
                is_open
            },
            Dialog::SelectDevice {
                devices,
                selected_device_index,
                response_sender,
            } => {
                let mut is_open = true;
                let modal = Modal::new("device_picker".into());
                modal.show(ctx, |ui| {
                    let mut frame = egui::Frame::default().inner_margin(10.0).begin(ui);
                    frame.content_ui.set_min_width(150.0);

                    frame.content_ui.heading("Choose a Device");
                    frame.content_ui.add_space(10.0);

                    egui::ComboBox::from_label("")
                        .selected_text(&devices[*selected_device_index + 1])
                        .show_ui(&mut frame.content_ui, |ui| {
                            for i in (0..devices.len() - 1).step_by(2) {
                                let device_name = &devices[i + 1];
                                ui.selectable_value(selected_device_index, i, device_name);
                            }
                        });

                    frame.end(ui);

                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Ok").clicked() {
                                if let Err(e) = response_sender
                                    .send(Some(devices[*selected_device_index].clone()))
                                {
                                    warn!("Failed to send device selection: {}", e);
                                }
                                is_open = false;
                            }
                            if ui.button("Cancel").clicked() {
                                if let Err(e) = response_sender.send(None) {
                                    warn!("Failed to send cancellation: {}", e);
                                }
                                is_open = false;
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
