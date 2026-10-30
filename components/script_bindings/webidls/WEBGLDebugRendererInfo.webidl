/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
// skip-unless CARGO_FEATURE_WEBGL

/*
 * WebGL IDL definitions from the Khronos specification:
 * https://www.khronos.org/registry/webgl/extensions/WEBGL_debug_renderer_info/
 */

[LegacyNoInterfaceObject, Exposed=Window]
interface WEBGLDebugRendererInfo {
    const GLenum UNMASKED_VENDOR_WEBGL   = 0x9245;
    const GLenum UNMASKED_RENDERER_WEBGL = 0x9246;
};
