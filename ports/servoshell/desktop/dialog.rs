/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::sync::Arc;

use egui::{
    Area, Button, CornerRadius, Frame, Id, Modal, Order, RichText, Sense, Stroke, Vec2, pos2,
};
use egui_file_dialog::{DialogState, FileDialog as EguiFileDialog};
use euclid::Length;
use log::warn;
use servo::{
    AlertDialog, AuthenticationRequest, ColorPicker, ConfirmDialog, ContextMenu, ContextMenuItem,
    DeviceIndependentPixel, EmbedderControlId, FilePicker, GenericSender, PermissionRequest,
    PromptDialog, RgbColor, SelectElement, SelectElementOption, SelectElementOptionOrOptgroup,
    SimpleDialog,
};

/// The minimum width of many UI elements including dialog boxes and menus,
/// for the sake of consistency.
const MINIMUM_UI_ELEMENT_WIDTH: f32 = 150.0;

#[expect(clippy::large_enum_variant)]
pub enum Dialog {
    File {
        dialog: EguiFileDialog,
        maybe_picker: Option<FilePicker>,
    },
    Alert(Option<AlertDialog>),
    Confirm(Option<ConfirmDialog>),
    Prompt(Option<PromptDialog>),
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
    ContextMenu {
        menu: Option<ContextMenu>,
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
        match dialog {
            SimpleDialog::Alert(alert_dialog) => Self::Alert(Some(alert_dialog)),
            SimpleDialog::Confirm(confirm_dialog) => Self::Confirm(Some(confirm_dialog)),
            SimpleDialog::Prompt(prompt_dialog) => Self::Prompt(Some(prompt_dialog)),
        }
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

    /// Returns false if the dialog has been closed, or true otherwise.
    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        enum DialogAction {
            Dismiss,
            Submit,
            Continue,
        }

        match self {
            Dialog::File {
                dialog,
                maybe_picker,
            } => {
                let action = maybe_picker
                    .as_mut()
                    .map(|picker| {
                        if *dialog.state() == DialogState::Closed {
                            if picker.allow_select_multiple() {
                                dialog.pick_multiple();
                            } else {
                                dialog.pick_file();
                            }
                        }

                        let state = dialog.update(ctx).state();
                        match state {
                            DialogState::Open => DialogAction::Continue,
                            DialogState::Picked(path) => {
                                let paths = std::slice::from_ref(path);
                                picker.select(paths);
                                DialogAction::Submit
                            },
                            DialogState::PickedMultiple(paths) => {
                                picker.select(paths);
                                DialogAction::Submit
                            },
                            DialogState::Cancelled | DialogState::Closed => DialogAction::Dismiss,
                        }
                    })
                    .unwrap_or(DialogAction::Dismiss);

                match action {
                    DialogAction::Dismiss => {
                        if let Some(picker) = maybe_picker.take() {
                            picker.dismiss();
                        }
                    },
                    DialogAction::Submit => {
                        if let Some(picker) = maybe_picker.take() {
                            picker.submit();
                        }
                    },
                    DialogAction::Continue => {},
                }
                matches!(action, DialogAction::Continue)
            },
            Dialog::Alert(maybe_alert_dialog) => {
                let Some(alert_dialog) = maybe_alert_dialog else {
                    return false;
                };

                let mut is_open = true;
                Modal::new("Alert".into()).show(ctx, |ui| {
                    make_dialog_label(alert_dialog.message(), ui, None);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Close").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                is_open = false;
                            }
                        },
                    );
                });

