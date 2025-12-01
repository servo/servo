/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;
use std::time::SystemTime;

use base::Epoch;
use base::generic_channel::GenericSender;
use base::id::{PipelineId, WebViewId};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{InputMethodType, RgbColor};
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct EmbedderControlId {
    pub webview_id: WebViewId,
    pub pipeline_id: PipelineId,
    pub index: Epoch,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EmbedderControlRequest {
    /// Indicates that the user has activated a `<select>` element.
    SelectElement(Vec<SelectElementOptionOrOptgroup>, Option<usize>),
    /// Indicates that the user has activated a `<input type=color>` element.
    ColorPicker(RgbColor),
    /// Indicates that the user has activated a `<input type=file>` element.
    FilePicker(FilePickerRequest),
    /// Indicates that the the user has activated a text or input control that should show
    /// an IME.
    InputMethod(InputMethodRequest),
    /// Indicates that the the user has triggered the display of a context menu.
    ContextMenu(ContextMenuRequest),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectElementOption {
    /// A unique identifier for the option that can be used to select it.
    pub id: usize,
    /// The label that should be used to display the option to the user.
    pub label: String,
    /// Whether or not the option is selectable
    pub is_disabled: bool,
}

/// Represents the contents of either an `<option>` or an `<optgroup>` element
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SelectElementOptionOrOptgroup {
    Option(SelectElementOption),
    Optgroup {
        label: String,
        options: Vec<SelectElementOption>,
    },
}

/// Request to present a context menu to the user. This is triggered by things like
/// right-clicking on web content.
#[derive(Debug, Deserialize, Serialize)]
pub struct ContextMenuRequest {
    pub element_info: ContextMenuElementInformation,
    pub items: Vec<ContextMenuItem>,
}

/// An item in a context menu.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ContextMenuItem {
    Item {
        label: String,
        action: ContextMenuAction,
        enabled: bool,
    },
    Separator,
}

/// A particular action associated with a [`ContextMenuItem`]. These actions are
/// context-sensitive, which means that some of them are available only for some
/// page elements.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ContextMenuAction {
    GoBack,
    GoForward,
    Reload,

    CopyLink,
    OpenLinkInNewWebView,

    CopyImageLink,
    OpenImageInNewView,

    Cut,
    Copy,
    Paste,
    SelectAll,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
    pub struct ContextMenuElementInformationFlags: u8 {
        /// Whether or not the element this context menu was activated for was a link.
        const Link = 1 << 1;
        /// Whether or not the element this context menu was activated for was an image.
        const Image = 1 << 2;
        /// Whether or not the element this context menu was activated for was editable
        /// text.
        const EditableText = 1 << 3;
        /// Whether or not the element this context menu was activated for was covered by
        /// a selection.
        const Selection = 1 << 4;
    }
}

/// Information about the element that a context menu was activated for. values which
/// do not apply to this element will be `None`.
///
/// Note that an element might be both an image and a link, if the element is an `<img>`
/// tag nested inside of a `<a>` tag.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ContextMenuElementInformation {
    pub flags: ContextMenuElementInformationFlags,
    pub link_url: Option<Url>,
    pub image_url: Option<Url>,
}

/// Request to present an IME to the user when an editable element is focused. If `type` is
/// [`InputMethodType::Text`], then the `text` parameter specifies the pre-existing text content and
/// `insertion_point` the zero-based index into the string of the insertion point.
#[derive(Debug, Deserialize, Serialize)]
pub struct InputMethodRequest {
    pub input_method_type: InputMethodType,
    pub text: String,
    pub insertion_point: Option<u32>,
    pub multiline: bool,
}

/// Filter for file selection;
/// the `String` content is expected to be extension (e.g, "doc", without the prefixing ".")
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterPattern(pub String);

#[derive(Debug, Deserialize, Serialize)]
pub struct FilePickerRequest {
    pub origin: String,
    pub current_paths: Vec<PathBuf>,
    pub filter_patterns: Vec<FilterPattern>,
    pub allow_select_multiple: bool,
    pub accept_current_paths_for_testing: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EmbedderControlResponse {
    SelectElement(Option<usize>),
    ColorPicker(Option<RgbColor>),
    FilePicker(Option<Vec<SelectedFile>>),
    ContextMenu(Option<ContextMenuAction>),
}

/// Response to file selection request
#[derive(Debug, Deserialize, Serialize)]
pub struct SelectedFile {
    pub id: Uuid,
    pub filename: PathBuf,
    pub modified: SystemTime,
    pub size: u64,
    // https://w3c.github.io/FileAPI/#dfn-type
    pub type_string: String,
}

#[derive(Deserialize, Serialize)]
pub enum SimpleDialogRequest {
    Alert {
        id: EmbedderControlId,
        message: String,
        response_sender: GenericSender<AlertResponse>,
    },
    Confirm {
        id: EmbedderControlId,
        message: String,
        response_sender: GenericSender<ConfirmResponse>,
    },
    Prompt {
        id: EmbedderControlId,
        message: String,
        default: String,
        response_sender: GenericSender<PromptResponse>,
    },
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum AlertResponse {
    Ok,
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum ConfirmResponse {
    Ok,
    Cancel,
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum PromptResponse {
    Ok(String),
    Cancel,
}
