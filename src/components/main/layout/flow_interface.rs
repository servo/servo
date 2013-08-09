/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::flow::{FlowContext, VisitChildView, VisitOrChildView};
use layout::flow::{BlockFlow,FloatFlow,InlineFlow};
use layout::block::BlockFlowData;
use layout::float::FloatFlowData;
use layout::inline::InlineFlowData;

// This file defines useful methods for use in layout visitors. Each type of FlowData
// is given a method to access its parent flow, and each flow is given a method to
// iterate over its children.

pub trait Visitor<Child> {
    pub fn each_child(&self, &fn(Child) -> bool) -> bool;
}

impl<V:VisitOrChildView> Visitor<FlowContext<VisitChildView,VisitChildView>>
for FlowContext<V,VisitChildView> {
    pub fn each_child(&self, 
                      callback: &fn(FlowContext<VisitChildView,VisitChildView>) -> bool)
                      -> bool {
        let mut maybe_current = self.first_child();
        while !maybe_current.is_none() {
            let current = maybe_current.get_ref().clone();
            if !callback(current.clone()) {
                break;
            }

            maybe_current = current.next_sibling();
        }

        true
    }
}

pub trait FlowDataMethods<R:Visitor<FlowContext<VisitChildView,VisitChildView>>> {
    pub fn flow(@mut self) -> R;
}

impl<V:VisitOrChildView> FlowDataMethods<FlowContext<V,VisitChildView>> 
for  BlockFlowData<V,VisitChildView> {
    fn flow(@mut self) -> FlowContext<V,VisitChildView> {
        BlockFlow(self)
    }
}

impl<V:VisitOrChildView> FlowDataMethods<FlowContext<V,VisitChildView>> 
for  FloatFlowData<V,VisitChildView> {
    fn flow(@mut self) -> FlowContext<V,VisitChildView> {
        FloatFlow(self)
    }
}

impl<V:VisitOrChildView> FlowDataMethods<FlowContext<V,VisitChildView>> 
for  InlineFlowData<V,VisitChildView> {
    fn flow(@mut self) -> FlowContext<V,VisitChildView> {
        InlineFlow(self)
    }
}

