/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use indexmap::IndexMap;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::like::Maplike;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use style::stylesheets::font_feature_values_rule::{
    FFVDeclaration, PairValues, SingleValue, VectorValues,
};

use crate::dom::GlobalScope;
use crate::dom::bindings::codegen::Bindings::CSSFontFeatureValuesRuleBinding::CSSFontFeatureValuesMapMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::maplike;

/// <https://drafts.csswg.org/css-fonts/#cssfontfeaturevaluesmap>
#[dom_struct]
pub(crate) struct CSSFontFeatureValuesMap {
    reflector: Reflector,
    #[custom_trace]
    internal: DomRefCell<IndexMap<DOMString, Vec<u32>>>,
}

impl CSSFontFeatureValuesMap {
    fn new_inherited(map: IndexMap<DOMString, Vec<u32>>) -> CSSFontFeatureValuesMap {
        CSSFontFeatureValuesMap {
            reflector: Reflector::new(),
            internal: DomRefCell::new(map),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        map: IndexMap<DOMString, Vec<u32>>,
    ) -> DomRoot<CSSFontFeatureValuesMap> {
        reflect_dom_object_with_cx(
            Box::new(CSSFontFeatureValuesMap::new_inherited(map)),
            global,
            cx,
        )
    }

    pub(crate) fn build_from<DeclarationValue>(
        cx: &mut JSContext,
        global: &GlobalScope,
        declarations: &[FFVDeclaration<DeclarationValue>],
    ) -> DomRoot<Self>
    where
        DeclarationValue: AsValues,
    {
        let mut map = IndexMap::default();
        for declaration in declarations {
            map.insert(
                declaration.name.to_string().into(),
                declaration.value.as_vec(),
            );
        }
        Self::new(cx, global, map)
    }
}

impl Maplike for CSSFontFeatureValuesMap {
    type Key = DOMString;
    type Value = Vec<u32>;

    maplike!(self, internal);
}

impl CSSFontFeatureValuesMapMethods<crate::DomTypeHolder> for CSSFontFeatureValuesMap {
    fn Size(&self) -> u32 {
        self.internal.borrow().len() as u32
    }
}

/// A trait to generalize over [SingleValue], [PairValues] and [VectorValues`].
pub(crate) trait AsValues {
    fn as_vec(&self) -> Vec<u32>;
}

impl AsValues for SingleValue {
    fn as_vec(&self) -> Vec<u32> {
        vec![self.0]
    }
}

impl AsValues for PairValues {
    fn as_vec(&self) -> Vec<u32> {
        let mut result = vec![self.0];
        if let Some(second_value) = self.1 {
            result.push(second_value);
        }
        result
    }
}

impl AsValues for VectorValues {
    fn as_vec(&self) -> Vec<u32> {
        self.0.clone()
    }
}
