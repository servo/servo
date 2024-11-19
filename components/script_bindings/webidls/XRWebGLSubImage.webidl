/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/layers/#xrwebglsubimagetype
[SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
interface XRWebGLSubImage : XRSubImage {
  [SameObject] readonly attribute WebGLTexture colorTexture;
  [SameObject] readonly attribute WebGLTexture? depthStencilTexture;
  readonly attribute unsigned long? imageIndex;
  readonly attribute unsigned long textureWidth;
  readonly attribute unsigned long textureHeight;
};
