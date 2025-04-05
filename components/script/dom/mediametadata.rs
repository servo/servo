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
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::mediasession::MediaSession;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaMetadata {
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

    pub(crate) fn new(
        global: &Window,
        init: &MediaMetadataInit,
        can_gc: CanGc,
    ) -> DomRoot<MediaMetadata> {
        Self::new_with_proto(global, None, init, can_gc)
    }

    fn new_with_proto(
        global: &Window,
        proto: Option<HandleObject>,
        init: &MediaMetadataInit,
        can_gc: CanGc,
    ) -> DomRoot<MediaMetadata> {
        reflect_dom_object_with_proto(
            Box::new(MediaMetadata::new_inherited(init)),
            global,
            proto,
            can_gc,
        )
    }

    fn queue_update_metadata_algorithm(&self) {
        if self.session.get().is_none() {}
    }

    pub(crate) fn set_session(&self, session: &MediaSession) {
        self.session.set(Some(session));
    }
}

impl MediaMetadataMethods<crate::DomTypeHolder> for MediaMetadata {
    /// <https://w3c.github.io/mediasession/#dom-mediametadata-mediametadata>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        init: &MediaMetadataInit,
    ) -> Fallible<DomRoot<MediaMetadata>> {
        Ok(MediaMetadata::new_with_proto(window, proto, init, can_gc))
    }

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
