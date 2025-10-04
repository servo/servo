/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, SharedRwLockReadGuard, ToCssWithGuard};
use style::stylesheets::{CssRuleType, NestedDeclarationsRule};

use crate::dom::bindings::codegen::Bindings::CSSNestedDeclarationsBinding::CSSNestedDeclarationsMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrule::{CSSRule, SpecificCSSRule};
use crate::dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSNestedDeclarations {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    nesteddeclarationsrule: RefCell<Arc<Locked<NestedDeclarationsRule>>>,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
}

impl CSSNestedDeclarations {
    pub(crate) fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        nesteddeclarationsrule: Arc<Locked<NestedDeclarationsRule>>,
    ) -> Self {
        Self {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            nesteddeclarationsrule: RefCell::new(nesteddeclarationsrule),
            style_decl: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        nesteddeclarationsrule: Arc<Locked<NestedDeclarationsRule>>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(
                parent_stylesheet,
                nesteddeclarationsrule,
            )),
            window,
            can_gc,
        )
    }

    pub(crate) fn update_rule(
        &self,
        nesteddeclarationsrule: Arc<Locked<NestedDeclarationsRule>>,
        guard: &SharedRwLockReadGuard,
    ) {
        if let Some(ref style_decl) = self.style_decl.get() {
            style_decl
                .update_property_declaration_block(&nesteddeclarationsrule.read_with(guard).block);
        }
        *self.nesteddeclarationsrule.borrow_mut() = nesteddeclarationsrule;
    }
}

impl SpecificCSSRule for CSSNestedDeclarations {
    fn ty(&self) -> CssRuleType {
        CssRuleType::NestedDeclarations
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.nesteddeclarationsrule
            .borrow()
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}

impl CSSNestedDeclarationsMethods<crate::DomTypeHolder> for CSSNestedDeclarations {
    /// <https://drafts.csswg.org/css-nesting/#dom-cssnesteddeclarations-style>
    fn Style(&self, can_gc: CanGc) -> DomRoot<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let guard = self.cssrule.shared_lock().read();
            CSSStyleDeclaration::new(
                self.global().as_window(),
                CSSStyleOwner::CSSRule(
                    Dom::from_ref(self.upcast()),
                    RefCell::new(
                        self.nesteddeclarationsrule
                            .borrow()
                            .read_with(&guard)
                            .block
                            .clone(),
                    ),
                ),
                None,
                CSSModificationAccess::ReadWrite,
                can_gc,
            )
        })
    }
}
