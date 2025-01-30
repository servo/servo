/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::Error;
use crate::Viewport;
use crate::Viewports;

use euclid::Rect;
use euclid::Size2D;

use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "ipc", derive(Deserialize, Serialize))]
pub struct ContextId(pub u64);

#[cfg(feature = "ipc")]
use serde::{Deserialize, Serialize};

pub trait GLTypes {
    type Device;
    type Context;
    type Bindings;
}

pub trait GLContexts<GL: GLTypes> {
    fn bindings(&mut self, device: &GL::Device, context_id: ContextId) -> Option<&GL::Bindings>;
    fn context(&mut self, device: &GL::Device, context_id: ContextId) -> Option<&mut GL::Context>;
}

impl GLTypes for () {
    type Bindings = ();
    type Device = ();
    type Context = ();
}

impl GLContexts<()> for () {
    fn context(&mut self, _: &(), _: ContextId) -> Option<&mut ()> {
        Some(self)
    }

    fn bindings(&mut self, _: &(), _: ContextId) -> Option<&()> {
        Some(self)
    }
}

pub trait LayerGrandManagerAPI<GL: GLTypes> {
    fn create_layer_manager(&self, factory: LayerManagerFactory<GL>)
        -> Result<LayerManager, Error>;

    fn clone_layer_grand_manager(&self) -> LayerGrandManager<GL>;
}

pub struct LayerGrandManager<GL>(Box<dyn Send + LayerGrandManagerAPI<GL>>);

impl<GL: GLTypes> Clone for LayerGrandManager<GL> {
    fn clone(&self) -> Self {
        self.0.clone_layer_grand_manager()
    }
}

impl<GL> Debug for LayerGrandManager<GL> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        "LayerGrandManager(...)".fmt(fmt)
    }
}

impl<GL: GLTypes> LayerGrandManager<GL> {
    pub fn new<GM>(grand_manager: GM) -> LayerGrandManager<GL>
    where
        GM: 'static + Send + LayerGrandManagerAPI<GL>,
    {
        LayerGrandManager(Box::new(grand_manager))
    }

    pub fn create_layer_manager<F, M>(&self, factory: F) -> Result<LayerManager, Error>
    where
        F: 'static + Send + FnOnce(&mut GL::Device, &mut dyn GLContexts<GL>) -> Result<M, Error>,
        M: 'static + LayerManagerAPI<GL>,
    {
        self.0
            .create_layer_manager(LayerManagerFactory::new(factory))
    }
}

pub trait LayerManagerAPI<GL: GLTypes> {
    fn create_layer(
        &mut self,
        device: &mut GL::Device,
        contexts: &mut dyn GLContexts<GL>,
        context_id: ContextId,
        init: LayerInit,
    ) -> Result<LayerId, Error>;

    fn destroy_layer(
        &mut self,
        device: &mut GL::Device,
        contexts: &mut dyn GLContexts<GL>,
        context_id: ContextId,
        layer_id: LayerId,
    );

    fn layers(&self) -> &[(ContextId, LayerId)];

    fn begin_frame(
        &mut self,
        device: &mut GL::Device,
        contexts: &mut dyn GLContexts<GL>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<Vec<SubImages>, Error>;

    fn end_frame(
        &mut self,
        device: &mut GL::Device,
        contexts: &mut dyn GLContexts<GL>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<(), Error>;
}

pub struct LayerManager(Box<dyn Send + LayerManagerAPI<()>>);

impl Debug for LayerManager {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        "LayerManager(...)".fmt(fmt)
    }
}

impl LayerManager {
    pub fn create_layer(
        &mut self,
        context_id: ContextId,
        init: LayerInit,
    ) -> Result<LayerId, Error> {
        self.0.create_layer(&mut (), &mut (), context_id, init)
    }

    pub fn destroy_layer(&mut self, context_id: ContextId, layer_id: LayerId) {
        self.0.destroy_layer(&mut (), &mut (), context_id, layer_id);
    }

    pub fn begin_frame(
        &mut self,
        layers: &[(ContextId, LayerId)],
    ) -> Result<Vec<SubImages>, Error> {
        self.0.begin_frame(&mut (), &mut (), layers)
    }