                if !is_open
                    && let Some(alert_dialog) = maybe_alert_dialog.take() {
                        alert_dialog.confirm();
                    }
                is_open
            },
            Dialog::Confirm(maybe_confirm_dialog) => {
                let Some(confirm_dialog) = maybe_confirm_dialog else {
                    return false;
                };

                let mut dialog_action = DialogAction::Continue;
                Modal::new("Confirm".into()).show(ctx, |ui| {
                    make_dialog_label(confirm_dialog.message(), ui, None);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Ok").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                dialog_action = DialogAction::Submit;
                            }
                            if ui.button("Cancel").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                dialog_action = DialogAction::Dismiss;
                            }
                        },
                    );
                });

                match dialog_action {
                    DialogAction::Dismiss => {
                        if let Some(confirm_dialog) = maybe_confirm_dialog.take() {
                            confirm_dialog.dismiss();
                        }
                        false
                    },
                    DialogAction::Submit => {
                        if let Some(confirm_dialog) = maybe_confirm_dialog.take() {
                            confirm_dialog.confirm();
                        }
                        false
                    },
                    DialogAction::Continue => true,
                }
            },
            Dialog::Prompt(maybe_prompt_dialog) => {
                let Some(prompt_dialog) = maybe_prompt_dialog else {
                    return false;
                };

                let mut dialog_action = DialogAction::Continue;
                Modal::new("Prompt".into()).show(ctx, |ui| {
                    let mut prompt_text = prompt_dialog.current_value().to_owned();
                    make_dialog_label(prompt_dialog.message(), ui, Some(&mut prompt_text));
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Ok").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                prompt_dialog.set_current_value(&prompt_text);
                                dialog_action = DialogAction::Submit;
                            }
                            if ui.button("Cancel").clicked() ||
                                ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                dialog_action = DialogAction::Dismiss;
                            }
                        },
                    );
                    prompt_dialog.set_current_value(&prompt_text);
                });
                match dialog_action {
                    DialogAction::Dismiss => {
                        if let Some(prompt_dialog) = maybe_prompt_dialog.take() {
                            prompt_dialog.dismiss();
                        }
                        false
                    },
                    DialogAction::Submit => {
                        if let Some(prompt_dialog) = maybe_prompt_dialog.take() {
                            prompt_dialog.confirm();
                        }
                        false
                    },
                    DialogAction::Continue => true,
                }
            },
            Dialog::Authentication {
                username,
                password,
                request,
            } => {
                let mut is_open = true;
                Modal::new("authentication".into()).show(ctx, |ui| {
                    let mut frame = egui::Frame::default().inner_margin(10.0).begin(ui);
                    frame.content_ui.set_min_width(MINIMUM_UI_ELEMENT_WIDTH);

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
                    frame.content_ui.set_min_width(MINIMUM_UI_ELEMENT_WIDTH);

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
                                        ui.label(RichText::new(label).strong());

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
            Dialog::ContextMenu {
                menu,
                toolbar_offset,
            } => {
                let mut is_open = true;
                if let Some(context_menu) = menu {
                    let mut selected_action = None;
                    let mut position = context_menu.position();
                    position.min.y += toolbar_offset.0 as i32;
                    position.max.y += toolbar_offset.0 as i32;

                    let response = Area::new(Id::new("context_menu"))
                        .fixed_pos(pos2(position.min.x as f32, position.min.y as f32))
                        .order(Order::Foreground)
                        .show(ctx, |ui| {
                            Frame::popup(ui.style()).show(ui, |ui| {
                                ui.set_min_width(MINIMUM_UI_ELEMENT_WIDTH);
                                for item in context_menu.items() {
                                    match item {
                                        ContextMenuItem::Item {
                                            label,
                                            action,
                                            enabled,
                                        } => {
                                            let (color, sense) = match enabled {
                                                true => (
                                                    ui.visuals().strong_text_color(),
                                                    Sense::click(),
                                                ),
                                                false => {
                                                    (ui.visuals().weak_text_color(), Sense::empty())
                                                },
                                            };

                                            ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                                ui.visuals().panel_fill;
                                            ui.style_mut().visuals.widgets.inactive.bg_fill =
                                                ui.visuals().panel_fill;
                                            let button =
                                                Button::new(RichText::new(label).color(color))
                                                    .sense(sense)
                                                    .corner_radius(CornerRadius::ZERO)
                                                    .stroke(Stroke::NONE)
                                                    .wrap_mode(egui::TextWrapMode::Extend)
                                                    .min_size(Vec2 {
                                                        x: MINIMUM_UI_ELEMENT_WIDTH,
                                                        y: 0.0,
                                                    });

                                            if ui.add(button).clicked() {
                                                selected_action = Some(*action);
                                                ui.close();
                                            }
                                        },
                                        ContextMenuItem::Separator => {
                                            ui.separator();
                                        },
                                    }
                                }
                            })
                        });

                    if response.response.clicked_elsewhere() {
                        is_open = false;
                    }

                    if let Some(action) = selected_action
                        && let Some(context_menu) = menu.take() {
                            context_menu.select(action);
                            return false;
                        }
                }
                is_open
            },
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

    pub(crate) fn new_context_menu(
        menu: ContextMenu,
        toolbar_offset: Length<f32, DeviceIndependentPixel>,
    ) -> Dialog {
        Dialog::ContextMenu {
            menu: Some(menu),
            toolbar_offset,
        }
    }
}

fn make_dialog_label(message: &str, ui: &mut egui::Ui, input_text: Option<&mut String>) {
    let mut frame = egui::Frame::default().inner_margin(10.0).begin(ui);
    frame.content_ui.set_min_width(MINIMUM_UI_ELEMENT_WIDTH);
    frame.content_ui.label(message);
    if let Some(input_text) = input_text {
        frame.content_ui.text_edit_singleline(input_text);
    }
    frame.end(ui);
}
