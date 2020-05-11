/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

mod counter_style_rule;
mod document_rule;
mod font_face_rule;
pub mod font_feature_values_rule;
pub mod import_rule;
pub mod keyframes_rule;
mod loader;
mod media_rule;
mod namespace_rule;
pub mod origin;
mod page_rule;
mod rule_list;
mod rule_parser;
mod rules_iterator;
mod style_rule;
mod stylesheet;
pub mod supports_rule;
pub mod viewport_rule;

#[cfg(feature = "gecko")]
use crate::gecko_bindings::sugar::refptr::RefCounted;
#[cfg(feature = "gecko")]
use crate::gecko_bindings::{bindings, structs};
use crate::parser::ParserContext;
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use cssparser::{parse_one_rule, Parser, ParserInput};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt;
#[cfg(feature = "gecko")]
use std::mem::{self, ManuallyDrop};
use style_traits::ParsingMode;
#[cfg(feature = "gecko")]
use to_shmem::{self, SharedMemoryBuilder, ToShmem};

pub use self::counter_style_rule::CounterStyleRule;
pub use self::document_rule::DocumentRule;
pub use self::font_face_rule::FontFaceRule;
pub use self::font_feature_values_rule::FontFeatureValuesRule;
pub use self::import_rule::ImportRule;
pub use self::keyframes_rule::KeyframesRule;
pub use self::loader::StylesheetLoader;
pub use self::media_rule::MediaRule;
pub use self::namespace_rule::NamespaceRule;
pub use self::origin::{Origin, OriginSet, OriginSetIterator, PerOrigin, PerOriginIter};
pub use self::page_rule::PageRule;
pub use self::rule_list::{CssRules, CssRulesHelpers};
pub use self::rule_parser::{InsertRuleContext, State, TopLevelRuleParser};
pub use self::rules_iterator::{AllRules, EffectiveRules};
pub use self::rules_iterator::{NestedRuleIterationCondition, RulesIterator};
pub use self::style_rule::StyleRule;
pub use self::stylesheet::{AllowImportRules, SanitizationData, SanitizationKind};
pub use self::stylesheet::{DocumentStyleSheet, Namespaces, Stylesheet};
pub use self::stylesheet::{StylesheetContents, StylesheetInDocument, UserAgentStylesheets};
pub use self::supports_rule::SupportsRule;
pub use self::viewport_rule::ViewportRule;

/// The CORS mode used for a CSS load.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToShmem)]
pub enum CorsMode {
    /// No CORS mode, so cross-origin loads can be done.
    None,
    /// Anonymous CORS request.
    Anonymous,
}

/// Extra data that the backend may need to resolve url values.
///
/// If the usize's lowest bit is 0, then this is a strong reference to a
/// structs::URLExtraData object.
///
/// Otherwise, shifting the usize's bits the right by one gives the
/// UserAgentStyleSheetID value corresponding to the style sheet whose
/// URLExtraData this is, which is stored in URLExtraData_sShared.  We don't
/// hold a strong reference to that object from here, but we rely on that
/// array's objects being held alive until shutdown.
///
/// We use this packed representation rather than an enum so that
/// `from_ptr_ref` can work.
#[cfg(feature = "gecko")]
#[derive(PartialEq)]
#[repr(C)]
pub struct UrlExtraData(usize);

/// Extra data that the backend may need to resolve url values.
#[cfg(not(feature = "gecko"))]
pub type UrlExtraData = ::servo_url::ServoUrl;

#[cfg(feature = "gecko")]
impl Clone for UrlExtraData {
    fn clone(&self) -> UrlExtraData {
        UrlExtraData::new(self.ptr())
    }
}

#[cfg(feature = "gecko")]
impl Drop for UrlExtraData {
    fn drop(&mut self) {
        // No need to release when we have an index into URLExtraData_sShared.
        if self.0 & 1 == 0 {
            unsafe {
                self.as_ref().release();
            }
        }
    }
}

#[cfg(feature = "gecko")]
impl ToShmem for UrlExtraData {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> to_shmem::Result<Self> {
        if self.0 & 1 == 0 {
            let shared_extra_datas = unsafe { &structs::URLExtraData_sShared };
            let self_ptr = self.as_ref() as *const _ as *mut _;
            let sheet_id = shared_extra_datas
                .iter()
                .position(|r| r.mRawPtr == self_ptr);
            let sheet_id = match sheet_id {
                Some(id) => id,
                None => {
                    return Err(String::from(
                        "ToShmem failed for UrlExtraData: expected sheet's URLExtraData to be in \
                         URLExtraData::sShared",
                    ));
                },
            };
            Ok(ManuallyDrop::new(UrlExtraData((sheet_id << 1) | 1)))
        } else {
            Ok(ManuallyDrop::new(UrlExtraData(self.0)))
        }
    }
}

#[cfg(feature = "gecko")]
impl UrlExtraData {
    /// Create a new UrlExtraData wrapping a pointer to the specified Gecko
    /// URLExtraData object.
    pub fn new(ptr: *mut structs::URLExtraData) -> UrlExtraData {
        unsafe {
            (*ptr).addref();
        }
        UrlExtraData(ptr as usize)
    }

