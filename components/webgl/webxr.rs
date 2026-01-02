/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::num::NonZeroU32;
use std::rc::Rc;

use canvas_traits::webgl::{
    WebGLMsg, WebGLSender, WebXRCommand, WebXRLayerManagerId, webgl_channel,
};
use rustc_hash::FxHashMap;
use surfman::{Context, Device};
use webxr::SurfmanGL as WebXRSurfman;
use webxr_api::{
    ContextId as WebXRContextId, Error as WebXRError, GLContexts as WebXRContexts,
    GLTypes as WebXRTypes, LayerGrandManager as WebXRLayerGrandManager,
    LayerGrandManagerAPI as WebXRLayerGrandManagerAPI, LayerId as WebXRLayerId,
    LayerInit as WebXRLayerInit, LayerManager as WebXRLayerManager,
    LayerManagerAPI as WebXRLayerManagerAPI, LayerManagerFactory as WebXRLayerManagerFactory,
    SubImages as WebXRSubImages,
};

use crate::webgl_thread::WebGLThread;

/// Bridge between WebGL and WebXR
pub(crate) struct WebXRBridge {
    factory_receiver: crossbeam_channel::Receiver<WebXRLayerManagerFactory<WebXRSurfman>>,
    managers: FxHashMap<WebXRLayerManagerId, Box<dyn WebXRLayerManagerAPI<WebXRSurfman>>>,
    next_manager_id: NonZeroU32,
}

impl WebXRBridge {
    pub(crate) fn new(init: WebXRBridgeInit) -> WebXRBridge {
        let WebXRBridgeInit {
            factory_receiver, ..
        } = init;
        let managers = FxHashMap::default();
        let next_manager_id = NonZeroU32::MIN;
        WebXRBridge {
            factory_receiver,
            managers,
            next_manager_id,
        }
    }
}

impl WebXRBridge {
    pub(crate) fn create_layer_manager(
        &mut self,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
    ) -> Result<WebXRLayerManagerId, WebXRError> {
        let factory = self
            .factory_receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)?;
        let manager = factory.build(contexts)?;
        let manager_id = WebXRLayerManagerId::new(self.next_manager_id);
        self.next_manager_id = self
            .next_manager_id
            .checked_add(1)
            .expect("next_manager_id should not overflow");
        self.managers.insert(manager_id, manager);
        Ok(manager_id)
    }

    pub(crate) fn destroy_layer_manager(&mut self, manager_id: WebXRLayerManagerId) {
        self.managers.remove(&manager_id);
    }

    pub(crate) fn create_layer(
        &mut self,
        manager_id: WebXRLayerManagerId,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        context_id: WebXRContextId,
        layer_init: WebXRLayerInit,
    ) -> Result<WebXRLayerId, WebXRError> {
        let manager = self
            .managers
            .get_mut(&manager_id)
            .ok_or(WebXRError::NoMatchingDevice)?;
        manager.create_layer(contexts, context_id, layer_init)
    }

    pub(crate) fn destroy_layer(
        &mut self,
        manager_id: WebXRLayerManagerId,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        context_id: WebXRContextId,
        layer_id: WebXRLayerId,
    ) {
        if let Some(manager) = self.managers.get_mut(&manager_id) {
            manager.destroy_layer(contexts, context_id, layer_id);
        }
    }

    pub(crate) fn destroy_all_layers(
        &mut self,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        context_id: WebXRContextId,
    ) {
        for manager in self.managers.values_mut() {
            for (other_id, layer_id) in manager.layers().to_vec() {
                if other_id == context_id {
                    manager.destroy_layer(contexts, context_id, layer_id);
                }
            }
        }
    }

    pub(crate) fn begin_frame(
        &mut self,
        manager_id: WebXRLayerManagerId,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<Vec<WebXRSubImages>, WebXRError> {
        let manager = self
            .managers
            .get_mut(&manager_id)
            .ok_or(WebXRError::NoMatchingDevice)?;
        manager.begin_frame(contexts, layers)
    }

    pub(crate) fn end_frame(
        &mut self,
        manager_id: WebXRLayerManagerId,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<(), WebXRError> {
        let manager = self
            .managers
            .get_mut(&manager_id)
            .ok_or(WebXRError::NoMatchingDevice)?;
        manager.end_frame(contexts, layers)
    }
}

pub(crate) struct WebXRBridgeInit {
    sender: WebGLSender<WebGLMsg>,
    factory_receiver: crossbeam_channel::Receiver<WebXRLayerManagerFactory<WebXRSurfman>>,
    factory_sender: crossbeam_channel::Sender<WebXRLayerManagerFactory<WebXRSurfman>>,
}

impl WebXRBridgeInit {
    pub(crate) fn new(sender: WebGLSender<WebGLMsg>) -> WebXRBridgeInit {
        let (factory_sender, factory_receiver) = crossbeam_channel::unbounded();
        WebXRBridgeInit {
            sender,
            factory_sender,
            factory_receiver,
        }
    }

    pub(crate) fn layer_grand_manager(&self) -> WebXRLayerGrandManager<WebXRSurfman> {
        WebXRLayerGrandManager::new(WebXRBridgeGrandManager {
            sender: self.sender.clone(),
            factory_sender: self.factory_sender.clone(),
        })
    }
}

struct WebXRBridgeGrandManager {
    sender: WebGLSender<WebGLMsg>,
    // WebXR layer manager factories use generic trait objects under the
    // hood, which aren't deserializable (even using typetag)
    // so we can't send them over the regular webgl channel.
    // Fortunately, the webgl thread runs in the same process as
    // the webxr threads, so we can use a crossbeam channel to send
    // factories.
    factory_sender: crossbeam_channel::Sender<WebXRLayerManagerFactory<WebXRSurfman>>,
}

impl WebXRLayerGrandManagerAPI<WebXRSurfman> for WebXRBridgeGrandManager {
    fn create_layer_manager(
        &self,
        factory: WebXRLayerManagerFactory<WebXRSurfman>,
    ) -> Result<WebXRLayerManager, WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self.factory_sender.send(factory);
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::CreateLayerManager(
                sender,
            )));
        let sender = self.sender.clone();
        let manager_id = receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)??;
        let layers = Vec::new();
        Ok(WebXRLayerManager::new(WebXRBridgeManager {
            manager_id,
            sender,
            layers,
        }))
    }

    fn clone_layer_grand_manager(&self) -> WebXRLayerGrandManager<WebXRSurfman> {
        WebXRLayerGrandManager::new(WebXRBridgeGrandManager {
            sender: self.sender.clone(),
            factory_sender: self.factory_sender.clone(),
        })
    }
}

