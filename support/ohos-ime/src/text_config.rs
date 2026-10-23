use std::num::TryFromIntError;
// use std::ptr::NonNull;
// use ohos_ime_sys::text_config::{InputMethod_TextConfig, OH_TextConfig_Create, OH_TextConfig_Destroy, OH_TextConfig_SetEnterKeyType, OH_TextConfig_SetInputType, OH_TextConfig_SetPreviewTextSupport, OH_TextConfig_SetSelection, OH_TextConfig_SetWindowId};
use ohos_ime_sys::types::{InputMethod_EnterKeyType, InputMethod_TextInputType};

#[derive(Clone)]
pub struct TextSelection {
    pub(crate) start: i32,
    pub(crate) end: i32,
}

pub struct InvalidSelection(());

impl From<TryFromIntError> for InvalidSelection {
    fn from(_: TryFromIntError) -> Self {
        InvalidSelection(())
    }
}

impl TextSelection {
    // Todo: Since we have utf-8 rust strings, but use utf-16 on the arkts side, do the indexes perhaps
    // need to be updated for cases where 2 utf-8 codepoints map to 1 utf-16 codepoint?
    // But we don't have any information about the string here, so I guess we would need to
    // impose this as a usage requirement on the user.
    /// Create a new Text Selection.
    pub fn new(start: usize, end: usize) -> Result<TextSelection, InvalidSelection> {
        Ok(TextSelection {
            start: start.try_into()?,
            end: end.try_into()?,
        })
    }
}

pub struct TextConfig {
    pub(crate) input_type: InputMethod_TextInputType,
    pub(crate) enterkey_type: InputMethod_EnterKeyType,
    pub(crate) preview_text_support: bool,
    pub(crate) selection: Option<TextSelection>,
    pub(crate) window_id: Option<i32>,
}

impl Default for TextConfig {
    fn default() -> TextConfig {
        TextConfigBuilder::new().build()
    }
}

pub struct TextConfigBuilder {
    input_type: InputMethod_TextInputType,
    enterkey_type: InputMethod_EnterKeyType,
    preview_text_support: bool,
    selection: Option<TextSelection>,
    window_id: Option<i32>,
}

impl TextConfigBuilder {
    pub const fn new() -> TextConfigBuilder {
        TextConfigBuilder {
            input_type: InputMethod_TextInputType::IME_TEXT_INPUT_TYPE_TEXT,
            enterkey_type: InputMethod_EnterKeyType::IME_ENTER_KEY_UNSPECIFIED,
            preview_text_support: false,
            selection: None,
            window_id: None,
        }
    }

    pub fn build(&self) -> TextConfig {
        TextConfig {
            // raw: config,
            input_type: self.input_type,
            enterkey_type: self.enterkey_type,
            preview_text_support: self.preview_text_support,
            selection: self.selection.clone(),
            window_id: self.window_id,
        }
    }

    pub fn input_type(mut self, input_type: InputMethod_TextInputType) -> TextConfigBuilder {
        self.input_type = input_type;
        self
    }

    pub fn enterkey_type(mut self, enterkey_type: InputMethod_EnterKeyType) -> TextConfigBuilder {
        self.enterkey_type = enterkey_type;
        self
    }

    pub fn preview_text_support(mut self, preview_text_support: bool) -> TextConfigBuilder {
        self.preview_text_support = preview_text_support;
        self
    }

    pub fn selection(mut self, selection: TextSelection) -> TextConfigBuilder {
        self.selection = Some(selection);
        self
    }

    pub fn window_id(mut self, window_id: i32) -> TextConfigBuilder {
        self.window_id = Some(window_id);
        self
    }
}

impl Default for TextConfigBuilder {
    fn default() -> TextConfigBuilder {
        Self::new()
    }
}
