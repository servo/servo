/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvr/#interface-vreyeparameters

[Pref="dom.webvr.enabled"]
interface VREyeParameters {
  readonly attribute Float32Array offset;
  [SameObject] readonly attribute VRFieldOfView fieldOfView;
  readonly attribute unsigned long renderWidth;
  readonly attribute unsigned long renderHeight;
};
