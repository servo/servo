/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvr/#interface-vrdisplaycapabilities
[Pref="dom.webvr.enabled"]
interface VRDisplayCapabilities {
  readonly attribute boolean hasPosition;
  readonly attribute boolean hasOrientation;
  readonly attribute boolean hasExternalDisplay;
  readonly attribute boolean canPresent;
  readonly attribute unsigned long maxLayers;
};
