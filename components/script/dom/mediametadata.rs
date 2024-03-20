/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MediaMetadataBinding::{
    MediaMetadataInit, MediaMetadataMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::mediasession::MediaSession;
use crate::dom::window::Window;

#[dom_struct]
pub struct MediaMetadata {
    reflector_: Reflector,
    session: MutNullableDom<MediaSession>,
    title: DomRefCell<DOMString>,
    artist: DomRefCell<DOMString>,
    album: DomRefCell<DOMString>,
}

impl MediaMetadata {
    fn new_inherited(init: &MediaMetadataInit) -> MediaMetadata {
        MediaMetadata {
            reflector_: Reflector::new(),
            session: Default::default(),
            title: DomRefCell::new(init.title.clone()),
            artist: DomRefCell::new(init.artist.clone()),
            album: DomRefCell::new(init.album.clone()),
        }
    }

    pub fn new(global: &Window, init: &MediaMetadataInit) -> DomRoot<MediaMetadata> {
        Self::new_with_proto(global, None, init)
    }

    fn new_with_proto(
        global: &Window,
        proto: Option<HandleObject>,
        init: &MediaMetadataInit,
    ) -> DomRoot<MediaMetadata> {
        reflect_dom_object_with_proto(Box::new(MediaMetadata::new_inherited(init)), global, proto)
    }

    /// <https://w3c.github.io/mediasession/#dom-mediametadata-mediametadata>
    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        init: &MediaMetadataInit,
    ) -> Fallible<DomRoot<MediaMetadata>> {
        Ok(MediaMetadata::new_with_proto(window, proto, init))
    }

    fn queue_update_metadata_algorithm(&self) {
        if self.session.get().is_none() {}
    }

    pub fn set_session(&self, session: &MediaSession) {
        self.session.set(Some(session));
    }
}

impl MediaMetadataMethods for MediaMetadata {
    /// <https://w3c.github.io/mediasession/#dom-mediametadata-title>
    fn Title(&self) -> DOMString {
        self.title.borrow().clone()
    }

    /// <https://w3c.github.io/mediasession/#dom-mediametadata-title>
    fn SetTitle(&self, value: DOMString) {
        *self.title.borrow_mut() = value;
        self.queue_update_metadata_algorithm();
    }

    /// <https://w3c.github.io/mediasession/#dom-mediametadata-artist>
    fn Artist(&self) -> DOMString {
        self.artist.borrow().clone()
    }

    /// <https://w3c.github.io/mediasession/#dom-mediametadata-artist>
    fn SetArtist(&self, value: DOMString) {
        *self.artist.borrow_mut() = value;
        self.queue_update_metadata_algorithm();
    }

    /// <https://w3c.github.io/mediasession/#dom-mediametadata-album>
    fn Album(&self) -> DOMString {
        self.album.borrow().clone()
    }

    /// <https://w3c.github.io/mediasession/#dom-mediametadata-album>
    fn SetAlbum(&self, value: DOMString) {
        *self.album.borrow_mut() = value;
        self.queue_update_metadata_algorithm();
    }
}
