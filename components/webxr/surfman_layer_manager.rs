/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An implementation of layer management using surfman

use crate::gl_utils::GlClearer;
use euclid::{Point2D, Rect, Size2D};
use glow::{self as gl, Context as Gl, HasContext, PixelUnpackData};
use std::collections::HashMap;
use std::num::NonZeroU32;
use surfman::chains::{PreserveBuffer, SwapChains, SwapChainsAPI};
use surfman::{Context as SurfmanContext, Device as SurfmanDevice, SurfaceAccess, SurfaceTexture};
use webxr_api::{
    ContextId, Error, GLContexts, GLTypes, LayerId, LayerInit, LayerManagerAPI, SubImage,
    SubImages, Viewports,
};

#[derive(Copy, Clone, Debug)]
pub enum SurfmanGL {}

impl GLTypes for SurfmanGL {
    type Device = SurfmanDevice;
    type Context = SurfmanContext;
    type Bindings = Gl;
}

pub struct SurfmanLayerManager {
    layers: Vec<(ContextId, LayerId)>,
    swap_chains: SwapChains<LayerId, SurfmanDevice>,
    surface_textures: HashMap<LayerId, SurfaceTexture>,
    depth_stencil_textures: HashMap<LayerId, Option<gl::NativeTexture>>,
    viewports: Viewports,
    clearer: GlClearer,
}

impl SurfmanLayerManager {
    pub fn new(
        viewports: Viewports,
        swap_chains: SwapChains<LayerId, SurfmanDevice>,
    ) -> SurfmanLayerManager {
        let layers = Vec::new();
        let surface_textures = HashMap::new();
        let depth_stencil_textures = HashMap::new();
        let clearer = GlClearer::new(false);
        SurfmanLayerManager {
            layers,
            swap_chains,
            surface_textures,
            depth_stencil_textures,
            viewports,
            clearer,
        }
    }
}

impl LayerManagerAPI<SurfmanGL> for SurfmanLayerManager {
    fn create_layer(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        context_id: ContextId,
        init: LayerInit,
    ) -> Result<LayerId, Error> {
        let texture_size = init.texture_size(&self.viewports);
        let layer_id = LayerId::new();
        let access = SurfaceAccess::GPUOnly;
        let size = texture_size.to_untyped();
        // TODO: Treat depth and stencil separately?
        let has_depth_stencil = match init {
            LayerInit::WebGLLayer { stencil, depth, .. } => stencil | depth,
            LayerInit::ProjectionLayer { stencil, depth, .. } => stencil | depth,
        };
        if has_depth_stencil {
            let gl = contexts
                .bindings(device, context_id)
                .ok_or(Error::NoMatchingDevice)?;
            let depth_stencil_texture = unsafe { gl.create_texture().ok() };
            unsafe {
                gl.bind_texture(gl::TEXTURE_2D, depth_stencil_texture);
                gl.tex_image_2d(
                    gl::TEXTURE_2D,
                    0,
                    gl::DEPTH24_STENCIL8 as _,
                    size.width,
                    size.height,
                    0,
                    gl::DEPTH_STENCIL,
                    gl::UNSIGNED_INT_24_8,
                    PixelUnpackData::Slice(None),
                );
            }
            self.depth_stencil_textures
                .insert(layer_id, depth_stencil_texture);
        }
        let context = contexts
            .context(device, context_id)
            .ok_or(Error::NoMatchingDevice)?;
        self.swap_chains
            .create_detached_swap_chain(layer_id, size, device, context, access)
            .map_err(|err| Error::BackendSpecific(format!("{:?}", err)))?;
        self.layers.push((context_id, layer_id));
        Ok(layer_id)
    }

    fn destroy_layer(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        context_id: ContextId,
        layer_id: LayerId,
    ) {
        self.clearer
            .destroy_layer(device, contexts, context_id, layer_id);
        let context = match contexts.context(device, context_id) {
            Some(context) => context,
            None => return,
        };
        self.layers.retain(|&ids| ids != (context_id, layer_id));
        let _ = self.swap_chains.destroy(layer_id, device, context);
        self.surface_textures.remove(&layer_id);
        if let Some(depth_stencil_texture) = self.depth_stencil_textures.remove(&layer_id) {
            let gl = contexts.bindings(device, context_id).unwrap();
            if let Some(depth_stencil_texture) = depth_stencil_texture {
                unsafe {
                    gl.delete_texture(depth_stencil_texture);
                }
            }
        }
    }

    fn layers(&self) -> &[(ContextId, LayerId)] {
        &self.layers[..]
    }

    fn begin_frame(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<Vec<SubImages>, Error> {
        layers
            .iter()
            .map(|&(context_id, layer_id)| {
                let context = contexts
                    .context(device, context_id)
                    .ok_or(Error::NoMatchingDevice)?;
                let swap_chain = self
                    .swap_chains
                    .get(layer_id)
                    .ok_or(Error::NoMatchingDevice)?;
                let surface_size = Size2D::from_untyped(swap_chain.size());
                let surface_texture = swap_chain
                    .take_surface_texture(device, context)
                    .map_err(|_| Error::NoMatchingDevice)?;
                let color_texture = device.surface_texture_object(&surface_texture);
                let color_target = device.surface_gl_texture_target();
                let depth_stencil_texture = self
                    .depth_stencil_textures
                    .get(&layer_id)
                    .cloned()
                    .flatten();
                let texture_array_index = None;
                let origin = Point2D::new(0, 0);
                let sub_image = Some(SubImage {
                    color_texture,
                    depth_stencil_texture: depth_stencil_texture.map(|nt| nt.0.get()),
                    texture_array_index,
                    viewport: Rect::new(origin, surface_size),
                });
                let view_sub_images = self
                    .viewports
                    .viewports
                    .iter()
                    .map(|&viewport| SubImage {
                        color_texture,
                        depth_stencil_texture: depth_stencil_texture.map(|texture| texture.0.get()),
                        texture_array_index,
                        viewport,
                    })
                    .collect();
                self.surface_textures.insert(layer_id, surface_texture);
                self.clearer.clear(
                    device,
                    contexts,
                    context_id,
                    layer_id,
                    NonZeroU32::new(color_texture).map(gl::NativeTexture),
                    color_target,
                    depth_stencil_texture,
                );
                Ok(SubImages {
                    layer_id,
                    sub_image,
                    view_sub_images,
                })
            })
            .collect()
    }

    fn end_frame(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<(), Error> {
        for &(context_id, layer_id) in layers {
            let gl = contexts
                .bindings(device, context_id)
                .ok_or(Error::NoMatchingDevice)?;
            unsafe {
                gl.flush();
            }
            let context = contexts
                .context(device, context_id)
                .ok_or(Error::NoMatchingDevice)?;
            let surface_texture = self
                .surface_textures
                .remove(&layer_id)
                .ok_or(Error::NoMatchingDevice)?;
            let swap_chain = self
                .swap_chains
                .get(layer_id)
                .ok_or(Error::NoMatchingDevice)?;
            swap_chain
                .recycle_surface_texture(device, context, surface_texture)
                .map_err(|err| Error::BackendSpecific(format!("{:?}", err)))?;
            swap_chain
                .swap_buffers(device, context, PreserveBuffer::No)
                .map_err(|err| Error::BackendSpecific(format!("{:?}", err)))?;
        }
        Ok(())
    }
}
