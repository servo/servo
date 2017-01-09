/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvr/#interface-vrstageparameters
[Pref="dom.webvr.enabled"]
interface VRStageParameters {
  readonly attribute Float32Array sittingToStandingTransform;
  readonly attribute float sizeX;
  readonly attribute float sizeZ;
};