    /// True if this URL scheme is chrome.
    #[inline]
    pub fn is_chrome(&self) -> bool {
        self.as_ref().mIsChrome
    }

    /// Create a reference to this `UrlExtraData` from a reference to pointer.
    ///
    /// The pointer must be valid and non null.
    ///
    /// This method doesn't touch refcount.
    #[inline]
    pub unsafe fn from_ptr_ref(ptr: &*mut structs::URLExtraData) -> &Self {
        mem::transmute(ptr)
    }

    /// Returns a pointer to the Gecko URLExtraData object.
    pub fn ptr(&self) -> *mut structs::URLExtraData {
        if self.0 & 1 == 0 {
            self.0 as *mut structs::URLExtraData
        } else {
            unsafe {
                let sheet_id = self.0 >> 1;
                structs::URLExtraData_sShared[sheet_id].mRawPtr
            }
        }
    }

    fn as_ref(&self) -> &structs::URLExtraData {
        unsafe { &*(self.ptr() as *const structs::URLExtraData) }
    }
}

#[cfg(feature = "gecko")]
impl fmt::Debug for UrlExtraData {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        macro_rules! define_debug_struct {
            ($struct_name:ident, $gecko_class:ident, $debug_fn:ident) => {
                struct $struct_name(*mut structs::$gecko_class);
                impl fmt::Debug for $struct_name {
                    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        use nsstring::nsCString;
                        let mut spec = nsCString::new();
                        unsafe {
                            bindings::$debug_fn(self.0, &mut spec);
                        }
                        spec.fmt(formatter)
                    }
                }
            };
        }

        define_debug_struct!(DebugURI, nsIURI, Gecko_nsIURI_Debug);
        define_debug_struct!(
            DebugReferrerInfo,
            nsIReferrerInfo,
            Gecko_nsIReferrerInfo_Debug
        );

        formatter
            .debug_struct("URLExtraData")
            .field("is_chrome", &self.is_chrome())
            .field(
                "base",
                &DebugURI(self.as_ref().mBaseURI.raw::<structs::nsIURI>()),
            )
            .field(
                "referrer",
                &DebugReferrerInfo(
                    self.as_ref()
                        .mReferrerInfo
                        .raw::<structs::nsIReferrerInfo>(),
                ),
            )
            .finish()
    }
}

// XXX We probably need to figure out whether we should mark Eq here.
// It is currently marked so because properties::UnparsedValue wants Eq.
#[cfg(feature = "gecko")]
impl Eq for UrlExtraData {}

/// A CSS rule.
///
/// TODO(emilio): Lots of spec links should be around.
#[derive(Clone, Debug, ToShmem)]
#[allow(missing_docs)]
pub enum CssRule {
    // No Charset here, CSSCharsetRule has been removed from CSSOM
    // https://drafts.csswg.org/cssom/#changes-from-5-december-2013
    Namespace(Arc<Locked<NamespaceRule>>),
    Import(Arc<Locked<ImportRule>>),
    Style(Arc<Locked<StyleRule>>),
    Media(Arc<Locked<MediaRule>>),
    FontFace(Arc<Locked<FontFaceRule>>),
    FontFeatureValues(Arc<Locked<FontFeatureValuesRule>>),
    CounterStyle(Arc<Locked<CounterStyleRule>>),
    Viewport(Arc<Locked<ViewportRule>>),
    Keyframes(Arc<Locked<KeyframesRule>>),
    Supports(Arc<Locked<SupportsRule>>),
    Page(Arc<Locked<PageRule>>),
    Document(Arc<Locked<DocumentRule>>),
}

impl CssRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            // Not all fields are currently fully measured. Extra measurement
            // may be added later.
            CssRule::Namespace(_) => 0,

            // We don't need to measure ImportRule::stylesheet because we measure
            // it on the C++ side in the child list of the ServoStyleSheet.
            CssRule::Import(_) => 0,

            CssRule::Style(ref lock) => {
                lock.unconditional_shallow_size_of(ops) + lock.read_with(guard).size_of(guard, ops)
            },

            CssRule::Media(ref lock) => {
                lock.unconditional_shallow_size_of(ops) + lock.read_with(guard).size_of(guard, ops)
            },

            CssRule::FontFace(_) => 0,
            CssRule::FontFeatureValues(_) => 0,
            CssRule::CounterStyle(_) => 0,
            CssRule::Viewport(_) => 0,
            CssRule::Keyframes(_) => 0,

            CssRule::Supports(ref lock) => {
                lock.unconditional_shallow_size_of(ops) + lock.read_with(guard).size_of(guard, ops)
            },

            CssRule::Page(ref lock) => {
                lock.unconditional_shallow_size_of(ops) + lock.read_with(guard).size_of(guard, ops)
            },

            CssRule::Document(ref lock) => {
                lock.unconditional_shallow_size_of(ops) + lock.read_with(guard).size_of(guard, ops)
            },
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssRuleType {
    // https://drafts.csswg.org/cssom/#the-cssrule-interface
    Style = 1,
    Charset = 2,
    Import = 3,
    Media = 4,
    FontFace = 5,
    Page = 6,
    // https://drafts.csswg.org/css-animations-1/#interface-cssrule-idl
    Keyframes = 7,
    Keyframe = 8,
    // https://drafts.csswg.org/cssom/#the-cssrule-interface
    Margin = 9,
    Namespace = 10,
    // https://drafts.csswg.org/css-counter-styles-3/#extentions-to-cssrule-interface
    CounterStyle = 11,
    // https://drafts.csswg.org/css-conditional-3/#extentions-to-cssrule-interface
    Supports = 12,
    // https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#extentions-to-cssrule-interface
    Document = 13,
    // https://drafts.csswg.org/css-fonts-3/#om-fontfeaturevalues
    FontFeatureValues = 14,
    // https://drafts.csswg.org/css-device-adapt/#css-rule-interface
    Viewport = 15,
}