    pub fn end_frame(&mut self, layers: &[(ContextId, LayerId)]) -> Result<(), Error> {
        self.0.end_frame(&mut (), &mut (), layers)
    }
}

impl LayerManager {
    pub fn new<M>(manager: M) -> LayerManager
    where
        M: 'static + Send + LayerManagerAPI<()>,
    {
        LayerManager(Box::new(manager))
    }
}

impl Drop for LayerManager {
    fn drop(&mut self) {
        log::debug!("Dropping LayerManager");
        for (context_id, layer_id) in self.0.layers().to_vec() {
            self.destroy_layer(context_id, layer_id);
        }
    }
}

pub struct LayerManagerFactory<GL: GLTypes>(
    Box<
        dyn Send
            + FnOnce(
                &mut GL::Device,
                &mut dyn GLContexts<GL>,
            ) -> Result<Box<dyn LayerManagerAPI<GL>>, Error>,
    >,
);

impl<GL: GLTypes> Debug for LayerManagerFactory<GL> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        "LayerManagerFactory(...)".fmt(fmt)
    }
}

impl<GL: GLTypes> LayerManagerFactory<GL> {
    pub fn new<F, M>(factory: F) -> LayerManagerFactory<GL>
    where
        F: 'static + Send + FnOnce(&mut GL::Device, &mut dyn GLContexts<GL>) -> Result<M, Error>,
        M: 'static + LayerManagerAPI<GL>,
    {
        LayerManagerFactory(Box::new(move |device, contexts| {
            Ok(Box::new(factory(device, contexts)?))
        }))
    }

    pub fn build(
        self,
        device: &mut GL::Device,
        contexts: &mut dyn GLContexts<GL>,
    ) -> Result<Box<dyn LayerManagerAPI<GL>>, Error> {
        (self.0)(device, contexts)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "ipc", derive(Deserialize, Serialize))]
pub struct LayerId(usize);

static NEXT_LAYER_ID: AtomicUsize = AtomicUsize::new(0);

impl LayerId {
    pub fn new() -> LayerId {
        LayerId(NEXT_LAYER_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Deserialize, Serialize))]
pub enum LayerInit {
    // https://www.w3.org/TR/webxr/#dictdef-xrwebgllayerinit
    WebGLLayer {
        antialias: bool,
        depth: bool,
        stencil: bool,
        alpha: bool,
        ignore_depth_values: bool,
        framebuffer_scale_factor: f32,
    },
    // https://immersive-web.github.io/layers/#xrprojectionlayerinittype
    ProjectionLayer {
        depth: bool,
        stencil: bool,
        alpha: bool,
        scale_factor: f32,
    },
    // TODO: other layer types
}

impl LayerInit {
    pub fn texture_size(&self, viewports: &Viewports) -> Size2D<i32, Viewport> {
        match self {
            LayerInit::WebGLLayer {
                framebuffer_scale_factor: scale,
                ..
            }
            | LayerInit::ProjectionLayer {
                scale_factor: scale,
                ..
            } => {
                let native_size = viewports
                    .viewports
                    .iter()
                    .fold(Rect::zero(), |acc, view| acc.union(view))
                    .size;
                (native_size.to_f32() * *scale).to_i32()
            }
        }
    }
}

/// https://immersive-web.github.io/layers/#enumdef-xrlayerlayout
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Deserialize, Serialize))]
pub enum LayerLayout {
    // TODO: Default
    // Allocates one texture
    Mono,
    // Allocates one texture, which is split in half vertically, giving two subimages
    StereoLeftRight,
    // Allocates one texture, which is split in half horizonally, giving two subimages
    StereoTopBottom,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Deserialize, Serialize))]
pub struct SubImages {
    pub layer_id: LayerId,
    pub sub_image: Option<SubImage>,
    pub view_sub_images: Vec<SubImage>,
}

/// https://immersive-web.github.io/layers/#xrsubimagetype
#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Deserialize, Serialize))]
pub struct SubImage {
    pub color_texture: u32,
    // TODO: make this Option<NonZeroU32>
    pub depth_stencil_texture: Option<u32>,
    pub texture_array_index: Option<u32>,
    pub viewport: Rect<i32, Viewport>,
}
