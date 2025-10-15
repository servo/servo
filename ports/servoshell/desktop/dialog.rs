/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::sync::Arc;

use egui::Modal;
use egui_file_dialog::{DialogState, FileDialog as EguiFileDialog};
use euclid::Length;
use log::warn;
use servo::base::generic_channel::GenericSender;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::{
    AlertResponse, AuthenticationRequest, ColorPicker, ConfirmResponse, EmbedderControlId,
    FilePicker, PermissionRequest, PromptResponse, RgbColor, SelectElement, SelectElementOption,
    SelectElementOptionOrOptgroup, SimpleDialog, WebDriverUserPrompt,
};

#[allow(clippy::large_enum_variant)]
pub enum Dialog {
    File {
        dialog: EguiFileDialog,
        maybe_picker: Option<FilePicker>,
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
        response_sender: GenericSender<Option<String>>,
    },
    SelectElement {
        maybe_prompt: Option<SelectElement>,
        toolbar_offset: Length<f32, DeviceIndependentPixel>,
    },
    ColorPicker {
        current_color: egui::Color32,
        maybe_prompt: Option<ColorPicker>,
        toolbar_offset: Length<f32, DeviceIndependentPixel>,
    },
}

impl Dialog {
    pub fn new_file_dialog(file_picker: FilePicker) -> Self {
        let mut dialog = EguiFileDialog::new();
        if !file_picker.filter_patterns().is_empty() {
            let filter_patterns = file_picker.filter_patterns().to_owned();
            dialog = dialog
                .add_file_filter(
                    "All Supported Types",
                    Arc::new(move |path: &Path| {
                        path.extension()
                            .and_then(|e| e.to_str())
                            .is_some_and(|ext| {
                                let ext = ext.to_lowercase();
                                filter_patterns.iter().any(|pattern| ext == pattern.0)
                            })
                    }),
                )
                .default_file_filter("All Supported Types");
        }

        Dialog::File {
            dialog,
            maybe_picker: Some(file_picker),
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
        response_sender: GenericSender<Option<String>>,
    ) -> Self {
        Dialog::SelectDevice {
            devices,
            selected_device_index: 0,
            response_sender,
        }
    }

    pub fn new_select_element_dialog(
        prompt: SelectElement,
        toolbar_offset: Length<f32, DeviceIndependentPixel>,
    ) -> Self {
        Dialog::SelectElement {
            maybe_prompt: Some(prompt),
            toolbar_offset,
        }
    }

    pub fn new_color_picker_dialog(
        prompt: ColorPicker,
        toolbar_offset: Length<f32, DeviceIndependentPixel>,
    ) -> Self {
        let current_color = prompt
            .current_color()
            .map(|color| egui::Color32::from_rgb(color.red, color.green, color.blue))
            .unwrap_or_default();
        Dialog::ColorPicker {
            current_color,
            maybe_prompt: Some(prompt),
            toolbar_offset,
        }
    }

    pub fn accept(&self) {
        #[allow(clippy::single_match)]
        match self {
            Dialog::SimpleDialog(dialog) => {
                dialog.accept();
            },
            _ => {},
        }
    }

    pub fn dismiss(&self) {
        #[allow(clippy::single_match)]
        match self {
            Dialog::SimpleDialog(dialog) => {
                dialog.dismiss();
            },
            _ => {},
        }
    }

    pub fn message(&self) -> Option<String> {
        #[allow(clippy::single_match)]
        match self {
            Dialog::SimpleDialog(dialog) => Some(dialog.message().to_string()),
            _ => None,
        }
    }

    pub fn set_message(&mut self, text: String) {
        if let Dialog::SimpleDialog(dialog) = self {
            dialog.set_message(text);
        }
    }

    /// Returns false if the dialog has been closed, or true otherwise.
    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        match self {
            Dialog::File {
                dialog,
                maybe_picker,
            } => {
                enum SelectFilesAction {
                    Dismiss,
                    Submit,
                    Continue,
                }

                let action = maybe_picker
                    .as_mut()
                    .map(|picker| {
                        if dialog.state() == DialogState::Closed {
                            if picker.allow_select_multiple() {
                                dialog.pick_multiple();
                            } else {
                                dialog.pick_file();
                            }
                        }

                        let state = dialog.update(ctx).state();
                        match state {
                            DialogState::Open => SelectFilesAction::Continue,
                            DialogState::Picked(path) => {
                                picker.select(&[path]);
                                SelectFilesAction::Submit
                            },
                            DialogState::PickedMultiple(paths) => {
                                picker.select(&paths);
                                SelectFilesAction::Submit
                            },
                            DialogState::Cancelled | DialogState::Closed => {
                                SelectFilesAction::Dismiss
                            },
                        }
                    })
                    .unwrap_or(SelectFilesAction::Dismiss);

                match action {
                    SelectFilesAction::Dismiss => {
                        if let Some(picker) = maybe_picker.take() {
                            picker.dismiss();
                        }
                    },
                    SelectFilesAction::Submit => {
                        if let Some(picker) = maybe_picker.take() {
                            picker.submit();
                        }
                    },
                    SelectFilesAction::Continue => {},
                }
                matches!(action, SelectFilesAction::Continue)
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
                            if ui.button("Close").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
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
                            if ui.button("Ok").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                is_open = false;
                                if let Err(e) = response_sender.send(ConfirmResponse::Ok) {
                                    warn!("Failed to send alert dialog response: {}", e);
                                }
                            }
                            if ui.button("Cancel").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
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
                            if ui.button("Ok").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                is_open = false;
                                if let Err(e) =
                                    response_sender.send(PromptResponse::Ok(input.clone()))
                                {
                                    warn!("Failed to send input dialog response: {}", e);
                                }
                            }
                            if ui.button("Cancel").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
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
                request,
            } => {
                let mut is_open = true;
                Modal::new("authentication".into()).show(ctx, |ui| {
                    let mut frame = egui::Frame::default().inner_margin(10.0).begin(ui);
                    frame.content_ui.set_min_width(150.0);

                    if let Some(request) = request {
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
                            if ui.button("Sign in").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                let request =
                                    request.take().expect("non-None until dialog is closed");
                                request.authenticate(username.clone(), password.clone());
                                is_open = false;
                            }
                            if ui.button("Cancel").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
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
                            if ui.button("Allow").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                let request =
                                    request.take().expect("non-None until dialog is closed");
                                request.allow();
                                is_open = false;
                            }
                            if ui.button("Deny").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
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
                            if ui.button("Ok").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                if let Err(e) = response_sender
                                    .send(Some(devices[*selected_device_index].clone()))
                                {
                                    warn!("Failed to send device selection: {}", e);
                                }
                                is_open = false;
                            }
                            if ui.button("Cancel").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
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
            Dialog::SelectElement {
                maybe_prompt,
                toolbar_offset,
            } => {
                let Some(prompt) = maybe_prompt else {
                    // Prompt was dismissed, so the dialog should be closed too.
                    return false;
                };
                let mut is_open = true;

                let mut position = prompt.position();
                position.min.y += toolbar_offset.0 as i32;
                position.max.y += toolbar_offset.0 as i32;
                let area = egui::Area::new(egui::Id::new("select-window"))
                    .fixed_pos(egui::pos2(position.min.x as f32, position.max.y as f32));

                let mut selected_option = prompt.selected_option();

                fn display_option(
                    ui: &mut egui::Ui,
                    option: &SelectElementOption,
                    selected_option: &mut Option<usize>,
                    is_open: &mut bool,
                    in_group: bool,
                ) {
                    let is_checked =
                        selected_option.is_some_and(|selected_index| selected_index == option.id);

                    // TODO: Surely there's a better way to align text in a selectable label in egui.
                    let label_text = if in_group {
                        format!("   {}", option.label)
                    } else {
                        option.label.to_owned()
                    };
                    let label = if option.is_disabled {
                        egui::RichText::new(&label_text).strikethrough()
                    } else {
                        egui::RichText::new(&label_text)
                    };
                    let clickable_area = ui
                        .allocate_ui_with_layout(
                            [ui.available_width(), 0.0].into(),
                            egui::Layout::top_down_justified(egui::Align::LEFT),
                            |ui| ui.selectable_label(is_checked, label),
                        )
                        .inner;

                    if clickable_area.clicked() && !option.is_disabled {
                        *selected_option = Some(option.id);
                        *is_open = false;
                    }

                    if clickable_area.hovered() && option.is_disabled {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
                    }

                    if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
                        *is_open = false;
                    }
                }

                let modal = Modal::new("select_element_picker".into()).area(area);
                let backdrop_response = modal
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for option_or_optgroup in prompt.options() {
                                match &option_or_optgroup {
                                    SelectElementOptionOrOptgroup::Option(option) => {
                                        display_option(
                                            ui,
                                            option,
                                            &mut selected_option,
                                            &mut is_open,
                                            false,
                                        );
                                    },
                                    SelectElementOptionOrOptgroup::Optgroup { label, options } => {
                                        ui.label(egui::RichText::new(label).strong());

                                        for option in options {
                                            display_option(
                                                ui,
                                                option,
                                                &mut selected_option,
                                                &mut is_open,
                                                true,
                                            );
                                        }
                                    },
                                }
                            }
                        });
                    })
                    .backdrop_response;

                // FIXME: Doesn't update until you move your mouse or press a key - why?
                if backdrop_response.clicked() {
                    is_open = false;
                }

                prompt.select(selected_option);

                if !is_open {
                    maybe_prompt.take().unwrap().submit();
                }

                is_open
            },
            Dialog::ColorPicker {
                current_color,
                maybe_prompt,
                toolbar_offset,
            } => {
                let Some(prompt) = maybe_prompt else {
                    // Prompt was dismissed, so the dialog should be closed too.
                    return false;
                };
                let mut is_open = true;

                let mut position = prompt.position();
                position.min.y += toolbar_offset.0 as i32;
                position.max.y += toolbar_offset.0 as i32;
                let area = egui::Area::new(egui::Id::new("select-window"))
                    .fixed_pos(egui::pos2(position.min.x as f32, position.max.y as f32));

                let modal = Modal::new("select_element_picker".into()).area(area);
                let backdrop_response = modal
                    .show(ctx, |ui| {
                        egui::widgets::color_picker::color_picker_color32(
                            ui,
                            current_color,
                            egui::widgets::color_picker::Alpha::Opaque,
                        );

                        ui.add_space(10.);

                        if ui.button("Dismiss").clicked() ||
                            ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            is_open = false;
                            prompt.select(None);
                        }
                        if ui.button("Select").clicked() {
                            is_open = false;
                            let selected_color = RgbColor {
                                red: current_color.r(),
                                green: current_color.g(),
                                blue: current_color.b(),
                            };
                            prompt.select(Some(selected_color));
                        }
                    })
                    .backdrop_response;

                // FIXME: Doesn't update until you move your mouse or press a key - why?
                if backdrop_response.clicked() {
                    is_open = false;
                }

                is_open
            },
        }
    }

    pub fn webdriver_diaglog_type(&self) -> WebDriverUserPrompt {
        match self {
            Dialog::File { .. } => WebDriverUserPrompt::File,
            Dialog::SimpleDialog(SimpleDialog::Alert { .. }) => WebDriverUserPrompt::Alert,
            Dialog::SimpleDialog(SimpleDialog::Confirm { .. }) => WebDriverUserPrompt::Confirm,
            Dialog::SimpleDialog(SimpleDialog::Prompt { .. }) => WebDriverUserPrompt::Prompt,
            _ => WebDriverUserPrompt::Default,
        }
    }

    pub(crate) fn embedder_control_id(&self) -> Option<EmbedderControlId> {
        match self {
            Dialog::SelectElement { maybe_prompt, .. } => {
                maybe_prompt.as_ref().map(|element| element.id())
            },
            Dialog::ColorPicker { maybe_prompt, .. } => {
                maybe_prompt.as_ref().map(|element| element.id())
            },
            _ => None,
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
