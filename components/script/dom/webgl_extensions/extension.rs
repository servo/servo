/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::reflector::DomObject;
use dom::bindings::root::DomRoot;
use dom::bindings::trace::JSTraceable;
use dom::webglrenderingcontext::WebGLRenderingContext;
use super::WebGLExtensions;

/// Trait implemented by WebGL extensions.
pub trait WebGLExtension: Sized where Self::Extension: DomObject + JSTraceable {
    type Extension;

    /// Creates the DOM object of the WebGL extension.
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<Self::Extension>;

    /// Checks if the extension is supported.
    fn is_supported(ext: &WebGLExtensions) -> bool;

    /// Enable the extension.
    fn enable(ext: &WebGLExtensions);

    /// Name of the WebGL Extension.
    fn name() -> &'static str;
}