#[allow(missing_docs)]
pub enum RulesMutateError {
    Syntax,
    IndexSize,
    HierarchyRequest,
    InvalidState,
}

impl CssRule {
    /// Returns the CSSOM rule type of this rule.
    pub fn rule_type(&self) -> CssRuleType {
        match *self {
            CssRule::Style(_) => CssRuleType::Style,
            CssRule::Import(_) => CssRuleType::Import,
            CssRule::Media(_) => CssRuleType::Media,
            CssRule::FontFace(_) => CssRuleType::FontFace,
            CssRule::FontFeatureValues(_) => CssRuleType::FontFeatureValues,
            CssRule::CounterStyle(_) => CssRuleType::CounterStyle,
            CssRule::Keyframes(_) => CssRuleType::Keyframes,
            CssRule::Namespace(_) => CssRuleType::Namespace,
            CssRule::Viewport(_) => CssRuleType::Viewport,
            CssRule::Supports(_) => CssRuleType::Supports,
            CssRule::Page(_) => CssRuleType::Page,
            CssRule::Document(_) => CssRuleType::Document,
        }
    }

    fn rule_state(&self) -> State {
        match *self {
            // CssRule::Charset(..) => State::Start,
            CssRule::Import(..) => State::Imports,
            CssRule::Namespace(..) => State::Namespaces,
            _ => State::Body,
        }
    }

    /// Parse a CSS rule.
    ///
    /// Returns a parsed CSS rule and the final state of the parser.
    ///
    /// Input state is None for a nested rule
    pub fn parse(
        css: &str,
        insert_rule_context: InsertRuleContext,
        parent_stylesheet_contents: &StylesheetContents,
        shared_lock: &SharedRwLock,
        state: State,
        loader: Option<&dyn StylesheetLoader>,
        allow_import_rules: AllowImportRules,
    ) -> Result<Self, RulesMutateError> {
        let url_data = parent_stylesheet_contents.url_data.read();
        let context = ParserContext::new(
            parent_stylesheet_contents.origin,
            &url_data,
            None,
            ParsingMode::DEFAULT,
            parent_stylesheet_contents.quirks_mode,
            None,
            None,
        );

        let mut input = ParserInput::new(css);
        let mut input = Parser::new(&mut input);

        let mut guard = parent_stylesheet_contents.namespaces.write();

        // nested rules are in the body state
        let mut rule_parser = TopLevelRuleParser {
            context,
            shared_lock: &shared_lock,
            loader,
            state,
            dom_error: None,
            namespaces: &mut *guard,
            insert_rule_context: Some(insert_rule_context),
            allow_import_rules,
        };

        parse_one_rule(&mut input, &mut rule_parser)
            .map_err(|_| rule_parser.dom_error.unwrap_or(RulesMutateError::Syntax))
    }
}

impl DeepCloneWithLock for CssRule {
    /// Deep clones this CssRule.
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> CssRule {
        match *self {
            CssRule::Namespace(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Namespace(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Import(ref arc) => {
                let rule = arc
                    .read_with(guard)
                    .deep_clone_with_lock(lock, guard, params);
                CssRule::Import(Arc::new(lock.wrap(rule)))
            },
            CssRule::Style(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Style(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
            CssRule::Media(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Media(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
            CssRule::FontFace(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::FontFace(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::FontFeatureValues(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::FontFeatureValues(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::CounterStyle(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::CounterStyle(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Viewport(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Viewport(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Keyframes(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Keyframes(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
            CssRule::Supports(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Supports(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
            CssRule::Page(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Page(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
            CssRule::Document(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Document(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
        }
    }
}

impl ToCssWithGuard for CssRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        match *self {
            CssRule::Namespace(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Import(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Style(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::FontFace(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::FontFeatureValues(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::CounterStyle(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Viewport(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Keyframes(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Media(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Supports(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Page(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Document(ref lock) => lock.read_with(guard).to_css(guard, dest),
        }
    }
}
