/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use nsstring::nsCString;
use servo_arc::Arc;
use style::context::QuirksMode;
use style::error_reporting::NullReporter;
use style::gecko::data::GeckoStyleSheet;
use style::gecko::global_style_data::GLOBAL_STYLE_DATA;
use style::gecko_bindings::bindings;
use style::gecko_bindings::bindings::Gecko_LoadStyleSheet;
use style::gecko_bindings::structs::{Loader, LoaderReusableStyleSheets};
use style::gecko_bindings::structs::{ServoStyleSheet, SheetLoadData, SheetLoadDataHolder};
use style::gecko_bindings::structs::URLExtraData;
use style::gecko_bindings::sugar::ownership::FFIArcHelpers;
use style::gecko_bindings::sugar::refptr::RefPtr;
use style::media_queries::MediaList;
use style::parser::ParserContext;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::{ImportRule, Origin, StylesheetLoader as StyleStylesheetLoader};
use style::stylesheets::StylesheetContents;
use style::stylesheets::import_rule::ImportSheet;
use style::values::CssUrl;

pub struct StylesheetLoader(*mut Loader, *mut ServoStyleSheet, *mut SheetLoadData, *mut LoaderReusableStyleSheets);

impl StylesheetLoader {
    pub fn new(loader: *mut Loader,
               parent: *mut ServoStyleSheet,
               parent_load_data: *mut SheetLoadData,
               reusable_sheets: *mut LoaderReusableStyleSheets) -> Self {
        StylesheetLoader(loader, parent, parent_load_data, reusable_sheets)
    }
}

impl StyleStylesheetLoader for StylesheetLoader {
    fn request_stylesheet(
        &self,
        url: CssUrl,
        source_location: SourceLocation,
        _context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
    ) -> Arc<Locked<ImportRule>> {
        // After we get this raw pointer ImportRule will be moved into a lock and Arc
        // and so the Arc<Url> pointer inside will also move,
        // but the Url it points to or the allocating backing the String inside that Url won’t,
        // so this raw pointer will still be valid.

        let child_sheet = unsafe {
            Gecko_LoadStyleSheet(self.0,
                                 self.1,
                                 self.2,
                                 self.3,
                                 url.for_ffi(),
                                 media.into_strong())
        };

        debug_assert!(!child_sheet.is_null(),
                      "Import rules should always have a strong sheet");
        let sheet = unsafe { GeckoStyleSheet::from_addrefed(child_sheet) };
        let stylesheet = ImportSheet::new(sheet);
        Arc::new(lock.wrap(ImportRule { url, source_location, stylesheet }))
    }
}

pub struct AsyncStylesheetParser {
    load_data: RefPtr<SheetLoadDataHolder>,
    extra_data: RefPtr<URLExtraData>,
    bytes: nsCString,
    origin: Origin,
    quirks_mode: QuirksMode,
    line_number_offset: u32,
}

impl AsyncStylesheetParser {
    pub fn new(
        load_data: RefPtr<SheetLoadDataHolder>,
        extra_data: RefPtr<URLExtraData>,
        bytes: nsCString,
        origin: Origin,
        quirks_mode: QuirksMode,
        line_number_offset: u32,
    ) -> Self {
        AsyncStylesheetParser {
            load_data,
            extra_data,
            bytes,
            origin,
            quirks_mode,
            line_number_offset,
        }
    }

    pub fn parse(self) {
        let global_style_data = &*GLOBAL_STYLE_DATA;
        let input: &str = unsafe { (*self.bytes).as_str_unchecked() };

        // Note: Parallel CSS parsing doesn't report CSS errors. When errors
        // are being logged, Gecko prevents the parallel parsing path from
        // running.
        let sheet = Arc::new(StylesheetContents::from_str(
            input, self.extra_data.clone(), self.origin,
            &global_style_data.shared_lock, Some(&self), &NullReporter,
            self.quirks_mode.into(), self.line_number_offset)
        );

        unsafe {
            bindings::Gecko_StyleSheet_FinishAsyncParse(self.load_data.get(), sheet.into_strong());
        }
    }
}

impl StyleStylesheetLoader for AsyncStylesheetParser {
    fn request_stylesheet(
        &self,
        url: CssUrl,
        source_location: SourceLocation,
        _context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
    ) -> Arc<Locked<ImportRule>> {
        let stylesheet = ImportSheet::new_pending(self.origin, self.quirks_mode);
        let rule = Arc::new(lock.wrap(ImportRule { url: url.clone(), source_location, stylesheet }));

        unsafe {
            bindings::Gecko_LoadStyleSheetAsync(
                self.load_data.get(),
                url.for_ffi(),
                media.into_strong(),
                rule.clone().into_strong()
            );
        }

        rule
    }
}
