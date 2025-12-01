/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::generic_channel::GenericSender;
use base::id::PainterId;
use ipc_channel::ipc::{self, IpcSender};
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::RwLock;
use profile_traits::mem::ReportsChan;
use serde::{Deserialize, Serialize};
use style::values::computed::font::SingleFontFamily;
use webrender_api::units::Au;
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey, FontVariation};

use crate::{FontDescriptor, FontIdentifier, FontTemplate, FontTemplateRef};

/// Commands that the `FontContext` sends to the `SystemFontService`.
#[derive(Debug, Deserialize, Serialize)]
pub enum SystemFontServiceMessage {
    GetFontTemplates(
        Option<FontDescriptor>,
        SingleFontFamily,
        IpcSender<Vec<FontTemplate>>,
    ),
    GetFontInstance(
        PainterId,
        FontIdentifier,
        Au,
        FontInstanceFlags,
        Vec<FontVariation>,
        IpcSender<FontInstanceKey>,
    ),
    PrefetchFontKeys(PainterId),
    GetFontKey(PainterId, IpcSender<FontKey>),
    GetFontInstanceKey(PainterId, IpcSender<FontInstanceKey>),
    CollectMemoryReport(ReportsChan),
    Exit(IpcSender<()>),
    Ping,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SystemFontServiceProxySender(pub GenericSender<SystemFontServiceMessage>);

impl SystemFontServiceProxySender {
    pub fn to_proxy(&self) -> SystemFontServiceProxy {
        SystemFontServiceProxy {
            sender: self.0.clone(),
            templates: Default::default(),
        }
    }
}

#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
struct FontTemplateCacheKey {
    font_descriptor: Option<FontDescriptor>,
    family_descriptor: SingleFontFamily,
}

/// The public interface to the [`SystemFontService`], used by per-Document
/// `FontContext` instances.
#[derive(Debug, MallocSizeOf)]
pub struct SystemFontServiceProxy {
    sender: GenericSender<SystemFontServiceMessage>,
    templates: RwLock<HashMap<FontTemplateCacheKey, Vec<FontTemplateRef>>>,
}

impl SystemFontServiceProxy {
    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.sender
            .send(SystemFontServiceMessage::Exit(response_chan))
            .expect("Couldn't send SystemFontService exit message");
        response_port
            .recv()
            .expect("Couldn't receive SystemFontService reply");
    }

    pub fn to_sender(&self) -> SystemFontServiceProxySender {
        SystemFontServiceProxySender(self.sender.clone())
    }

    pub fn get_system_font_instance(
        &self,
        identifier: FontIdentifier,
        size: Au,
        flags: FontInstanceFlags,
        variations: Vec<FontVariation>,
        painter_id: PainterId,
    ) -> FontInstanceKey {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontInstance(
                painter_id,
                identifier,
                size,
                flags,
                variations,
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let instance_key = response_port.recv();
        if instance_key.is_err() {
            let font_thread_has_closed = self.sender.send(SystemFontServiceMessage::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        }
        instance_key.unwrap()
    }

    pub fn find_matching_font_templates(
        &self,
        descriptor_to_match: Option<&FontDescriptor>,
        family_descriptor: &SingleFontFamily,
    ) -> Vec<FontTemplateRef> {
        let cache_key = FontTemplateCacheKey {
            font_descriptor: descriptor_to_match.cloned(),
            family_descriptor: family_descriptor.clone(),
        };
        if let Some(templates) = self.templates.read().get(&cache_key).cloned() {
            return templates;
        }

        debug!(
            "SystemFontServiceProxy: cache miss for template_descriptor={:?} family_descriptor={:?}",
            descriptor_to_match, family_descriptor
        );

        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontTemplates(
                descriptor_to_match.cloned(),
                family_descriptor.clone(),
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let Ok(templates) = response_port.recv() else {
            let font_thread_has_closed = self.sender.send(SystemFontServiceMessage::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        };

        let templates: Vec<_> = templates.into_iter().map(FontTemplateRef::new).collect();
        self.templates.write().insert(cache_key, templates.clone());

        templates
    }

    pub fn generate_font_key(&self, painter_id: PainterId) -> FontKey {
        let (result_sender, result_receiver) =
            ipc::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontKey(
                painter_id,
                result_sender,
            ))
            .expect("failed to send message to system font service");
        result_receiver
            .recv()
            .expect("Failed to communicate with system font service.")
    }

    pub fn generate_font_instance_key(&self, painter_id: PainterId) -> FontInstanceKey {
        let (result_sender, result_receiver) =
            ipc::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontInstanceKey(
                painter_id,
                result_sender,
            ))
            .expect("failed to send message to system font service");
        result_receiver
            .recv()
            .expect("Failed to communicate with system font service.")
    }

    pub fn prefetch_font_keys_for_painter(&self, painter_id: PainterId) {
        let _ = self
            .sender
            .send(SystemFontServiceMessage::PrefetchFontKeys(painter_id));
    }
}
