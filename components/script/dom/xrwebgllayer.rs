/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerInit;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{DomObject, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::window::Window;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrviewport::XRViewport;
use dom_struct::dom_struct;

use std::cell::Cell;

#[dom_struct]
pub struct XRWebGLLayer {
    xrlayer: XRLayer,
    antialias: Cell<bool>,
    depth: Cell<bool>,
    stencil: Cell<bool>,
    alpha: Cell<bool>,
    context: Dom<WebGLRenderingContext>,
    session: Dom<XRSession>,
}

impl XRWebGLLayer {
    pub fn new_inherited(session: &XRSession, context: &WebGLRenderingContext,
                         init: &XRWebGLLayerInit) -> XRWebGLLayer {
        XRWebGLLayer {
            xrlayer: XRLayer::new_inherited(),
            antialias: Cell::new(init.antialias),
            depth: Cell::new(init.depth),
            stencil: Cell::new(init.stencil),
            alpha: Cell::new(init.alpha),
            context: Dom::from_ref(context),
            session: Dom::from_ref(session),
        }
    }

    pub fn new(global: &GlobalScope, session: &XRSession, context: &WebGLRenderingContext,
               init: &XRWebGLLayerInit) -> DomRoot<XRWebGLLayer> {
        reflect_dom_object(
            Box::new(XRWebGLLayer::new_inherited(session, context, init)),
            global,
            XRWebGLLayerBinding::Wrap,
        )
    }

    pub fn Constructor(global: &Window, session: &XRSession, 
                       context: &WebGLRenderingContext,
                       init: &XRWebGLLayerInit) -> Fallible<DomRoot<Self>> {
        Ok(XRWebGLLayer::new(&global.global(), session, context, init))
    }
}

impl XRWebGLLayerMethods for XRWebGLLayer {
    fn Depth(&self) -> bool {
        self.depth.get()
    }

    fn Stencil(&self) -> bool {
        self.stencil.get()
    }

    fn Antialias(&self) -> bool {
        self.antialias.get()
    }

    fn Alpha(&self) -> bool {
        self.alpha.get()
    }

    fn Context(&self) -> DomRoot<WebGLRenderingContext> {
        DomRoot::from_ref(&self.context)
    }

    fn GetViewport(&self, view: &XRView) -> Option<DomRoot<XRViewport>> {
        if self.session != view.session() {
            return None;
        }

        let size = self.context.size();

        let x = if view.Eye() == XREye::Left {
            0
        } else {
            size.width / 2
        };
        // XXXManishearth this assumes the WebVR default of canvases being cut in half
        // which need not be generally true for all devices, and will not work in
        // inline VR mode
        Some(XRViewport::new(&self.global(), x, 0, size.width / 2, size.height))
    }
}

