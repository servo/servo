use super::{
    constants as webgl, ext_constants as ext, WebGLExtension, WebGLExtensionSpec, WebGLExtensions,
};
use crate::dom::bindings::codegen::Bindings::WebGLDepthTextureBinding;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use canvas_traits::webgl::WebGLVersion;
use sparkle::gl;
use dom_struct::dom_struct;

#[dom_struct]
pub struct WebGLDepthTexture {
    reflector_: Reflector,
}

impl WebGLDepthTexture {
    fn new_inherited() -> WebGLDepthTexture {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for WebGLDepthTexture {
    type Extension = WebGLDepthTexture;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<WebGLDepthTexture> {
        reflect_dom_object(
            Box::new(WebGLDepthTexture::new_inherited()),
            &*ctx.global(),
            WebGLDepthTextureBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_gl_extension("ANGLE_depth_texture")
    }

    fn enable(ext: &WebGLExtensions) {
        let uint248 = gl::UNSIGNED_INT_24_8;
        ext.enable_tex_type(uint248);
        ext.add_effective_tex_internal_format(webgl::RGBA, uint248, ext::RGBA32F);
        ext.add_effective_tex_internal_format(webgl::RGB, uint248, ext::RGB32F);
        ext.add_effective_tex_internal_format(webgl::LUMINANCE, uint248, ext::LUMINANCE32F_ARB);
        ext.add_effective_tex_internal_format(webgl::ALPHA, uint248, ext::ALPHA32F_ARB);
        ext.add_effective_tex_internal_format(
            webgl::LUMINANCE_ALPHA,
            uint248,
            ext::LUMINANCE_ALPHA32F_ARB,
        );
    }

    fn name() -> &'static str {
        "ANGLE_depth_texture"
    }
}
