/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://immersive-web.github.io/layers/#xrlayerevent-interface
[SecureContext, Exposed=Window, Pref="dom_webxr_layers_enabled"]
interface XRLayerEvent : Event {
  constructor(DOMString type, XRLayerEventInit eventInitDict);
  [SameObject] readonly attribute XRLayer layer;
};

dictionary XRLayerEventInit : EventInit {
  required XRLayer layer;
};
