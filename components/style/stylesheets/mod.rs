/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

mod cascading_at_rule;
pub mod container_rule;
mod counter_style_rule;
mod document_rule;
mod font_face_rule;
pub mod font_feature_values_rule;
pub mod font_palette_values_rule;
pub mod import_rule;
pub mod keyframes_rule;
pub mod layer_rule;
mod loader;
mod media_rule;
mod namespace_rule;
pub mod origin;
mod page_rule;
mod property_rule;
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
use std::borrow::Cow;
use std::fmt;
#[cfg(feature = "gecko")]
use std::mem::{self, ManuallyDrop};
use style_traits::ParsingMode;
#[cfg(feature = "gecko")]
use to_shmem::{self, SharedMemoryBuilder, ToShmem};

pub use self::container_rule::ContainerRule;
pub use self::counter_style_rule::CounterStyleRule;
pub use self::document_rule::DocumentRule;
pub use self::font_face_rule::FontFaceRule;
pub use self::font_feature_values_rule::FontFeatureValuesRule;
pub use self::font_palette_values_rule::FontPaletteValuesRule;
pub use self::import_rule::ImportRule;
pub use self::keyframes_rule::KeyframesRule;
pub use self::layer_rule::{LayerBlockRule, LayerStatementRule};
pub use self::loader::StylesheetLoader;
pub use self::media_rule::MediaRule;
pub use self::namespace_rule::NamespaceRule;
pub use self::origin::{Origin, OriginSet, OriginSetIterator, PerOrigin, PerOriginIter};
pub use self::page_rule::{PageRule, PageSelector, PageSelectors};
pub use self::property_rule::PropertyRule;
pub use self::rule_list::{CssRules, CssRulesHelpers};
pub use self::rule_parser::{InsertRuleContext, State, TopLevelRuleParser};
pub use self::rules_iterator::{AllRules, EffectiveRules};
pub use self::rules_iterator::{
    EffectiveRulesIterator, NestedRuleIterationCondition, RulesIterator,
};
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
    pub fn chrome_rules_enabled(&self) -> bool {
        self.as_ref().mChromeRulesEnabled
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
            .field("chrome_rules_enabled", &self.chrome_rules_enabled())
            .field("base", &DebugURI(self.as_ref().mBaseURI.raw()))
            .field(
                "referrer",
                &DebugReferrerInfo(self.as_ref().mReferrerInfo.raw()),
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
    Namespace(Arc<NamespaceRule>),
    Import(Arc<Locked<ImportRule>>),
    Style(Arc<Locked<StyleRule>>),
    Media(Arc<MediaRule>),
    Container(Arc<ContainerRule>),
    FontFace(Arc<Locked<FontFaceRule>>),
    FontFeatureValues(Arc<FontFeatureValuesRule>),
    FontPaletteValues(Arc<FontPaletteValuesRule>),
    CounterStyle(Arc<Locked<CounterStyleRule>>),
    Viewport(Arc<ViewportRule>),
    Keyframes(Arc<Locked<KeyframesRule>>),
    Supports(Arc<SupportsRule>),
    Page(Arc<Locked<PageRule>>),
    Property(Arc<PropertyRule>),
    Document(Arc<DocumentRule>),
    LayerBlock(Arc<LayerBlockRule>),
    LayerStatement(Arc<LayerStatementRule>),
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
            CssRule::Media(ref arc) => {
                arc.unconditional_shallow_size_of(ops) + arc.size_of(guard, ops)
            },
            CssRule::Container(ref arc) => {
                arc.unconditional_shallow_size_of(ops) + arc.size_of(guard, ops)
            },
            CssRule::FontFace(_) => 0,
            CssRule::FontFeatureValues(_) => 0,
            CssRule::FontPaletteValues(_) => 0,
            CssRule::CounterStyle(_) => 0,
            CssRule::Viewport(_) => 0,
            CssRule::Keyframes(_) => 0,
            CssRule::Supports(ref arc) => {
                arc.unconditional_shallow_size_of(ops) + arc.size_of(guard, ops)
            },
            CssRule::Page(ref lock) => {
                lock.unconditional_shallow_size_of(ops) + lock.read_with(guard).size_of(guard, ops)
            },
            CssRule::Property(ref rule) => {
                rule.unconditional_shallow_size_of(ops) + rule.size_of(guard, ops)
            },
            CssRule::Document(ref arc) => {
                arc.unconditional_shallow_size_of(ops) + arc.size_of(guard, ops)
            },
            // TODO(emilio): Add memory reporting for these rules.
            CssRule::LayerBlock(_) | CssRule::LayerStatement(_) => 0,
        }
    }
}

/// https://drafts.csswg.org/cssom-1/#dom-cssrule-type
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
#[repr(u8)]
pub enum CssRuleType {
    // https://drafts.csswg.org/cssom/#the-cssrule-interface
    Style = 1,
    // Charset = 2, // Historical
    Import = 3,
    Media = 4,
    FontFace = 5,
    Page = 6,
    // https://drafts.csswg.org/css-animations-1/#interface-cssrule-idl
    Keyframes = 7,
    Keyframe = 8,
    // https://drafts.csswg.org/cssom/#the-cssrule-interface
    // Margin = 9, // Not implemented yet.
    Namespace = 10,
    // https://drafts.csswg.org/css-counter-styles-3/#extentions-to-cssrule-interface
    CounterStyle = 11,
    // https://drafts.csswg.org/css-conditional-3/#extentions-to-cssrule-interface
    Supports = 12,
    // https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#extentions-to-cssrule-interface
    Document = 13,
    // https://drafts.csswg.org/css-fonts/#om-fontfeaturevalues
    FontFeatureValues = 14,
    // https://drafts.csswg.org/css-device-adapt/#css-rule-interface
    Viewport = 15,
    // After viewport, all rules should return 0 from the API, but we still need
    // a constant somewhere.
    LayerBlock = 16,
    LayerStatement = 17,
    Container = 18,
    FontPaletteValues = 19,
    // 20 is an arbitrary number to use for Property.
    Property = 20,
}