struct WebXRBridgeManager {
    sender: WebGLSender<WebGLMsg>,
    manager_id: WebXRLayerManagerId,
    layers: Vec<(WebXRContextId, WebXRLayerId)>,
}

impl<GL: WebXRTypes> WebXRLayerManagerAPI<GL> for WebXRBridgeManager {
    fn create_layer(
        &mut self,
        _: &mut dyn WebXRContexts<GL>,
        context_id: WebXRContextId,
        init: WebXRLayerInit,
    ) -> Result<WebXRLayerId, WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::CreateLayer(
                self.manager_id,
                context_id,
                init,
                sender,
            )));
        let layer_id = receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)??;
        self.layers.push((context_id, layer_id));
        Ok(layer_id)
    }

    fn destroy_layer(
        &mut self,
        _: &mut dyn WebXRContexts<GL>,
        context_id: WebXRContextId,
        layer_id: WebXRLayerId,
    ) {
        self.layers.retain(|&ids| ids != (context_id, layer_id));
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::DestroyLayer(
                self.manager_id,
                context_id,
                layer_id,
            )));
    }

    fn layers(&self) -> &[(WebXRContextId, WebXRLayerId)] {
        &self.layers[..]
    }

    fn begin_frame(
        &mut self,
        _: &mut dyn WebXRContexts<GL>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<Vec<WebXRSubImages>, WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::BeginFrame(
                self.manager_id,
                layers.to_vec(),
                sender,
            )));
        receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)?
    }

    fn end_frame(
        &mut self,
        _: &mut dyn WebXRContexts<GL>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<(), WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::EndFrame(
                self.manager_id,
                layers.to_vec(),
                sender,
            )));
        receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)?
    }
}

impl Drop for WebXRBridgeManager {
    fn drop(&mut self) {
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::DestroyLayerManager(
                self.manager_id,
            )));
    }
}

impl WebXRContexts<WebXRSurfman> for WebGLThread {
    fn device(&self, context_id: WebXRContextId) -> Option<Rc<Device>> {
        self.maybe_device_for_context(context_id.into())
    }

    fn context(&mut self, context_id: WebXRContextId) -> Option<&mut Context> {
        let data = self.make_current_if_needed_mut(context_id.into())?;
        Some(&mut data.ctx)
    }

    fn bindings(&mut self, context_id: WebXRContextId) -> Option<&glow::Context> {
        let data = self.make_current_if_needed(context_id.into())?;
        Some(&data.gl)
    }
}