impl CssRuleType {
    /// Returns a bit that identifies this rule type.
    #[inline]
    pub const fn bit(self) -> u32 {
        1 << self as u32
    }
}

/// Set of rule types.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssRuleTypes(u32);

impl From<CssRuleType> for CssRuleTypes {
    fn from(ty: CssRuleType) -> Self {
        Self(ty.bit())
    }
}

impl CssRuleTypes {
    /// Returns whether the rule is in the current set.
    #[inline]
    pub fn contains(self, ty: CssRuleType) -> bool {
        self.0 & ty.bit() != 0
    }

    /// Returns all the rules specified in the set.
    pub fn bits(self) -> u32 {
        self.0
    }

    /// Inserts a rule type into the set.
    #[inline]
    pub fn insert(&mut self, ty: CssRuleType) {
        self.0 |= ty.bit()
    }
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
            CssRule::FontPaletteValues(_) => CssRuleType::FontPaletteValues,
            CssRule::CounterStyle(_) => CssRuleType::CounterStyle,
            CssRule::Keyframes(_) => CssRuleType::Keyframes,
            CssRule::Namespace(_) => CssRuleType::Namespace,
            CssRule::Viewport(_) => CssRuleType::Viewport,
            CssRule::Supports(_) => CssRuleType::Supports,
            CssRule::Page(_) => CssRuleType::Page,
            CssRule::Property(_) => CssRuleType::Property,
            CssRule::Document(_) => CssRuleType::Document,
            CssRule::LayerBlock(_) => CssRuleType::LayerBlock,
            CssRule::LayerStatement(_) => CssRuleType::LayerStatement,
            CssRule::Container(_) => CssRuleType::Container,
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
        let namespaces = parent_stylesheet_contents.namespaces.read();
        let context = ParserContext::new(
            parent_stylesheet_contents.origin,
            &url_data,
            None,
            ParsingMode::DEFAULT,
            parent_stylesheet_contents.quirks_mode,
            Cow::Borrowed(&*namespaces),
            None,
            None,
        );

        let mut input = ParserInput::new(css);
        let mut input = Parser::new(&mut input);

        // nested rules are in the body state
        let mut rule_parser = TopLevelRuleParser {
            context,
            shared_lock: &shared_lock,
            loader,
            state,
            dom_error: None,
            insert_rule_context: Some(insert_rule_context),
            allow_import_rules,
            declaration_parser_state: Default::default(),
            rules: Default::default(),
        };

        match parse_one_rule(&mut input, &mut rule_parser) {
            Ok(_) => Ok(rule_parser.rules.pop().unwrap()),
            Err(_) => Err(rule_parser.dom_error.unwrap_or(RulesMutateError::Syntax)),
        }
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
            CssRule::Namespace(ref arc) => CssRule::Namespace(arc.clone()),
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
            CssRule::Container(ref arc) => {
                CssRule::Container(Arc::new(arc.deep_clone_with_lock(lock, guard, params)))
            },
            CssRule::Media(ref arc) => {
                CssRule::Media(Arc::new(arc.deep_clone_with_lock(lock, guard, params)))
            },
            CssRule::FontFace(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::FontFace(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::FontFeatureValues(ref arc) => CssRule::FontFeatureValues(arc.clone()),
            CssRule::FontPaletteValues(ref arc) => CssRule::FontPaletteValues(arc.clone()),
            CssRule::CounterStyle(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::CounterStyle(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Viewport(ref arc) => CssRule::Viewport(arc.clone()),
            CssRule::Keyframes(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Keyframes(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
            CssRule::Supports(ref arc) => {
                CssRule::Supports(Arc::new(arc.deep_clone_with_lock(lock, guard, params)))
            },
            CssRule::Page(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Page(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard, params)),
                ))
            },
            CssRule::Property(ref arc) => {
                // @property rules are immutable, so we don't need any of the `Locked`
                // shenanigans, actually, and can just share the rule.
                CssRule::Property(arc.clone())
            },
            CssRule::Document(ref arc) => {
                CssRule::Document(Arc::new(arc.deep_clone_with_lock(lock, guard, params)))
            },
            CssRule::LayerStatement(ref arc) => CssRule::LayerStatement(arc.clone()),
            CssRule::LayerBlock(ref arc) => {
                CssRule::LayerBlock(Arc::new(arc.deep_clone_with_lock(lock, guard, params)))
            },
        }
    }
}

impl ToCssWithGuard for CssRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        match *self {
            CssRule::Namespace(ref rule) => rule.to_css(guard, dest),
            CssRule::Import(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Style(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::FontFace(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::FontFeatureValues(ref rule) => rule.to_css(guard, dest),
            CssRule::FontPaletteValues(ref rule) => rule.to_css(guard, dest),
            CssRule::CounterStyle(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Viewport(ref rule) => rule.to_css(guard, dest),
            CssRule::Keyframes(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Media(ref rule) => rule.to_css(guard, dest),
            CssRule::Supports(ref rule) => rule.to_css(guard, dest),
            CssRule::Page(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Property(ref rule) => rule.to_css(guard, dest),
            CssRule::Document(ref rule) => rule.to_css(guard, dest),
            CssRule::LayerBlock(ref rule) => rule.to_css(guard, dest),
            CssRule::LayerStatement(ref rule) => rule.to_css(guard, dest),
            CssRule::Container(ref rule) => rule.to_css(guard, dest),
        }
    }
}
