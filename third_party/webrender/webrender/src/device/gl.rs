/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::super::shader_source::{OPTIMIZED_SHADERS, UNOPTIMIZED_SHADERS};
use api::{ColorF, ImageDescriptor, ImageFormat};
use api::{MixBlendMode, ImageBufferKind, VoidPtrToSizeFn};
use api::{CrashAnnotator, CrashAnnotation, CrashAnnotatorGuard};
use api::units::*;
use euclid::default::Transform3D;
use gleam::gl;
use crate::render_api::MemoryReport;
use crate::internal_types::{FastHashMap, RenderTargetInfo, Swizzle, SwizzleSettings};
use crate::util::round_up_to_multiple;
use crate::profiler;
use log::Level;
use smallvec::SmallVec;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    cmp,
    collections::hash_map::Entry,
    marker::PhantomData,
    mem,
    num::NonZeroUsize,
    os::raw::c_void,
    ops::Add,
    path::PathBuf,
    ptr,
    rc::Rc,
    slice,
    sync::Arc,
    thread,
    time::Duration,
};
use webrender_build::shader::{
    ProgramSourceDigest, ShaderKind, ShaderVersion, build_shader_main_string,
    build_shader_prefix_string, do_build_shader_string, shader_source_from_file,
};
use malloc_size_of::MallocSizeOfOps;

/// Sequence number for frames, as tracked by the device layer.
#[derive(Debug, Copy, Clone, PartialEq, Ord, Eq, PartialOrd)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GpuFrameId(usize);

impl GpuFrameId {
    pub fn new(value: usize) -> Self {
        GpuFrameId(value)
    }
}

impl Add<usize> for GpuFrameId {
    type Output = GpuFrameId;

    fn add(self, other: usize) -> GpuFrameId {
        GpuFrameId(self.0 + other)
    }
}

pub struct TextureSlot(pub usize);

// In some places we need to temporarily bind a texture to any slot.
const DEFAULT_TEXTURE: TextureSlot = TextureSlot(0);

#[repr(u32)]
pub enum DepthFunction {
    Always = gl::ALWAYS,
    Less = gl::LESS,
    LessEqual = gl::LEQUAL,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum TextureFilter {
    Nearest,
    Linear,
    Trilinear,
}

/// A structure defining a particular workflow of texture transfers.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct TextureFormatPair<T> {
    /// Format the GPU natively stores texels in.
    pub internal: T,
    /// Format we expect the users to provide the texels in.
    pub external: T,
}

impl<T: Copy> From<T> for TextureFormatPair<T> {
    fn from(value: T) -> Self {
        TextureFormatPair {
            internal: value,
            external: value,
        }
    }
}

#[derive(Debug)]
pub enum VertexAttributeKind {
    F32,
    U8Norm,
    U16Norm,
    I32,
    U16,
}

#[derive(Debug)]
pub struct VertexAttribute {
    pub name: &'static str,
    pub count: u32,
    pub kind: VertexAttributeKind,
}

#[derive(Debug)]
pub struct VertexDescriptor {
    pub vertex_attributes: &'static [VertexAttribute],
    pub instance_attributes: &'static [VertexAttribute],
}

enum FBOTarget {
    Read,
    Draw,
}

/// Method of uploading texel data from CPU to GPU.
#[derive(Debug, Clone)]
pub enum UploadMethod {
    /// Just call `glTexSubImage` directly with the CPU data pointer
    Immediate,
    /// Accumulate the changes in PBO first before transferring to a texture.
    PixelBuffer(VertexUsageHint),
}

/// Plain old data that can be used to initialize a texture.
pub unsafe trait Texel: Copy {}
unsafe impl Texel for u8 {}
unsafe impl Texel for f32 {}

/// Returns the size in bytes of a depth target with the given dimensions.
fn depth_target_size_in_bytes(dimensions: &DeviceIntSize) -> usize {
    // DEPTH24 textures generally reserve 3 bytes for depth and 1 byte
    // for stencil, so we measure them as 32 bits.
    let pixels = dimensions.width * dimensions.height;
    (pixels as usize) * 4
}

pub fn get_gl_target(target: ImageBufferKind) -> gl::GLuint {
    match target {
        ImageBufferKind::Texture2D => gl::TEXTURE_2D,
        ImageBufferKind::TextureRect => gl::TEXTURE_RECTANGLE,
        ImageBufferKind::TextureExternal => gl::TEXTURE_EXTERNAL_OES,
    }
}

pub fn from_gl_target(target: gl::GLuint) -> ImageBufferKind {
    match target {
        gl::TEXTURE_2D => ImageBufferKind::Texture2D,
        gl::TEXTURE_RECTANGLE => ImageBufferKind::TextureRect,
        gl::TEXTURE_EXTERNAL_OES => ImageBufferKind::TextureExternal,
        _ => panic!("Unexpected target {:?}", target),
    }
}

fn supports_extension(extensions: &[String], extension: &str) -> bool {
    extensions.iter().any(|s| s == extension)
}

fn get_shader_version(gl: &dyn gl::Gl) -> ShaderVersion {
    match gl.get_type() {
        gl::GlType::Gl => ShaderVersion::Gl,
        gl::GlType::Gles => ShaderVersion::Gles,
    }
}

// Get an unoptimized shader string by name, from the built in resources or
// an override path, if supplied.
pub fn get_unoptimized_shader_source(shader_name: &str, base_path: Option<&PathBuf>) -> Cow<'static, str> {
    if let Some(ref base) = base_path {
        let shader_path = base.join(&format!("{}.glsl", shader_name));
        Cow::Owned(shader_source_from_file(&shader_path))
    } else {
        Cow::Borrowed(
            UNOPTIMIZED_SHADERS
            .get(shader_name)
            .expect("Shader not found")
            .source
        )
    }
}

pub trait FileWatcherHandler: Send {
    fn file_changed(&self, path: PathBuf);
}

impl VertexAttributeKind {
    fn size_in_bytes(&self) -> u32 {
        match *self {
            VertexAttributeKind::F32 => 4,
            VertexAttributeKind::U8Norm => 1,
            VertexAttributeKind::U16Norm => 2,
            VertexAttributeKind::I32 => 4,
            VertexAttributeKind::U16 => 2,
        }
    }
}

impl VertexAttribute {
    fn size_in_bytes(&self) -> u32 {
        self.count * self.kind.size_in_bytes()
    }

    fn bind_to_vao(
        &self,
        attr_index: gl::GLuint,
        divisor: gl::GLuint,
        stride: gl::GLint,
        offset: gl::GLuint,
        gl: &dyn gl::Gl,
    ) {
        gl.enable_vertex_attrib_array(attr_index);
        gl.vertex_attrib_divisor(attr_index, divisor);

        match self.kind {
            VertexAttributeKind::F32 => {
                gl.vertex_attrib_pointer(
                    attr_index,
                    self.count as gl::GLint,
                    gl::FLOAT,
                    false,
                    stride,
                    offset,
                );
            }
            VertexAttributeKind::U8Norm => {
                gl.vertex_attrib_pointer(
                    attr_index,
                    self.count as gl::GLint,
                    gl::UNSIGNED_BYTE,
                    true,
                    stride,
                    offset,
                );
            }
            VertexAttributeKind::U16Norm => {
                gl.vertex_attrib_pointer(
                    attr_index,
                    self.count as gl::GLint,
                    gl::UNSIGNED_SHORT,
                    true,
                    stride,
                    offset,
                );
            }
            VertexAttributeKind::I32 => {
                gl.vertex_attrib_i_pointer(
                    attr_index,
                    self.count as gl::GLint,
                    gl::INT,
                    stride,
                    offset,
                );
            }
            VertexAttributeKind::U16 => {
                gl.vertex_attrib_i_pointer(
                    attr_index,
                    self.count as gl::GLint,
                    gl::UNSIGNED_SHORT,
                    stride,
                    offset,
                );
            }
        }
    }
}

impl VertexDescriptor {
    fn instance_stride(&self) -> u32 {
        self.instance_attributes
            .iter()
            .map(|attr| attr.size_in_bytes())
            .sum()
    }

    fn bind_attributes(
        attributes: &[VertexAttribute],
        start_index: usize,
        divisor: u32,
        gl: &dyn gl::Gl,
        vbo: VBOId,
    ) {
        vbo.bind(gl);

        let stride: u32 = attributes
            .iter()
            .map(|attr| attr.size_in_bytes())
            .sum();

        let mut offset = 0;
        for (i, attr) in attributes.iter().enumerate() {
            let attr_index = (start_index + i) as gl::GLuint;
            attr.bind_to_vao(attr_index, divisor, stride as _, offset, gl);
            offset += attr.size_in_bytes();
        }
    }

    fn bind(&self, gl: &dyn gl::Gl, main: VBOId, instance: VBOId, instance_divisor: u32) {
        Self::bind_attributes(self.vertex_attributes, 0, 0, gl, main);

        if !self.instance_attributes.is_empty() {
            Self::bind_attributes(
                self.instance_attributes,
                self.vertex_attributes.len(),
                instance_divisor,
                gl,
                instance,
            );
        }
    }
}

impl VBOId {
    fn bind(&self, gl: &dyn gl::Gl) {
        gl.bind_buffer(gl::ARRAY_BUFFER, self.0);
    }
}

impl IBOId {
    fn bind(&self, gl: &dyn gl::Gl) {
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.0);
    }
}

impl FBOId {
    fn bind(&self, gl: &dyn gl::Gl, target: FBOTarget) {
        let target = match target {
            FBOTarget::Read => gl::READ_FRAMEBUFFER,
            FBOTarget::Draw => gl::DRAW_FRAMEBUFFER,
        };
        gl.bind_framebuffer(target, self.0);
    }
}

pub struct Stream<'a> {
    attributes: &'a [VertexAttribute],
    vbo: VBOId,
}

pub struct VBO<V> {
    id: gl::GLuint,
    target: gl::GLenum,
    allocated_count: usize,
    marker: PhantomData<V>,
}

impl<V> VBO<V> {
    pub fn allocated_count(&self) -> usize {
        self.allocated_count
    }

    pub fn stream_with<'a>(&self, attributes: &'a [VertexAttribute]) -> Stream<'a> {
        debug_assert_eq!(
            mem::size_of::<V>(),
            attributes.iter().map(|a| a.size_in_bytes() as usize).sum::<usize>()
        );
        Stream {
            attributes,
            vbo: VBOId(self.id),
        }
    }
}

impl<T> Drop for VBO<T> {
    fn drop(&mut self) {
        debug_assert!(thread::panicking() || self.id == 0);
    }
}

#[cfg_attr(feature = "replay", derive(Clone))]
#[derive(Debug)]
pub struct ExternalTexture {
    id: gl::GLuint,
    target: gl::GLuint,
    swizzle: Swizzle,
    uv_rect: TexelRect,
}

impl ExternalTexture {
    pub fn new(
        id: u32,
        target: ImageBufferKind,
        swizzle: Swizzle,
        uv_rect: TexelRect,
    ) -> Self {
        ExternalTexture {
            id,
            target: get_gl_target(target),
            swizzle,
            uv_rect,
        }
    }

    #[cfg(feature = "replay")]
    pub fn internal_id(&self) -> gl::GLuint {
        self.id
    }

    pub fn get_uv_rect(&self) -> TexelRect {
        self.uv_rect
    }
}

bitflags! {
    #[derive(Default)]
    pub struct TextureFlags: u32 {
        /// This texture corresponds to one of the shared texture caches.
        const IS_SHARED_TEXTURE_CACHE = 1 << 0;
    }
}

/// WebRender interface to an OpenGL texture.
///
/// Because freeing a texture requires various device handles that are not
/// reachable from this struct, manual destruction via `Device` is required.
/// Our `Drop` implementation asserts that this has happened.
#[derive(Debug)]
pub struct Texture {
    id: gl::GLuint,
    target: gl::GLuint,
    format: ImageFormat,
    size: DeviceIntSize,
    filter: TextureFilter,
    flags: TextureFlags,
    /// An internally mutable swizzling state that may change between batches.
    active_swizzle: Cell<Swizzle>,
    /// Framebuffer Object allowing this texture to be rendered to.
    ///
    /// Empty if this texture is not used as a render target or if a depth buffer is needed.
    fbo: Option<FBOId>,
    /// Same as the above, but with a depth buffer attached.
    ///
    /// FBOs are cheap to create but expensive to reconfigure (since doing so
    /// invalidates framebuffer completeness caching). Moreover, rendering with
    /// a depth buffer attached but the depth write+test disabled relies on the
    /// driver to optimize it out of the rendering pass, which most drivers
    /// probably do but, according to jgilbert, is best not to rely on.
    ///
    /// So we lazily generate a second list of FBOs with depth. This list is
    /// empty if this texture is not used as a render target _or_ if it is, but
    /// the depth buffer has never been requested.
    ///
    /// Note that we always fill fbo, and then lazily create fbo_with_depth
    /// when needed. We could make both lazy (i.e. render targets would have one
    /// or the other, but not both, unless they were actually used in both
    /// configurations). But that would complicate a lot of logic in this module,
    /// and FBOs are cheap enough to create.
    fbo_with_depth: Option<FBOId>,
    last_frame_used: GpuFrameId,
}

impl Texture {
    pub fn get_dimensions(&self) -> DeviceIntSize {
        self.size
    }

    pub fn get_format(&self) -> ImageFormat {
        self.format
    }

    pub fn get_filter(&self) -> TextureFilter {
        self.filter
    }

    pub fn get_target(&self) -> ImageBufferKind {
        from_gl_target(self.target)
    }

    pub fn supports_depth(&self) -> bool {
        self.fbo_with_depth.is_some()
    }

    pub fn last_frame_used(&self) -> GpuFrameId {
        self.last_frame_used
    }

    pub fn used_in_frame(&self, frame_id: GpuFrameId) -> bool {
        self.last_frame_used == frame_id
    }

    pub fn is_render_target(&self) -> bool {
        self.fbo.is_some()
    }

    /// Returns true if this texture was used within `threshold` frames of
    /// the current frame.
    pub fn used_recently(&self, current_frame_id: GpuFrameId, threshold: usize) -> bool {
        self.last_frame_used + threshold >= current_frame_id
    }

    /// Returns the flags for this texture.
    pub fn flags(&self) -> &TextureFlags {
        &self.flags
    }

    /// Returns a mutable borrow of the flags for this texture.
    pub fn flags_mut(&mut self) -> &mut TextureFlags {
        &mut self.flags
    }

    /// Returns the number of bytes (generally in GPU memory) that this texture
    /// consumes.
    pub fn size_in_bytes(&self) -> usize {
        let bpp = self.format.bytes_per_pixel() as usize;
        let w = self.size.width as usize;
        let h = self.size.height as usize;
        bpp * w * h
    }

    #[cfg(feature = "replay")]
    pub fn into_external(mut self) -> ExternalTexture {
        let ext = ExternalTexture {
            id: self.id,
            target: self.target,
            swizzle: Swizzle::default(),
            // TODO(gw): Support custom UV rect for external textures during captures
            uv_rect: TexelRect::new(
                0.0,
                0.0,
                self.size.width as f32,
                self.size.height as f32,
            ),
        };
        self.id = 0; // don't complain, moved out
        ext
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        debug_assert!(thread::panicking() || self.id == 0);
    }
}

pub struct Program {
    id: gl::GLuint,
    u_transform: gl::GLint,
    u_mode: gl::GLint,
    u_texture_size: gl::GLint,
    source_info: ProgramSourceInfo,
    is_initialized: bool,
}

impl Program {
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        debug_assert!(
            thread::panicking() || self.id == 0,
            "renderer::deinit not called"
        );
    }
}

pub struct CustomVAO {
    id: gl::GLuint,
}

impl Drop for CustomVAO {
    fn drop(&mut self) {
        debug_assert!(
            thread::panicking() || self.id == 0,
            "renderer::deinit not called"
        );
    }
}

pub struct VAO {
    id: gl::GLuint,
    ibo_id: IBOId,
    main_vbo_id: VBOId,
    instance_vbo_id: VBOId,
    instance_stride: usize,
    instance_divisor: u32,
    owns_vertices_and_indices: bool,
}

impl Drop for VAO {
    fn drop(&mut self) {
        debug_assert!(
            thread::panicking() || self.id == 0,
            "renderer::deinit not called"
        );
    }
}

#[derive(Debug)]
pub struct PBO {
    id: gl::GLuint,
    reserved_size: usize,
}

impl PBO {
    pub fn get_reserved_size(&self) -> usize {
        self.reserved_size
    }
}

impl Drop for PBO {
    fn drop(&mut self) {
        debug_assert!(
            thread::panicking() || self.id == 0,
            "renderer::deinit not called or PBO not returned to pool"
        );
    }
}

pub struct BoundPBO<'a> {
    device: &'a mut Device,
    pub data: &'a [u8]
}

impl<'a> Drop for BoundPBO<'a> {
    fn drop(&mut self) {
        self.device.gl.unmap_buffer(gl::PIXEL_PACK_BUFFER);
        self.device.gl.bind_buffer(gl::PIXEL_PACK_BUFFER, 0);
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct FBOId(gl::GLuint);

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct RBOId(gl::GLuint);

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct VBOId(gl::GLuint);

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
struct IBOId(gl::GLuint);

#[derive(Clone, Debug)]
enum ProgramSourceType {
    Unoptimized,
    Optimized(ShaderVersion),
}

#[derive(Clone, Debug)]
pub struct ProgramSourceInfo {
    base_filename: &'static str,
    features: Vec<&'static str>,
    full_name_cstr: Rc<std::ffi::CString>,
    source_type: ProgramSourceType,
    digest: ProgramSourceDigest,
}

impl ProgramSourceInfo {
    fn new(
        device: &Device,
        name: &'static str,
        features: &[&'static str],
    ) -> Self {

        // Compute the digest. Assuming the device has a `ProgramCache`, this
        // will always be needed, whereas the source is rarely needed.

        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        // Setup.
        let mut hasher = DefaultHasher::new();
        let gl_version = get_shader_version(&*device.gl());

        // Hash the renderer name.
        hasher.write(device.capabilities.renderer_name.as_bytes());

        let full_name = Self::make_full_name(name, features);

        let optimized_source = if device.use_optimized_shaders {
            OPTIMIZED_SHADERS.get(&(gl_version, &full_name)).or_else(|| {
                warn!("Missing optimized shader source for {}", &full_name);
                None
            })
        } else {
            None
        };

        let source_type = match optimized_source {
            Some(source_and_digest) => {
                // Optimized shader sources are used as-is, without any run-time processing.
                // The vertex and fragment shaders are different, so must both be hashed.
                // We use the hashes that were computed at build time, and verify it in debug builds.
                if cfg!(debug_assertions) {
                    let mut h = DefaultHasher::new();
                    h.write(source_and_digest.vert_source.as_bytes());
                    h.write(source_and_digest.frag_source.as_bytes());
                    let d: ProgramSourceDigest = h.into();
                    let digest = d.to_string();
                    debug_assert_eq!(digest, source_and_digest.digest);
                    hasher.write(digest.as_bytes());
                } else {
                    hasher.write(source_and_digest.digest.as_bytes());
                }

                ProgramSourceType::Optimized(gl_version)
            }
            None => {
                // For non-optimized sources we compute the hash by walking the static strings
                // in the same order as we would when concatenating the source, to avoid
                // heap-allocating in the common case.
                //
                // Note that we cheat a bit to make the hashing more efficient. First, the only
                // difference between the vertex and fragment shader is a single deterministic
                // define, so we don't need to hash both. Second, we precompute the digest of the
                // expanded source file at build time, and then just hash that digest here.
                let override_path = device.resource_override_path.as_ref();
                let source_and_digest = UNOPTIMIZED_SHADERS.get(&name).expect("Shader not found");

                // Hash the prefix string.
                build_shader_prefix_string(
                    gl_version,
                    &features,
                    ShaderKind::Vertex,
                    &name,
                    &mut |s| hasher.write(s.as_bytes()),
                );

                // Hash the shader file contents. We use a precomputed digest, and
                // verify it in debug builds.
                if override_path.is_some() || cfg!(debug_assertions) {
                    let mut h = DefaultHasher::new();
                    build_shader_main_string(
                        &name,
                        &|f| get_unoptimized_shader_source(f, override_path),
                        &mut |s| h.write(s.as_bytes())
                    );
                    let d: ProgramSourceDigest = h.into();
                    let digest = format!("{}", d);
                    debug_assert!(override_path.is_some() || digest == source_and_digest.digest);
                    hasher.write(digest.as_bytes());
                } else {
                    hasher.write(source_and_digest.digest.as_bytes());
                }

                ProgramSourceType::Unoptimized
            }
        };

        // Finish.
        ProgramSourceInfo {
            base_filename: name,
            features: features.to_vec(),
            full_name_cstr: Rc::new(std::ffi::CString::new(full_name).unwrap()),
            source_type,
            digest: hasher.into(),
        }
    }

    fn compute_source(&self, device: &Device, kind: ShaderKind) -> String {
        let full_name = self.full_name();
        match self.source_type {
            ProgramSourceType::Optimized(gl_version) => {
                let shader = OPTIMIZED_SHADERS
                    .get(&(gl_version, &full_name))
                    .unwrap_or_else(|| panic!("Missing optimized shader source for {}", full_name));

                match kind {
                    ShaderKind::Vertex => shader.vert_source.to_string(),
                    ShaderKind::Fragment => shader.frag_source.to_string(),
                }
            },
            ProgramSourceType::Unoptimized => {
                let mut src = String::new();
                device.build_shader_string(
                    &self.features,
                    kind,
                    self.base_filename,
                    |s| src.push_str(s),
                );
                src
            }
        }
    }

    fn make_full_name(base_filename: &'static str, features: &[&'static str]) -> String {
        if features.is_empty() {
            base_filename.to_string()
        } else {
            format!("{}_{}", base_filename, features.join("_"))
        }
    }

    fn full_name(&self) -> String {
        Self::make_full_name(self.base_filename, &self.features)
    }
}

#[cfg_attr(feature = "serialize_program", derive(Deserialize, Serialize))]
pub struct ProgramBinary {
    bytes: Vec<u8>,
    format: gl::GLenum,
    source_digest: ProgramSourceDigest,
}

impl ProgramBinary {
    fn new(bytes: Vec<u8>,
           format: gl::GLenum,
           source_digest: ProgramSourceDigest) -> Self {
        ProgramBinary {
            bytes,
            format,
            source_digest,
        }
    }

    /// Returns a reference to the source digest hash.
    pub fn source_digest(&self) -> &ProgramSourceDigest {
        &self.source_digest
    }
}

/// The interfaces that an application can implement to handle ProgramCache update
pub trait ProgramCacheObserver {
    fn save_shaders_to_disk(&self, entries: Vec<Arc<ProgramBinary>>);
    fn set_startup_shaders(&self, entries: Vec<Arc<ProgramBinary>>);
    fn try_load_shader_from_disk(&self, digest: &ProgramSourceDigest, program_cache: &Rc<ProgramCache>);
    fn notify_program_binary_failed(&self, program_binary: &Arc<ProgramBinary>);
}

struct ProgramCacheEntry {
    /// The binary.
    binary: Arc<ProgramBinary>,
    /// True if the binary has been linked, i.e. used for rendering.
    linked: bool,
}

pub struct ProgramCache {
    entries: RefCell<FastHashMap<ProgramSourceDigest, ProgramCacheEntry>>,

    /// Optional trait object that allows the client
    /// application to handle ProgramCache updating
    program_cache_handler: Option<Box<dyn ProgramCacheObserver>>,

    /// Programs that have not yet been cached to disk (by program_cache_handler)
    pending_entries: RefCell<Vec<Arc<ProgramBinary>>>,
}

impl ProgramCache {
    pub fn new(program_cache_observer: Option<Box<dyn ProgramCacheObserver>>) -> Rc<Self> {
        Rc::new(
            ProgramCache {
                entries: RefCell::new(FastHashMap::default()),
                program_cache_handler: program_cache_observer,
                pending_entries: RefCell::new(Vec::default()),
            }
        )
    }

    /// Save any new program binaries to the disk cache, and if startup has
    /// just completed then write the list of shaders to load on next startup.
    fn update_disk_cache(&self, startup_complete: bool) {
        if let Some(ref handler) = self.program_cache_handler {
            if !self.pending_entries.borrow().is_empty() {
                let pending_entries = self.pending_entries.replace(Vec::default());
                handler.save_shaders_to_disk(pending_entries);
            }

            if startup_complete {
                let startup_shaders = self.entries.borrow().values()
                    .filter(|e| e.linked).map(|e| e.binary.clone())
                    .collect::<Vec<_>>();
                handler.set_startup_shaders(startup_shaders);
            }
        }
    }

    /// Add a new ProgramBinary to the cache.
    /// This function is typically used after compiling and linking a new program.
    /// The binary will be saved to disk the next time update_disk_cache() is called.
    fn add_new_program_binary(&self, program_binary: Arc<ProgramBinary>) {
        self.pending_entries.borrow_mut().push(program_binary.clone());

        let digest = program_binary.source_digest.clone();
        let entry = ProgramCacheEntry {
            binary: program_binary,
            linked: true,
        };
        self.entries.borrow_mut().insert(digest, entry);
    }

    /// Load ProgramBinary to ProgramCache.
    /// The function is typically used to load ProgramBinary from disk.
    #[cfg(feature = "serialize_program")]
    pub fn load_program_binary(&self, program_binary: Arc<ProgramBinary>) {
        let digest = program_binary.source_digest.clone();
        let entry = ProgramCacheEntry {
            binary: program_binary,
            linked: false,
        };
        self.entries.borrow_mut().insert(digest, entry);
    }

    /// Returns the number of bytes allocated for shaders in the cache.
    pub fn report_memory(&self, op: VoidPtrToSizeFn) -> usize {
        self.entries.borrow().values()
            .map(|e| unsafe { op(e.binary.bytes.as_ptr() as *const c_void ) })
            .sum()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VertexUsageHint {
    Static,
    Dynamic,
    Stream,
}

impl VertexUsageHint {
    fn to_gl(&self) -> gl::GLuint {
        match *self {
            VertexUsageHint::Static => gl::STATIC_DRAW,
            VertexUsageHint::Dynamic => gl::DYNAMIC_DRAW,
            VertexUsageHint::Stream => gl::STREAM_DRAW,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UniformLocation(gl::GLint);

impl UniformLocation {
    pub const INVALID: Self = UniformLocation(-1);
}

#[derive(Debug)]
pub struct Capabilities {
    /// Whether multisampled render targets are supported.
    pub supports_multisampling: bool,
    /// Whether the function `glCopyImageSubData` is available.
    pub supports_copy_image_sub_data: bool,
    /// Whether the RGBAF32 textures can be bound to framebuffers.
    pub supports_color_buffer_float: bool,
    /// Whether the device supports persistently mapped buffers, via glBufferStorage.
    pub supports_buffer_storage: bool,
    /// Whether advanced blend equations are supported.
    pub supports_advanced_blend_equation: bool,
    /// Whether dual-source blending is supported.
    pub supports_dual_source_blending: bool,
    /// Whether KHR_debug is supported for getting debug messages from
    /// the driver.
    pub supports_khr_debug: bool,
    /// Whether we can configure texture units to do swizzling on sampling.
    pub supports_texture_swizzle: bool,
    /// Whether the driver supports uploading to textures from a non-zero
    /// offset within a PBO.
    pub supports_nonzero_pbo_offsets: bool,
    /// Whether the driver supports specifying the texture usage up front.
    pub supports_texture_usage: bool,
    /// Whether offscreen render targets can be partially updated.
    pub supports_render_target_partial_update: bool,
    /// Whether we can use SSBOs.
    pub supports_shader_storage_object: bool,
    /// Whether to enforce that texture uploads be batched regardless of what
    /// the pref says.
    pub requires_batched_texture_uploads: Option<bool>,
    /// Whether we are able to ue glClear to clear regions of an alpha render target.
    /// If false, we must use a shader to clear instead.
    pub supports_alpha_target_clears: bool,
    /// Whether the driver can reliably upload data to R8 format textures.
    pub supports_r8_texture_upload: bool,
    /// Whether clip-masking is supported natively by the GL implementation
    /// rather than emulated in shaders.
    pub uses_native_clip_mask: bool,
    /// Whether anti-aliasing is supported natively by the GL implementation
    /// rather than emulated in shaders.
    pub uses_native_antialiasing: bool,
    /// Whether the extension GL_OES_EGL_image_external_essl3 is supported. If true, external
    /// textures can be used as normal. If false, external textures can only be rendered with
    /// certain shaders, and must first be copied in to regular textures for others.
    pub supports_image_external_essl3: bool,
    /// The name of the renderer, as reported by GL
    pub renderer_name: String,
}

#[derive(Clone, Debug)]
pub enum ShaderError {
    Compilation(String, String), // name, error message
    Link(String, String),        // name, error message
}

/// A refcounted depth target, which may be shared by multiple textures across
/// the device.
struct SharedDepthTarget {
    /// The Render Buffer Object representing the depth target.
    rbo_id: RBOId,
    /// Reference count. When this drops to zero, the RBO is deleted.
    refcount: usize,
}

#[cfg(debug_assertions)]
impl Drop for SharedDepthTarget {
    fn drop(&mut self) {
        debug_assert!(thread::panicking() || self.refcount == 0);
    }
}

/// Describes for which texture formats to use the glTexStorage*
/// family of functions.
#[derive(PartialEq, Debug)]
enum TexStorageUsage {
    Never,
    NonBGRA8,
    Always,
}

/// Describes a required alignment for a stride,
/// which can either be represented in bytes or pixels.
#[derive(Copy, Clone, Debug)]
pub enum StrideAlignment {
    Bytes(NonZeroUsize),
    Pixels(NonZeroUsize),
}

impl StrideAlignment {
    pub fn num_bytes(&self, format: ImageFormat) -> NonZeroUsize {
        match *self {
            Self::Bytes(bytes) => bytes,
            Self::Pixels(pixels) => {
                assert!(format.bytes_per_pixel() > 0);
                NonZeroUsize::new(pixels.get() * format.bytes_per_pixel() as usize).unwrap()
            }
        }
    }
}

// We get 24 bits of Z value - use up 22 bits of it to give us
// 4 bits to account for GPU issues. This seems to manifest on
// some GPUs under certain perspectives due to z interpolation
// precision problems.
const RESERVE_DEPTH_BITS: i32 = 2;

pub struct Device {
    gl: Rc<dyn gl::Gl>,

    /// If non-None, |gl| points to a profiling wrapper, and this points to the
    /// underling Gl instance.
    base_gl: Option<Rc<dyn gl::Gl>>,

    // device state
    bound_textures: [gl::GLuint; 16],
    bound_program: gl::GLuint,
    bound_program_name: Rc<std::ffi::CString>,
    bound_vao: gl::GLuint,
    bound_read_fbo: (FBOId, DeviceIntPoint),
    bound_draw_fbo: FBOId,
    program_mode_id: UniformLocation,
    default_read_fbo: FBOId,
    default_draw_fbo: FBOId,

    /// Track depth state for assertions. Note that the default FBO has depth,
    /// so this defaults to true.
    depth_available: bool,

    upload_method: UploadMethod,
    use_batched_texture_uploads: bool,
    /// Whether to use draw calls instead of regular blitting commands.
    ///
    /// Note: this currently only applies to the batched texture uploads
    /// path.
    use_draw_calls_for_texture_copy: bool,

    // HW or API capabilities
    capabilities: Capabilities,

    color_formats: TextureFormatPair<ImageFormat>,
    bgra_formats: TextureFormatPair<gl::GLuint>,
    bgra_pixel_type: gl::GLuint,
    swizzle_settings: SwizzleSettings,
    depth_format: gl::GLuint,

    /// Map from texture dimensions to shared depth buffers for render targets.
    ///
    /// Render targets often have the same width/height, so we can save memory
    /// by sharing these across targets.
    depth_targets: FastHashMap<DeviceIntSize, SharedDepthTarget>,

    // debug
    inside_frame: bool,
    crash_annotator: Option<Box<dyn CrashAnnotator>>,

    // resources
    resource_override_path: Option<PathBuf>,

    /// Whether to use shaders that have been optimized at build time.
    use_optimized_shaders: bool,

    max_texture_size: i32,
    cached_programs: Option<Rc<ProgramCache>>,

    // Frame counter. This is used to map between CPU
    // frames and GPU frames.
    frame_id: GpuFrameId,

    /// When to use glTexStorage*. We prefer this over glTexImage* because it
    /// guarantees that mipmaps won't be generated (which they otherwise are on
    /// some drivers, particularly ANGLE). However, it is not always supported
    /// at all, or for BGRA8 format. If it's not supported for the required
    /// format, we fall back to glTexImage*.
    texture_storage_usage: TexStorageUsage,

    /// Required stride alignment for pixel transfers. This may be required for
    /// correctness reasons due to driver bugs, or for performance reasons to
    /// ensure we remain on the fast-path for transfers.
    required_pbo_stride: StrideAlignment,

    /// Whether we must ensure the source strings passed to glShaderSource()
    /// are null-terminated, to work around driver bugs.
    requires_null_terminated_shader_source: bool,

    /// Whether we must unbind any texture from GL_TEXTURE_EXTERNAL_OES before
    /// binding to GL_TEXTURE_2D, to work around an android emulator bug.
    requires_texture_external_unbind: bool,

    // GL extensions
    extensions: Vec<String>,

    /// Dumps the source of the shader with the given name
    dump_shader_source: Option<String>,

    surface_origin_is_top_left: bool,

    /// A debug boolean for tracking if the shader program has been set after
    /// a blend mode change.
    ///
    /// This is needed for compatibility with next-gen
    /// GPU APIs that switch states using "pipeline object" that bundles
    /// together the blending state with the shader.
    ///
    /// Having the constraint of always binding the shader last would allow
    /// us to have the "pipeline object" bound at that time. Without this
    /// constraint, we'd either have to eagerly bind the "pipeline object"
    /// on changing either the shader or the blend more, or lazily bind it
    /// at draw call time, neither of which is desirable.
    #[cfg(debug_assertions)]
    shader_is_ready: bool,
}

/// Contains the parameters necessary to bind a draw target.
#[derive(Clone, Copy, Debug)]
pub enum DrawTarget {
    /// Use the device's default draw target, with the provided dimensions,
    /// which are used to set the viewport.
    Default {
        /// Target rectangle to draw.
        rect: FramebufferIntRect,
        /// Total size of the target.
        total_size: FramebufferIntSize,
        surface_origin_is_top_left: bool,
    },
    /// Use the provided texture.
    Texture {
        /// Size of the texture in pixels
        dimensions: DeviceIntSize,
        /// Whether to draw with the texture's associated depth target
        with_depth: bool,
        /// FBO that corresponds to the selected layer / depth mode
        fbo_id: FBOId,
        /// Native GL texture ID
        id: gl::GLuint,
        /// Native GL texture target
        target: gl::GLuint,
    },
    /// Use an FBO attached to an external texture.
    External {
        fbo: FBOId,
        size: FramebufferIntSize,
    },
    /// An OS compositor surface
    NativeSurface {
        offset: DeviceIntPoint,
        external_fbo_id: u32,
        dimensions: DeviceIntSize,
    },
}

impl DrawTarget {
    pub fn new_default(size: DeviceIntSize, surface_origin_is_top_left: bool) -> Self {
        let total_size = device_size_as_framebuffer_size(size);
        DrawTarget::Default {
            rect: total_size.into(),
            total_size,
            surface_origin_is_top_left,
        }
    }

    /// Returns true if this draw target corresponds to the default framebuffer.
    pub fn is_default(&self) -> bool {
        match *self {
            DrawTarget::Default {..} => true,
            _ => false,
        }
    }

    pub fn from_texture(
        texture: &Texture,
        with_depth: bool,
    ) -> Self {
        let fbo_id = if with_depth {
            texture.fbo_with_depth.unwrap()
        } else {
            texture.fbo.unwrap()
        };

        DrawTarget::Texture {
            dimensions: texture.get_dimensions(),
            fbo_id,
            with_depth,
            id: texture.id,
            target: texture.target,
        }
    }

    /// Returns the dimensions of this draw-target.
    pub fn dimensions(&self) -> DeviceIntSize {
        match *self {
            DrawTarget::Default { total_size, .. } => total_size.cast_unit(),
            DrawTarget::Texture { dimensions, .. } => dimensions,
            DrawTarget::External { size, .. } => size.cast_unit(),
            DrawTarget::NativeSurface { dimensions, .. } => dimensions,
        }
    }

    pub fn to_framebuffer_rect(&self, device_rect: DeviceIntRect) -> FramebufferIntRect {
        let mut fb_rect = device_rect_as_framebuffer_rect(&device_rect);
        match *self {
            DrawTarget::Default { ref rect, surface_origin_is_top_left, .. } => {
                // perform a Y-flip here
                if !surface_origin_is_top_left {
                    fb_rect.origin.y = rect.origin.y + rect.size.height - fb_rect.origin.y - fb_rect.size.height;
                    fb_rect.origin.x += rect.origin.x;
                }
            }
            DrawTarget::Texture { .. } | DrawTarget::External { .. } | DrawTarget::NativeSurface { .. } => (),
        }
        fb_rect
    }

    pub fn surface_origin_is_top_left(&self) -> bool {
        match *self {
            DrawTarget::Default { surface_origin_is_top_left, .. } => surface_origin_is_top_left,
            DrawTarget::Texture { .. } | DrawTarget::External { .. } | DrawTarget::NativeSurface { .. } => true,
        }
    }

    /// Given a scissor rect, convert it to the right coordinate space
    /// depending on the draw target kind. If no scissor rect was supplied,
    /// returns a scissor rect that encloses the entire render target.
    pub fn build_scissor_rect(
        &self,
        scissor_rect: Option<DeviceIntRect>,
    ) -> FramebufferIntRect {
        let dimensions = self.dimensions();

        match scissor_rect {
            Some(scissor_rect) => match *self {
                DrawTarget::Default { ref rect, .. } => {
                    self.to_framebuffer_rect(scissor_rect)
                        .intersection(rect)
                        .unwrap_or_else(FramebufferIntRect::zero)
                }
                DrawTarget::NativeSurface { offset, .. } => {
                    device_rect_as_framebuffer_rect(&scissor_rect.translate(offset.to_vector()))
                }
                DrawTarget::Texture { .. } | DrawTarget::External { .. } => {
                    device_rect_as_framebuffer_rect(&scissor_rect)
                }
            }
            None => {
                FramebufferIntRect::new(
                    FramebufferIntPoint::zero(),
                    device_size_as_framebuffer_size(dimensions),
                )
            }
        }
    }
}

/// Contains the parameters necessary to bind a texture-backed read target.
#[derive(Clone, Copy, Debug)]
pub enum ReadTarget {
    /// Use the device's default draw target.
    Default,
    /// Use the provided texture,
    Texture {
        /// ID of the FBO to read from.
        fbo_id: FBOId,
    },
    /// Use an FBO attached to an external texture.
    External {
        fbo: FBOId,
    },
    /// An FBO bound to a native (OS compositor) surface
    NativeSurface {
        fbo_id: FBOId,
        offset: DeviceIntPoint,
    },
}

impl ReadTarget {
    pub fn from_texture(
        texture: &Texture,
    ) -> Self {
        ReadTarget::Texture {
            fbo_id: texture.fbo.unwrap(),
        }
    }

    fn offset(&self) -> DeviceIntPoint {
        match *self {
            ReadTarget::Default |
            ReadTarget::Texture { .. } |
            ReadTarget::External { .. } => {
                DeviceIntPoint::zero()
            }

            ReadTarget::NativeSurface { offset, .. } => {
                offset
            }
        }
    }
}

impl From<DrawTarget> for ReadTarget {
    fn from(t: DrawTarget) -> Self {
        match t {
            DrawTarget::Default { .. } => {
                ReadTarget::Default
            }
            DrawTarget::NativeSurface { external_fbo_id, offset, .. } => {
                ReadTarget::NativeSurface {
                    fbo_id: FBOId(external_fbo_id),
                    offset,
                }
            }
            DrawTarget::Texture { fbo_id, .. } => {
                ReadTarget::Texture { fbo_id }
            }
            DrawTarget::External { fbo, .. } => {
                ReadTarget::External { fbo }
            }
        }
    }
}

impl Device {
    pub fn new(
        mut gl: Rc<dyn gl::Gl>,
        crash_annotator: Option<Box<dyn CrashAnnotator>>,
        resource_override_path: Option<PathBuf>,
        use_optimized_shaders: bool,
        upload_method: UploadMethod,
        cached_programs: Option<Rc<ProgramCache>>,
        allow_texture_storage_support: bool,
        allow_texture_swizzling: bool,
        dump_shader_source: Option<String>,
        surface_origin_is_top_left: bool,
        panic_on_gl_error: bool,
    ) -> Device {
        let mut max_texture_size = [0];
        unsafe {
            gl.get_integer_v(gl::MAX_TEXTURE_SIZE, &mut max_texture_size);
        }

        // We cap the max texture size at 16384. Some hardware report higher
        // capabilities but get very unstable with very large textures.
        // Bug 1702494 tracks re-evaluating this cap.
        let max_texture_size = max_texture_size[0].min(16384);

        let renderer_name = gl.get_string(gl::RENDERER);
        info!("Renderer: {}", renderer_name);
        info!("Max texture size: {}", max_texture_size);

        let mut extension_count = [0];
        unsafe {
            gl.get_integer_v(gl::NUM_EXTENSIONS, &mut extension_count);
        }
        let extension_count = extension_count[0] as gl::GLuint;
        let mut extensions = Vec::new();
        for i in 0 .. extension_count {
            extensions.push(gl.get_string_i(gl::EXTENSIONS, i));
        }

        // On debug builds, assert that each GL call is error-free. We don't do
        // this on release builds because the synchronous call can stall the
        // pipeline.
        let supports_khr_debug = supports_extension(&extensions, "GL_KHR_debug");
        if panic_on_gl_error || cfg!(debug_assertions) {
            gl = gl::ErrorReactingGl::wrap(gl, move |gl, name, code| {
                if supports_khr_debug {
                    Self::log_driver_messages(gl);
                }
                println!("Caught GL error {:x} at {}", code, name);
                panic!("Caught GL error {:x} at {}", code, name);
            });
        }

        if supports_extension(&extensions, "GL_ANGLE_provoking_vertex") {
            gl.provoking_vertex_angle(gl::FIRST_VERTEX_CONVENTION);
        }

        let supports_texture_usage = supports_extension(&extensions, "GL_ANGLE_texture_usage");

        // Our common-case image data in Firefox is BGRA, so we make an effort
        // to use BGRA as the internal texture storage format to avoid the need
        // to swizzle during upload. Currently we only do this on GLES (and thus
        // for Windows, via ANGLE).
        //
        // On Mac, Apple docs [1] claim that BGRA is a more efficient internal
        // format, but they don't support it with glTextureStorage. As a workaround,
        // we pretend that it's RGBA8 for the purposes of texture transfers,
        // but swizzle R with B for the texture sampling.
        //
        // We also need our internal format types to be sized, since glTexStorage*
        // will reject non-sized internal format types.
        //
        // Unfortunately, with GL_EXT_texture_format_BGRA8888, BGRA8 is not a
        // valid internal format (for glTexImage* or glTexStorage*) unless
        // GL_EXT_texture_storage is also available [2][3], which is usually
        // not the case on GLES 3 as the latter's functionality has been
        // included by default but the former has not been updated.
        // The extension is available on ANGLE, but on Android this usually
        // means we must fall back to using unsized BGRA and glTexImage*.
        //
        // Overall, we have the following factors in play when choosing the formats:
        //   - with glTexStorage, the internal format needs to match the external format,
        //     or the driver would have to do the conversion, which is slow
        //   - on desktop GL, there is no BGRA internal format. However, initializing
        //     the textures with glTexImage as RGBA appears to use BGRA internally,
        //     preferring BGRA external data [4].
        //   - when glTexStorage + BGRA internal format is not supported,
        //     and the external data is BGRA, we have the following options:
        //       1. use glTexImage with RGBA internal format, this costs us VRAM for mipmaps
        //       2. use glTexStorage with RGBA internal format, this costs us the conversion by the driver
        //       3. pretend we are uploading RGBA and set up the swizzling of the texture unit - this costs us batch breaks
        //
        // [1] https://developer.apple.com/library/archive/documentation/
        //     GraphicsImaging/Conceptual/OpenGL-MacProgGuide/opengl_texturedata/
        //     opengl_texturedata.html#//apple_ref/doc/uid/TP40001987-CH407-SW22
        // [2] https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_format_BGRA8888.txt
        // [3] https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_storage.txt
        // [4] http://http.download.nvidia.com/developer/Papers/2005/Fast_Texture_Transfers/Fast_Texture_Transfers.pdf

        // To support BGRA8 with glTexStorage* we specifically need
        // GL_EXT_texture_storage and GL_EXT_texture_format_BGRA8888.
        let supports_gles_bgra = supports_extension(&extensions, "GL_EXT_texture_format_BGRA8888");

        // On the android emulator glTexImage fails to create textures larger than 3379.
        // So we must use glTexStorage instead. See bug 1591436.
        let is_emulator = renderer_name.starts_with("Android Emulator");
        let avoid_tex_image = is_emulator;
        let mut gl_version = [0; 2];
        unsafe {
            gl.get_integer_v(gl::MAJOR_VERSION, &mut gl_version[0..1]);
            gl.get_integer_v(gl::MINOR_VERSION, &mut gl_version[1..2]);
        }
        info!("GL context {:?} {}.{}", gl.get_type(), gl_version[0], gl_version[1]);

        // We block texture storage on mac because it doesn't support BGRA
        let supports_texture_storage = allow_texture_storage_support && !cfg!(target_os = "macos") &&
            match gl.get_type() {
                gl::GlType::Gl => supports_extension(&extensions, "GL_ARB_texture_storage"),
                gl::GlType::Gles => true,
            };
        let supports_texture_swizzle = allow_texture_swizzling &&
            match gl.get_type() {
                // see https://www.g-truc.net/post-0734.html
                gl::GlType::Gl => gl_version >= [3, 3] ||
                    supports_extension(&extensions, "GL_ARB_texture_swizzle"),
                gl::GlType::Gles => true,
            };

        let (color_formats, bgra_formats, bgra_pixel_type, bgra8_sampling_swizzle, texture_storage_usage) = match gl.get_type() {
            // There is `glTexStorage`, use it and expect RGBA on the input.
            gl::GlType::Gl if supports_texture_storage && supports_texture_swizzle => (
                TextureFormatPair::from(ImageFormat::RGBA8),
                TextureFormatPair { internal: gl::RGBA8, external: gl::RGBA },
                gl::UNSIGNED_BYTE,
                Swizzle::Bgra, // pretend it's RGBA, rely on swizzling
                TexStorageUsage::Always
            ),
            // There is no `glTexStorage`, upload as `glTexImage` with BGRA input.
            gl::GlType::Gl => (
                TextureFormatPair { internal: ImageFormat::BGRA8, external: ImageFormat::BGRA8 },
                TextureFormatPair { internal: gl::RGBA, external: gl::BGRA },
                gl::UNSIGNED_INT_8_8_8_8_REV,
                Swizzle::Rgba, // converted on uploads by the driver, no swizzling needed
                TexStorageUsage::Never
            ),
            // glTexStorage is always supported in GLES 3, but because the GL_EXT_texture_storage
            // extension is supported we can use glTexStorage with BGRA8 as the internal format.
            // Prefer BGRA textures over RGBA.
            gl::GlType::Gles if supports_gles_bgra
                && supports_extension(&extensions, "GL_EXT_texture_storage") =>
            (
                TextureFormatPair::from(ImageFormat::BGRA8),
                TextureFormatPair { internal: gl::BGRA8_EXT, external: gl::BGRA_EXT },
                gl::UNSIGNED_BYTE,
                Swizzle::Rgba, // no conversion needed
                TexStorageUsage::Always,
            ),
            // BGRA is not supported as an internal format with glTexStorage, therefore we will
            // use RGBA textures instead and pretend BGRA data is RGBA when uploading.
            // The swizzling will happen at the texture unit.
            gl::GlType::Gles if supports_texture_swizzle => (
                TextureFormatPair::from(ImageFormat::RGBA8),
                TextureFormatPair { internal: gl::RGBA8, external: gl::RGBA },
                gl::UNSIGNED_BYTE,
                Swizzle::Bgra, // pretend it's RGBA, rely on swizzling
                TexStorageUsage::Always,
            ),
            // BGRA is not supported as an internal format with glTexStorage, and we cannot use
            // swizzling either. Therefore prefer BGRA textures over RGBA, but use glTexImage
            // to initialize BGRA textures. glTexStorage can still be used for other formats.
            gl::GlType::Gles if supports_gles_bgra && !avoid_tex_image => (
                TextureFormatPair::from(ImageFormat::BGRA8),
                TextureFormatPair::from(gl::BGRA_EXT),
                gl::UNSIGNED_BYTE,
                Swizzle::Rgba, // no conversion needed
                TexStorageUsage::NonBGRA8,
            ),
            // Neither BGRA or swizzling are supported. GLES does not allow format conversion
            // during upload so we must use RGBA textures and pretend BGRA data is RGBA when
            // uploading. Images may be rendered incorrectly as a result.
            gl::GlType::Gles => {
                warn!("Neither BGRA or texture swizzling are supported. Images may be rendered incorrectly.");
                (
                    TextureFormatPair::from(ImageFormat::RGBA8),
                    TextureFormatPair { internal: gl::RGBA8, external: gl::RGBA },
                    gl::UNSIGNED_BYTE,
                    Swizzle::Rgba,
                    TexStorageUsage::Always,
                )
            }
        };

        let is_software_webrender = renderer_name.starts_with("Software WebRender");
        let upload_method = if is_software_webrender {
            // Uploads in SWGL generally reduce to simple memory copies.
            UploadMethod::Immediate
        } else {
            upload_method
        };
        // Prefer 24-bit depth format. While 16-bit depth also works, it may exhaust depth ids easily.
        let depth_format = gl::DEPTH_COMPONENT24;

        info!("GL texture cache {:?}, bgra {:?} swizzle {:?}, texture storage {:?}, depth {:?}",
            color_formats, bgra_formats, bgra8_sampling_swizzle, texture_storage_usage, depth_format);

        // On Mali-T devices glCopyImageSubData appears to stall the pipeline until any pending
        // renders to the source texture have completed. On Mali-G, it has been observed to
        // indefinitely hang in some circumstances. Using an alternative such as glBlitFramebuffer
        // is preferable on such devices, so pretend we don't support glCopyImageSubData.
        // See bugs 1669494 and 1677757.
        let supports_copy_image_sub_data = if renderer_name.starts_with("Mali") {
            false
        } else {
            supports_extension(&extensions, "GL_EXT_copy_image") ||
            supports_extension(&extensions, "GL_ARB_copy_image")
        };

        // We have seen crashes on x86 PowerVR Rogue G6430 devices during GPU cache
        // updates using the scatter shader. It seems likely that GL_EXT_color_buffer_float
        // is broken. See bug 1709408.
        let is_x86_powervr_rogue_g6430 = renderer_name.starts_with("PowerVR Rogue G6430")
            && cfg!(target_arch = "x86");
        let supports_color_buffer_float = match gl.get_type() {
            gl::GlType::Gl => true,
            gl::GlType::Gles if is_x86_powervr_rogue_g6430 => false,
            gl::GlType::Gles => supports_extension(&extensions, "GL_EXT_color_buffer_float"),
        };

        let is_adreno = renderer_name.starts_with("Adreno");

        // There appears to be a driver bug on older versions of the Adreno
        // driver which prevents usage of persistenly mapped buffers.
        // See bugs 1678585 and 1683936.
        // TODO: only disable feature for affected driver versions.
        let supports_buffer_storage = if is_adreno {
            false
        } else {
            supports_extension(&extensions, "GL_EXT_buffer_storage") ||
            supports_extension(&extensions, "GL_ARB_buffer_storage")
        };

        // KHR_blend_equation_advanced renders incorrectly on Adreno
        // devices. This has only been confirmed up to Adreno 5xx, and has been
        // fixed for Android 9, so this condition could be made more specific.
        let supports_advanced_blend_equation =
            supports_extension(&extensions, "GL_KHR_blend_equation_advanced") &&
            !is_adreno;

        let supports_dual_source_blending = match gl.get_type() {
            gl::GlType::Gl => supports_extension(&extensions,"GL_ARB_blend_func_extended") &&
                supports_extension(&extensions,"GL_ARB_explicit_attrib_location"),
            gl::GlType::Gles => supports_extension(&extensions,"GL_EXT_blend_func_extended"),
        };

        // Software webrender relies on the unoptimized shader source.
        let use_optimized_shaders = use_optimized_shaders && !is_software_webrender;

        // On the android emulator, glShaderSource can crash if the source
        // strings are not null-terminated. See bug 1591945.
        let requires_null_terminated_shader_source = is_emulator;

        // The android emulator gets confused if you don't explicitly unbind any texture
        // from GL_TEXTURE_EXTERNAL_OES before binding another to GL_TEXTURE_2D. See bug 1636085.
        let requires_texture_external_unbind = is_emulator;

        let is_macos = cfg!(target_os = "macos");
             //  && renderer_name.starts_with("AMD");
             //  (XXX: we apply this restriction to all GPUs to handle switching)

        let is_angle = renderer_name.starts_with("ANGLE");
        let is_adreno_3xx = renderer_name.starts_with("Adreno (TM) 3");

        // Some GPUs require the stride of the data during texture uploads to be
        // aligned to certain requirements, either for correctness or performance
        // reasons.
        let required_pbo_stride = if is_adreno_3xx {
            // On Adreno 3xx, alignments of < 128 bytes can result in corrupted
            // glyphs. See bug 1696039.
            StrideAlignment::Bytes(NonZeroUsize::new(128).unwrap())
        } else if is_adreno {
            // On later Adreno devices it must be a multiple of 64 *pixels* to
            // hit the fast path, meaning value in bytes varies with the texture
            // format. This is purely an optimization.
            StrideAlignment::Pixels(NonZeroUsize::new(64).unwrap())
        } else if is_macos {
            // On AMD Mac, it must always be a multiple of 256 bytes.
            // We apply this restriction to all GPUs to handle switching
            StrideAlignment::Bytes(NonZeroUsize::new(256).unwrap())
        } else if is_angle {
            // On ANGLE, PBO texture uploads get incorrectly truncated if
            // the stride is greater than the width * bpp.
            StrideAlignment::Bytes(NonZeroUsize::new(1).unwrap())
        } else {
            // Other platforms may have similar requirements and should be added
            // here. The default value should be 4 bytes.
            StrideAlignment::Bytes(NonZeroUsize::new(4).unwrap())
        };

        // On AMD Macs there is a driver bug which causes some texture uploads
        // from a non-zero offset within a PBO to fail. See bug 1603783.
        let supports_nonzero_pbo_offsets = !is_macos;

        let is_mali = renderer_name.starts_with("Mali");

        // On Mali-Gxx and Txxx there is a driver bug when rendering partial updates to
        // offscreen render targets, so we must ensure we render to the entire target.
        // See bug 1663355.
        let supports_render_target_partial_update = !is_mali;

        let supports_shader_storage_object = match gl.get_type() {
            // see https://www.g-truc.net/post-0734.html
            gl::GlType::Gl => supports_extension(&extensions, "GL_ARB_shader_storage_buffer_object"),
            gl::GlType::Gles => gl_version >= [3, 1],
        };

        // SWGL uses swgl_clipMask() instead of implementing clip-masking in shaders.
        // This allows certain shaders to potentially bypass the more expensive alpha-
        // pass variants if they know the alpha-pass was only required to deal with
        // clip-masking.
        let uses_native_clip_mask = is_software_webrender;

        // SWGL uses swgl_antiAlias() instead of implementing anti-aliasing in shaders.
        // As above, this allows bypassing certain alpha-pass variants.
        let uses_native_antialiasing = is_software_webrender;

        let supports_image_external_essl3 = supports_extension(&extensions, "GL_OES_EGL_image_external_essl3");

        let is_mali_g = renderer_name.starts_with("Mali-G");

        let mut requires_batched_texture_uploads = None;
        if is_software_webrender {
            // No benefit to batching texture uploads with swgl.
            requires_batched_texture_uploads = Some(false);
        } else if is_mali_g {
            // On Mali-Gxx the driver really struggles with many small texture uploads,
            // and handles fewer, larger uploads better.
            requires_batched_texture_uploads = Some(true);
        }

        // On Mali-Txxx devices we have observed crashes during draw calls when rendering
        // to an alpha target immediately after using glClear to clear regions of it.
        // Using a shader to clear the regions avoids the crash. See bug 1638593.
        let is_mali_t = renderer_name.starts_with("Mali-T");
        let supports_alpha_target_clears = !is_mali_t;

        // On Linux we we have seen uploads to R8 format textures result in
        // corruption on some AMD cards.
        // See https://bugzilla.mozilla.org/show_bug.cgi?id=1687554#c13
        let supports_r8_texture_upload = if cfg!(target_os = "linux")
            && renderer_name.starts_with("AMD Radeon RX")
        {
            false
        } else {
            true
        };

        Device {
            gl,
            base_gl: None,
            crash_annotator,
            resource_override_path,
            use_optimized_shaders,
            upload_method,
            use_batched_texture_uploads: requires_batched_texture_uploads.unwrap_or(false),
            use_draw_calls_for_texture_copy: false,

            inside_frame: false,

            capabilities: Capabilities {
                supports_multisampling: false, //TODO
                supports_copy_image_sub_data,
                supports_color_buffer_float,
                supports_buffer_storage,
                supports_advanced_blend_equation,
                supports_dual_source_blending,
                supports_khr_debug,
                supports_texture_swizzle,
                supports_nonzero_pbo_offsets,
                supports_texture_usage,
                supports_render_target_partial_update,
                supports_shader_storage_object,
                requires_batched_texture_uploads,
                supports_alpha_target_clears,
                supports_r8_texture_upload,
                uses_native_clip_mask,
                uses_native_antialiasing,
                supports_image_external_essl3,
                renderer_name,
            },

            color_formats,
            bgra_formats,
            bgra_pixel_type,
            swizzle_settings: SwizzleSettings {
                bgra8_sampling_swizzle,
            },
            depth_format,

            depth_targets: FastHashMap::default(),

            bound_textures: [0; 16],
            bound_program: 0,
            bound_program_name: Rc::new(std::ffi::CString::new("").unwrap()),
            bound_vao: 0,
            bound_read_fbo: (FBOId(0), DeviceIntPoint::zero()),
            bound_draw_fbo: FBOId(0),
            program_mode_id: UniformLocation::INVALID,
            default_read_fbo: FBOId(0),
            default_draw_fbo: FBOId(0),

            depth_available: true,

            max_texture_size,
            cached_programs,
            frame_id: GpuFrameId(0),
            extensions,
            texture_storage_usage,
            requires_null_terminated_shader_source,
            requires_texture_external_unbind,
            required_pbo_stride,
            dump_shader_source,
            surface_origin_is_top_left,

            #[cfg(debug_assertions)]
            shader_is_ready: false,
        }
    }

    pub fn gl(&self) -> &dyn gl::Gl {
        &*self.gl
    }

    pub fn rc_gl(&self) -> &Rc<dyn gl::Gl> {
        &self.gl
    }

    /// Ensures that the maximum texture size is less than or equal to the
    /// provided value. If the provided value is less than the value supported
    /// by the driver, the latter is used.
    pub fn clamp_max_texture_size(&mut self, size: i32) {
        self.max_texture_size = self.max_texture_size.min(size);
    }

    /// Returns the limit on texture dimensions (width or height).
    pub fn max_texture_size(&self) -> i32 {
        self.max_texture_size
    }

    pub fn surface_origin_is_top_left(&self) -> bool {
        self.surface_origin_is_top_left
    }

    pub fn get_capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    pub fn preferred_color_formats(&self) -> TextureFormatPair<ImageFormat> {
        self.color_formats.clone()
    }

    pub fn swizzle_settings(&self) -> Option<SwizzleSettings> {
        if self.capabilities.supports_texture_swizzle {
            Some(self.swizzle_settings)
        } else {
            None
        }
    }

    pub fn depth_bits(&self) -> i32 {
        match self.depth_format {
            gl::DEPTH_COMPONENT16 => 16,
            gl::DEPTH_COMPONENT24 => 24,
            _ => panic!("Unknown depth format {:?}", self.depth_format),
        }
    }

    // See gpu_types.rs where we declare the number of possible documents and
    // number of items per document. This should match up with that.
    pub fn max_depth_ids(&self) -> i32 {
        return 1 << (self.depth_bits() - RESERVE_DEPTH_BITS);
    }

    pub fn ortho_near_plane(&self) -> f32 {
        return -self.max_depth_ids() as f32;
    }

    pub fn ortho_far_plane(&self) -> f32 {
        return (self.max_depth_ids() - 1) as f32;
    }

    pub fn required_pbo_stride(&self) -> StrideAlignment {
        self.required_pbo_stride
    }

    pub fn upload_method(&self) -> &UploadMethod {
        &self.upload_method
    }

    pub fn use_batched_texture_uploads(&self) -> bool {
        self.use_batched_texture_uploads
    }

    pub fn use_draw_calls_for_texture_copy(&self) -> bool {
        self.use_draw_calls_for_texture_copy
    }

    pub fn set_use_batched_texture_uploads(&mut self, enabled: bool) {
        if self.capabilities.requires_batched_texture_uploads.is_some() {
            return;
        }
        self.use_batched_texture_uploads = enabled;
    }

    pub fn set_use_draw_calls_for_texture_copy(&mut self, enabled: bool) {
        self.use_draw_calls_for_texture_copy = enabled;
    }

    pub fn reset_state(&mut self) {
        for i in 0 .. self.bound_textures.len() {
            self.bound_textures[i] = 0;
            self.gl.active_texture(gl::TEXTURE0 + i as gl::GLuint);
            self.gl.bind_texture(gl::TEXTURE_2D, 0);
        }

        self.bound_vao = 0;
        self.gl.bind_vertex_array(0);

        self.bound_read_fbo = (self.default_read_fbo, DeviceIntPoint::zero());
        self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, self.default_read_fbo.0);

        self.bound_draw_fbo = self.default_draw_fbo;
        self.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, self.bound_draw_fbo.0);
    }

    #[cfg(debug_assertions)]
    fn print_shader_errors(source: &str, log: &str) {
        // hacky way to extract the offending lines
        if !log.starts_with("0:") && !log.starts_with("0(") {
            return;
        }
        let end_pos = match log[2..].chars().position(|c| !c.is_digit(10)) {
            Some(pos) => 2 + pos,
            None => return,
        };
        let base_line_number = match log[2 .. end_pos].parse::<usize>() {
            Ok(number) if number >= 2 => number - 2,
            _ => return,
        };
        for (line, prefix) in source.lines().skip(base_line_number).zip(&["|",">","|"]) {
            error!("{}\t{}", prefix, line);
        }
    }

    pub fn compile_shader(
        &self,
        name: &str,
        shader_type: gl::GLenum,
        source: &String,
    ) -> Result<gl::GLuint, ShaderError> {
        debug!("compile {}", name);
        let id = self.gl.create_shader(shader_type);

        let mut new_source = Cow::from(source.as_str());
        // Ensure the source strings we pass to glShaderSource are
        // null-terminated on buggy platforms.
        if self.requires_null_terminated_shader_source {
            new_source.to_mut().push('\0');
        }

        self.gl.shader_source(id, &[new_source.as_bytes()]);
        self.gl.compile_shader(id);
        let log = self.gl.get_shader_info_log(id);
        let mut status = [0];
        unsafe {
            self.gl.get_shader_iv(id, gl::COMPILE_STATUS, &mut status);
        }
        if status[0] == 0 {
            let type_str = match shader_type {
                gl::VERTEX_SHADER => "vertex",
                gl::FRAGMENT_SHADER => "fragment",
                _ => panic!("Unexpected shader type {:x}", shader_type),
            };
            error!("Failed to compile {} shader: {}\n{}", type_str, name, log);
            #[cfg(debug_assertions)]
            Self::print_shader_errors(source, &log);
            Err(ShaderError::Compilation(name.to_string(), log))
        } else {
            if !log.is_empty() {
                warn!("Warnings detected on shader: {}\n{}", name, log);
            }
            Ok(id)
        }
    }

    pub fn begin_frame(&mut self) -> GpuFrameId {
        debug_assert!(!self.inside_frame);
        self.inside_frame = true;
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = false;
        }

        // If our profiler state has changed, apply or remove the profiling
        // wrapper from our GL context.
        let being_profiled = profiler::thread_is_being_profiled();
        let using_wrapper = self.base_gl.is_some();

        // We can usually unwind driver stacks on x86 so we don't need to manually instrument
        // gl calls there. Timestamps can be pretty expensive on Windows (2us each and perhaps
        // an opportunity to be descheduled?) which makes the profiles gathered with this
        // turned on less useful so only profile on ARM.
        if cfg!(any(target_arch = "arm", target_arch = "aarch64"))
            && being_profiled
            && !using_wrapper
        {
            fn note(name: &str, duration: Duration) {
                profiler::add_text_marker(cstr!("OpenGL Calls"), name, duration);
            }
            let threshold = Duration::from_millis(1);
            let wrapped = gl::ProfilingGl::wrap(self.gl.clone(), threshold, note);
            let base = mem::replace(&mut self.gl, wrapped);
            self.base_gl = Some(base);
        } else if !being_profiled && using_wrapper {
            self.gl = self.base_gl.take().unwrap();
        }

        // Retrieve the currently set FBO.
        let mut default_read_fbo = [0];
        unsafe {
            self.gl.get_integer_v(gl::READ_FRAMEBUFFER_BINDING, &mut default_read_fbo);
        }
        self.default_read_fbo = FBOId(default_read_fbo[0] as gl::GLuint);
        let mut default_draw_fbo = [0];
        unsafe {
            self.gl.get_integer_v(gl::DRAW_FRAMEBUFFER_BINDING, &mut default_draw_fbo);
        }
        self.default_draw_fbo = FBOId(default_draw_fbo[0] as gl::GLuint);

        // Shader state
        self.bound_program = 0;
        self.program_mode_id = UniformLocation::INVALID;
        self.gl.use_program(0);

        // Reset common state
        self.reset_state();

        // Pixel op state
        self.gl.pixel_store_i(gl::UNPACK_ALIGNMENT, 1);
        self.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);

        // Default is sampler 0, always
        self.gl.active_texture(gl::TEXTURE0);

        self.frame_id
    }

    fn bind_texture_impl(
        &mut self, slot: TextureSlot, id: gl::GLuint, target: gl::GLenum, set_swizzle: Option<Swizzle>
    ) {
        debug_assert!(self.inside_frame);

        if self.bound_textures[slot.0] != id || set_swizzle.is_some() {
            self.gl.active_texture(gl::TEXTURE0 + slot.0 as gl::GLuint);
            // The android emulator gets confused if you don't explicitly unbind any texture
            // from GL_TEXTURE_EXTERNAL_OES before binding to GL_TEXTURE_2D. See bug 1636085.
            if target == gl::TEXTURE_2D && self.requires_texture_external_unbind {
                self.gl.bind_texture(gl::TEXTURE_EXTERNAL_OES, 0);
            }
            self.gl.bind_texture(target, id);
            if let Some(swizzle) = set_swizzle {
                if self.capabilities.supports_texture_swizzle {
                    let components = match swizzle {
                        Swizzle::Rgba => [gl::RED, gl::GREEN, gl::BLUE, gl::ALPHA],
                        Swizzle::Bgra => [gl::BLUE, gl::GREEN, gl::RED, gl::ALPHA],
                    };
                    self.gl.tex_parameter_i(target, gl::TEXTURE_SWIZZLE_R, components[0] as i32);
                    self.gl.tex_parameter_i(target, gl::TEXTURE_SWIZZLE_G, components[1] as i32);
                    self.gl.tex_parameter_i(target, gl::TEXTURE_SWIZZLE_B, components[2] as i32);
                    self.gl.tex_parameter_i(target, gl::TEXTURE_SWIZZLE_A, components[3] as i32);
                } else {
                    debug_assert_eq!(swizzle, Swizzle::default());
                }
            }
            self.gl.active_texture(gl::TEXTURE0);
            self.bound_textures[slot.0] = id;
        }
    }

    pub fn bind_texture<S>(&mut self, slot: S, texture: &Texture, swizzle: Swizzle)
    where
        S: Into<TextureSlot>,
    {
        let old_swizzle = texture.active_swizzle.replace(swizzle);
        let set_swizzle = if old_swizzle != swizzle {
            Some(swizzle)
        } else {
            None
        };
        self.bind_texture_impl(slot.into(), texture.id, texture.target, set_swizzle);
    }

    pub fn bind_external_texture<S>(&mut self, slot: S, external_texture: &ExternalTexture)
    where
        S: Into<TextureSlot>,
    {
        self.bind_texture_impl(slot.into(), external_texture.id, external_texture.target, None);
    }

    pub fn bind_read_target_impl(
        &mut self,
        fbo_id: FBOId,
        offset: DeviceIntPoint,
    ) {
        debug_assert!(self.inside_frame);

        if self.bound_read_fbo != (fbo_id, offset) {
            fbo_id.bind(self.gl(), FBOTarget::Read);
        }

        self.bound_read_fbo = (fbo_id, offset);
    }

    pub fn bind_read_target(&mut self, target: ReadTarget) {
        let fbo_id = match target {
            ReadTarget::Default => self.default_read_fbo,
            ReadTarget::Texture { fbo_id } => fbo_id,
            ReadTarget::External { fbo } => fbo,
            ReadTarget::NativeSurface { fbo_id, .. } => fbo_id,
        };

        self.bind_read_target_impl(fbo_id, target.offset())
    }

    fn bind_draw_target_impl(&mut self, fbo_id: FBOId) {
        debug_assert!(self.inside_frame);

        if self.bound_draw_fbo != fbo_id {
            self.bound_draw_fbo = fbo_id;
            fbo_id.bind(self.gl(), FBOTarget::Draw);
        }
    }

    pub fn reset_read_target(&mut self) {
        let fbo = self.default_read_fbo;
        self.bind_read_target_impl(fbo, DeviceIntPoint::zero());
    }


    pub fn reset_draw_target(&mut self) {
        let fbo = self.default_draw_fbo;
        self.bind_draw_target_impl(fbo);
        self.depth_available = true;
    }

    pub fn bind_draw_target(
        &mut self,
        target: DrawTarget,
    ) {
        let (fbo_id, rect, depth_available) = match target {
            DrawTarget::Default { rect, .. } => {
                (self.default_draw_fbo, rect, false)
            }
            DrawTarget::Texture { dimensions, fbo_id, with_depth, .. } => {
                let rect = FramebufferIntRect::new(
                    FramebufferIntPoint::zero(),
                    device_size_as_framebuffer_size(dimensions),
                );
                (fbo_id, rect, with_depth)
            },
            DrawTarget::External { fbo, size } => {
                (fbo, size.into(), false)
            }
            DrawTarget::NativeSurface { external_fbo_id, offset, dimensions, .. } => {
                (
                    FBOId(external_fbo_id),
                    device_rect_as_framebuffer_rect(&DeviceIntRect::new(offset, dimensions)),
                    true
                )
            }
        };

        self.depth_available = depth_available;
        self.bind_draw_target_impl(fbo_id);
        self.gl.viewport(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );
    }

    /// Creates an unbound FBO object. Additional attachment API calls are
    /// required to make it complete.
    pub fn create_fbo(&mut self) -> FBOId {
        FBOId(self.gl.gen_framebuffers(1)[0])
    }

    /// Creates an FBO with the given texture bound as the color attachment.
    pub fn create_fbo_for_external_texture(&mut self, texture_id: u32) -> FBOId {
        let fbo = self.create_fbo();
        fbo.bind(self.gl(), FBOTarget::Draw);
        self.gl.framebuffer_texture_2d(
            gl::DRAW_FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture_id,
            0,
        );
        debug_assert_eq!(
            self.gl.check_frame_buffer_status(gl::DRAW_FRAMEBUFFER),
            gl::FRAMEBUFFER_COMPLETE,
            "Incomplete framebuffer",
        );
        self.bound_draw_fbo.bind(self.gl(), FBOTarget::Draw);
        fbo
    }

    pub fn delete_fbo(&mut self, fbo: FBOId) {
        self.gl.delete_framebuffers(&[fbo.0]);
    }

    pub fn bind_external_draw_target(&mut self, fbo_id: FBOId) {
        debug_assert!(self.inside_frame);

        if self.bound_draw_fbo != fbo_id {
            self.bound_draw_fbo = fbo_id;
            fbo_id.bind(self.gl(), FBOTarget::Draw);
        }
    }

    /// Link a program, attaching the supplied vertex format.
    ///
    /// If `create_program()` finds a binary shader on disk, it will kick
    /// off linking immediately, which some drivers (notably ANGLE) run
    /// in parallel on background threads. As such, this function should
    /// ideally be run sometime later, to give the driver time to do that
    /// before blocking due to an API call accessing the shader.
    ///
    /// This generally means that the first run of the application will have
    /// to do a bunch of blocking work to compile the shader from source, but
    /// subsequent runs should load quickly.
    pub fn link_program(
        &mut self,
        program: &mut Program,
        descriptor: &VertexDescriptor,
    ) -> Result<(), ShaderError> {
        let _guard = CrashAnnotatorGuard::new(
            &self.crash_annotator,
            CrashAnnotation::CompileShader,
            &program.source_info.full_name_cstr
        );

        assert!(!program.is_initialized());
        let mut build_program = true;
        let info = &program.source_info;

        // See if we hit the binary shader cache
        if let Some(ref cached_programs) = self.cached_programs {
            // If the shader is not in the cache, attempt to load it from disk
            if cached_programs.entries.borrow().get(&program.source_info.digest).is_none() {
                if let Some(ref handler) = cached_programs.program_cache_handler {
                    handler.try_load_shader_from_disk(&program.source_info.digest, cached_programs);
                    if let Some(entry) = cached_programs.entries.borrow().get(&program.source_info.digest) {
                        self.gl.program_binary(program.id, entry.binary.format, &entry.binary.bytes);
                    }
                }
            }

            if let Some(entry) = cached_programs.entries.borrow_mut().get_mut(&info.digest) {
                let mut link_status = [0];
                unsafe {
                    self.gl.get_program_iv(program.id, gl::LINK_STATUS, &mut link_status);
                }
                if link_status[0] == 0 {
                    let error_log = self.gl.get_program_info_log(program.id);
                    error!(
                      "Failed to load a program object with a program binary: {} renderer {}\n{}",
                      &info.base_filename,
                      self.capabilities.renderer_name,
                      error_log
                    );
                    if let Some(ref program_cache_handler) = cached_programs.program_cache_handler {
                        program_cache_handler.notify_program_binary_failed(&entry.binary);
                    }
                } else {
                    entry.linked = true;
                    build_program = false;
                }
            }
        }

        // If not, we need to do a normal compile + link pass.
        if build_program {
            // Compile the vertex shader
            let vs_source = info.compute_source(self, ShaderKind::Vertex);
            let vs_id = match self.compile_shader(&info.full_name(), gl::VERTEX_SHADER, &vs_source) {
                    Ok(vs_id) => vs_id,
                    Err(err) => return Err(err),
                };

            // Compile the fragment shader
            let fs_source = info.compute_source(self, ShaderKind::Fragment);
            let fs_id =
                match self.compile_shader(&info.full_name(), gl::FRAGMENT_SHADER, &fs_source) {
                    Ok(fs_id) => fs_id,
                    Err(err) => {
                        self.gl.delete_shader(vs_id);
                        return Err(err);
                    }
                };

            // Check if shader source should be dumped
            if Some(info.base_filename) == self.dump_shader_source.as_ref().map(String::as_ref) {
                let path = std::path::Path::new(info.base_filename);
                std::fs::write(path.with_extension("vert"), vs_source).unwrap();
                std::fs::write(path.with_extension("frag"), fs_source).unwrap();
            }

            // Attach shaders
            self.gl.attach_shader(program.id, vs_id);
            self.gl.attach_shader(program.id, fs_id);

            // Bind vertex attributes
            for (i, attr) in descriptor
                .vertex_attributes
                .iter()
                .chain(descriptor.instance_attributes.iter())
                .enumerate()
            {
                self.gl
                    .bind_attrib_location(program.id, i as gl::GLuint, attr.name);
            }

            if self.cached_programs.is_some() {
                self.gl.program_parameter_i(program.id, gl::PROGRAM_BINARY_RETRIEVABLE_HINT, gl::TRUE as gl::GLint);
            }

            // Link!
            self.gl.link_program(program.id);

            if cfg!(debug_assertions) {
                // Check that all our overrides worked
                for (i, attr) in descriptor
                    .vertex_attributes
                    .iter()
                    .chain(descriptor.instance_attributes.iter())
                    .enumerate()
                {
                    //Note: we can't assert here because the driver may optimize out some of the
                    // vertex attributes legitimately, returning their location to be -1.
                    let location = self.gl.get_attrib_location(program.id, attr.name);
                    if location != i as gl::GLint {
                        warn!("Attribute {:?} is not found in the shader {}. Expected at {}, found at {}",
                            attr, program.source_info.base_filename, i, location);
                    }
                }
            }

            // GL recommends detaching and deleting shaders once the link
            // is complete (whether successful or not). This allows the driver
            // to free any memory associated with the parsing and compilation.
            self.gl.detach_shader(program.id, vs_id);
            self.gl.detach_shader(program.id, fs_id);
            self.gl.delete_shader(vs_id);
            self.gl.delete_shader(fs_id);

            let mut link_status = [0];
            unsafe {
                self.gl.get_program_iv(program.id, gl::LINK_STATUS, &mut link_status);
            }
            if link_status[0] == 0 {
                let error_log = self.gl.get_program_info_log(program.id);
                error!(
                    "Failed to link shader program: {}\n{}",
                    &info.base_filename,
                    error_log
                );
                self.gl.delete_program(program.id);
                return Err(ShaderError::Link(info.base_filename.to_owned(), error_log));
            }

            if let Some(ref cached_programs) = self.cached_programs {
                if !cached_programs.entries.borrow().contains_key(&info.digest) {
                    let (buffer, format) = self.gl.get_program_binary(program.id);
                    if buffer.len() > 0 {
                        let binary = Arc::new(ProgramBinary::new(buffer, format, info.digest.clone()));
                        cached_programs.add_new_program_binary(binary);
                    }
                }
            }
        }

        // If we get here, the link succeeded, so get the uniforms.
        program.is_initialized = true;
        program.u_transform = self.gl.get_uniform_location(program.id, "uTransform");
        program.u_mode = self.gl.get_uniform_location(program.id, "uMode");
        program.u_texture_size = self.gl.get_uniform_location(program.id, "uTextureSize");

        Ok(())
    }

    pub fn bind_program(&mut self, program: &Program) -> bool {
        debug_assert!(self.inside_frame);
        debug_assert!(program.is_initialized());
        if !program.is_initialized() {
            return false;
        }
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = true;
        }

        if self.bound_program != program.id {
            self.gl.use_program(program.id);
            self.bound_program = program.id;
            self.bound_program_name = program.source_info.full_name_cstr.clone();
            self.program_mode_id = UniformLocation(program.u_mode);
        }
        true
    }

    pub fn create_texture(
        &mut self,
        target: ImageBufferKind,
        format: ImageFormat,
        mut width: i32,
        mut height: i32,
        filter: TextureFilter,
        render_target: Option<RenderTargetInfo>,
    ) -> Texture {
        debug_assert!(self.inside_frame);

        if width > self.max_texture_size || height > self.max_texture_size {
            error!("Attempting to allocate a texture of size {}x{} above the limit, trimming", width, height);
            width = width.min(self.max_texture_size);
            height = height.min(self.max_texture_size);
        }

        // Set up the texture book-keeping.
        let mut texture = Texture {
            id: self.gl.gen_textures(1)[0],
            target: get_gl_target(target),
            size: DeviceIntSize::new(width, height),
            format,
            filter,
            active_swizzle: Cell::default(),
            fbo: None,
            fbo_with_depth: None,
            last_frame_used: self.frame_id,
            flags: TextureFlags::default(),
        };
        self.bind_texture(DEFAULT_TEXTURE, &texture, Swizzle::default());
        self.set_texture_parameters(texture.target, filter);

        if self.capabilities.supports_texture_usage && render_target.is_some() {
            self.gl.tex_parameter_i(texture.target, gl::TEXTURE_USAGE_ANGLE, gl::FRAMEBUFFER_ATTACHMENT_ANGLE as gl::GLint);
        }

        // Allocate storage.
        let desc = self.gl_describe_format(texture.format);

        // Firefox doesn't use mipmaps, but Servo uses them for standalone image
        // textures images larger than 512 pixels. This is the only case where
        // we set the filter to trilinear.
        let mipmap_levels =  if texture.filter == TextureFilter::Trilinear {
            let max_dimension = cmp::max(width, height);
            ((max_dimension) as f64).log2() as gl::GLint + 1
        } else {
            1
        };

        // We never want to upload texture data at the same time as allocating the texture.
        self.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);

        // Use glTexStorage where available, since it avoids allocating
        // unnecessary mipmap storage and generally improves performance with
        // stronger invariants.
        let use_texture_storage = match self.texture_storage_usage {
            TexStorageUsage::Always => true,
            TexStorageUsage::NonBGRA8 => texture.format != ImageFormat::BGRA8,
            TexStorageUsage::Never => false,
        };
        if use_texture_storage {
            self.gl.tex_storage_2d(
                texture.target,
                mipmap_levels,
                desc.internal,
                texture.size.width as gl::GLint,
                texture.size.height as gl::GLint,
            );
        } else {
            self.gl.tex_image_2d(
                texture.target,
                0,
                desc.internal as gl::GLint,
                texture.size.width as gl::GLint,
                texture.size.height as gl::GLint,
                0,
                desc.external,
                desc.pixel_type,
                None,
            );            
        }

        // Set up FBOs, if required.
        if let Some(rt_info) = render_target {
            self.init_fbos(&mut texture, false);
            if rt_info.has_depth {
                self.init_fbos(&mut texture, true);
            }
        }

        texture
    }

    fn set_texture_parameters(&mut self, target: gl::GLuint, filter: TextureFilter) {
        let mag_filter = match filter {
            TextureFilter::Nearest => gl::NEAREST,
            TextureFilter::Linear | TextureFilter::Trilinear => gl::LINEAR,
        };

        let min_filter = match filter {
            TextureFilter::Nearest => gl::NEAREST,
            TextureFilter::Linear => gl::LINEAR,
            TextureFilter::Trilinear => gl::LINEAR_MIPMAP_LINEAR,
        };

        self.gl
            .tex_parameter_i(target, gl::TEXTURE_MAG_FILTER, mag_filter as gl::GLint);
        self.gl
            .tex_parameter_i(target, gl::TEXTURE_MIN_FILTER, min_filter as gl::GLint);

        self.gl
            .tex_parameter_i(target, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as gl::GLint);
        self.gl
            .tex_parameter_i(target, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as gl::GLint);
    }

    /// Copies the entire contents of one texture to another. The dest texture must be at least
    /// as large as the source texture in each dimension. No scaling is performed, so if the dest
    /// texture is larger than the source texture then some of its pixels will not be written to.
    pub fn copy_entire_texture(
        &mut self,
        dst: &mut Texture,
        src: &Texture,
    ) {
        debug_assert!(self.inside_frame);
        debug_assert!(dst.size.width >= src.size.width);
        debug_assert!(dst.size.height >= src.size.height);

        self.copy_texture_sub_region(
            src,
            0,
            0,
            dst,
            0,
            0,
            src.size.width as _,
            src.size.height as _,
        );
    }

    /// Copies the specified subregion from src_texture to dest_texture.
    pub fn copy_texture_sub_region(
        &mut self,
        src_texture: &Texture,
        src_x: usize,
        src_y: usize,
        dest_texture: &Texture,
        dest_x: usize,
        dest_y: usize,
        width: usize,
        height: usize,
    ) {
        if self.capabilities.supports_copy_image_sub_data {
            assert_ne!(
                src_texture.id, dest_texture.id,
                "glCopyImageSubData's behaviour is undefined if src and dst images are identical and the rectangles overlap."
            );
            unsafe {
                self.gl.copy_image_sub_data(
                    src_texture.id,
                    src_texture.target,
                    0,
                    src_x as _,
                    src_y as _,
                    0,
                    dest_texture.id,
                    dest_texture.target,
                    0,
                    dest_x as _,
                    dest_y as _,
                    0,
                    width as _,
                    height as _,
                    1,
                );
            }
        } else {
            let src_offset = FramebufferIntPoint::new(src_x as i32, src_y as i32);
            let dest_offset = FramebufferIntPoint::new(dest_x as i32, dest_y as i32);
            let size = FramebufferIntSize::new(width as i32, height as i32);

            self.blit_render_target(
                ReadTarget::from_texture(src_texture),
                FramebufferIntRect::new(src_offset, size),
                DrawTarget::from_texture(dest_texture, false),
                FramebufferIntRect::new(dest_offset, size),
                // In most cases the filter shouldn't matter, as there is no scaling involved
                // in the blit. We were previously using Linear, but this caused issues when
                // blitting RGBAF32 textures on Mali, so use Nearest to be safe.
                TextureFilter::Nearest,
            );
        }
    }

    /// Notifies the device that the contents of a render target are no longer
    /// needed.
    pub fn invalidate_render_target(&mut self, texture: &Texture) {
        let (fbo, attachments) = if texture.supports_depth() {
            (&texture.fbo_with_depth,
             &[gl::COLOR_ATTACHMENT0, gl::DEPTH_ATTACHMENT] as &[gl::GLenum])
        } else {
            (&texture.fbo, &[gl::COLOR_ATTACHMENT0] as &[gl::GLenum])
        };

        if let Some(fbo_id) = fbo {
            let original_bound_fbo = self.bound_draw_fbo;
            // Note: The invalidate extension may not be supported, in which
            // case this is a no-op. That's ok though, because it's just a
            // hint.
            self.bind_external_draw_target(*fbo_id);
            self.gl.invalidate_framebuffer(gl::FRAMEBUFFER, attachments);
            self.bind_external_draw_target(original_bound_fbo);
        }
    }

    /// Notifies the device that the contents of the current framebuffer's depth
    /// attachment is no longer needed. Unlike invalidate_render_target, this can
    /// be called even when the contents of the colour attachment is still required.
    /// This should be called before unbinding the framebuffer at the end of a pass,
    /// to allow tiled GPUs to avoid writing the contents back to memory.
    pub fn invalidate_depth_target(&mut self) {
        assert!(self.depth_available);
        let attachments = if self.bound_draw_fbo == self.default_draw_fbo {
            &[gl::DEPTH] as &[gl::GLenum]
        } else {
            &[gl::DEPTH_ATTACHMENT] as &[gl::GLenum]
        };
        self.gl.invalidate_framebuffer(gl::DRAW_FRAMEBUFFER, attachments);
    }

    /// Notifies the device that a render target is about to be reused.
    ///
    /// This method adds or removes a depth target as necessary.
    pub fn reuse_render_target<T: Texel>(
        &mut self,
        texture: &mut Texture,
        rt_info: RenderTargetInfo,
    ) {
        texture.last_frame_used = self.frame_id;

        // Add depth support if needed.
        if rt_info.has_depth && !texture.supports_depth() {
            self.init_fbos(texture, true);
        }
    }

    fn init_fbos(&mut self, texture: &mut Texture, with_depth: bool) {
        let (fbo, depth_rb) = if with_depth {
            let depth_target = self.acquire_depth_target(texture.get_dimensions());
            (&mut texture.fbo_with_depth, Some(depth_target))
        } else {
            (&mut texture.fbo, None)
        };

        // Generate the FBOs.
        assert!(fbo.is_none());
        let fbo_id = FBOId(*self.gl.gen_framebuffers(1).first().unwrap());
        *fbo = Some(fbo_id);

        // Bind the FBOs.
        let original_bound_fbo = self.bound_draw_fbo;

        self.bind_external_draw_target(fbo_id);

        self.gl.framebuffer_texture_2d(
            gl::DRAW_FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            texture.target,
            texture.id,
            0,
        );

        if let Some(depth_rb) = depth_rb {
            self.gl.framebuffer_renderbuffer(
                gl::DRAW_FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                depth_rb.0,
            );
        }

        debug_assert_eq!(
            self.gl.check_frame_buffer_status(gl::DRAW_FRAMEBUFFER),
            gl::FRAMEBUFFER_COMPLETE,
            "Incomplete framebuffer",
        );

        self.bind_external_draw_target(original_bound_fbo);
    }

    fn acquire_depth_target(&mut self, dimensions: DeviceIntSize) -> RBOId {
        let gl = &self.gl;
        let depth_format = self.depth_format;
        let target = self.depth_targets.entry(dimensions).or_insert_with(|| {
            let renderbuffer_ids = gl.gen_renderbuffers(1);
            let depth_rb = renderbuffer_ids[0];
            gl.bind_renderbuffer(gl::RENDERBUFFER, depth_rb);
            gl.renderbuffer_storage(
                gl::RENDERBUFFER,
                depth_format,
                dimensions.width as _,
                dimensions.height as _,
            );
            SharedDepthTarget {
                rbo_id: RBOId(depth_rb),
                refcount: 0,
            }
        });
        target.refcount += 1;
        target.rbo_id
    }

    fn release_depth_target(&mut self, dimensions: DeviceIntSize) {
        let mut entry = match self.depth_targets.entry(dimensions) {
            Entry::Occupied(x) => x,
            Entry::Vacant(..) => panic!("Releasing unknown depth target"),
        };
        debug_assert!(entry.get().refcount != 0);
        entry.get_mut().refcount -= 1;
        if entry.get().refcount == 0 {
            let (_, target) = entry.remove_entry();
            self.gl.delete_renderbuffers(&[target.rbo_id.0]);
        }
    }

    /// Perform a blit between self.bound_read_fbo and self.bound_draw_fbo.
    fn blit_render_target_impl(
        &mut self,
        src_rect: FramebufferIntRect,
        dest_rect: FramebufferIntRect,
        filter: TextureFilter,
    ) {
        debug_assert!(self.inside_frame);

        let filter = match filter {
            TextureFilter::Nearest => gl::NEAREST,
            TextureFilter::Linear | TextureFilter::Trilinear => gl::LINEAR,
        };

        let src_x0 = src_rect.origin.x + self.bound_read_fbo.1.x;
        let src_y0 = src_rect.origin.y + self.bound_read_fbo.1.y;

        self.gl.blit_framebuffer(
            src_x0,
            src_y0,
            src_x0 + src_rect.size.width,
            src_y0 + src_rect.size.height,
            dest_rect.origin.x,
            dest_rect.origin.y,
            dest_rect.origin.x + dest_rect.size.width,
            dest_rect.origin.y + dest_rect.size.height,
            gl::COLOR_BUFFER_BIT,
            filter,
        );
    }

    /// Perform a blit between src_target and dest_target.
    /// This will overwrite self.bound_read_fbo and self.bound_draw_fbo.
    pub fn blit_render_target(
        &mut self,
        src_target: ReadTarget,
        src_rect: FramebufferIntRect,
        dest_target: DrawTarget,
        dest_rect: FramebufferIntRect,
        filter: TextureFilter,
    ) {
        debug_assert!(self.inside_frame);

        self.bind_read_target(src_target);

        self.bind_draw_target(dest_target);

        self.blit_render_target_impl(src_rect, dest_rect, filter);
    }

    /// Performs a blit while flipping vertically. Useful for blitting textures
    /// (which use origin-bottom-left) to the main framebuffer (which uses
    /// origin-top-left).
    pub fn blit_render_target_invert_y(
        &mut self,
        src_target: ReadTarget,
        src_rect: FramebufferIntRect,
        dest_target: DrawTarget,
        dest_rect: FramebufferIntRect,
    ) {
        debug_assert!(self.inside_frame);

        let mut inverted_dest_rect = dest_rect;
        inverted_dest_rect.origin.y = dest_rect.max_y();
        inverted_dest_rect.size.height *= -1;

        self.blit_render_target(
            src_target,
            src_rect,
            dest_target,
            inverted_dest_rect,
            TextureFilter::Linear,
        );
    }

    pub fn delete_texture(&mut self, mut texture: Texture) {
        debug_assert!(self.inside_frame);
        let had_depth = texture.supports_depth();
        if let Some(fbo) = texture.fbo {
            self.gl.delete_framebuffers(&[fbo.0]);
            texture.fbo = None;
        }
        if let Some(fbo) = texture.fbo_with_depth {
            self.gl.delete_framebuffers(&[fbo.0]);
            texture.fbo_with_depth = None;
        }

        if had_depth {
            self.release_depth_target(texture.get_dimensions());
        }

        self.gl.delete_textures(&[texture.id]);

        for bound_texture in &mut self.bound_textures {
            if *bound_texture == texture.id {
                *bound_texture = 0;
            }
        }

        // Disarm the assert in Texture::drop().
        texture.id = 0;
    }

    #[cfg(feature = "replay")]
    pub fn delete_external_texture(&mut self, mut external: ExternalTexture) {
        self.gl.delete_textures(&[external.id]);
        external.id = 0;
    }

    pub fn delete_program(&mut self, mut program: Program) {
        self.gl.delete_program(program.id);
        program.id = 0;
    }

    /// Create a shader program and link it immediately.
    pub fn create_program_linked(
        &mut self,
        base_filename: &'static str,
        features: &[&'static str],
        descriptor: &VertexDescriptor,
    ) -> Result<Program, ShaderError> {
        let mut program = self.create_program(base_filename, features)?;
        self.link_program(&mut program, descriptor)?;
        Ok(program)
    }

    /// Create a shader program. This does minimal amount of work to start
    /// loading a binary shader. If a binary shader is found, we invoke
    /// glProgramBinary, which, at least on ANGLE, will load and link the
    /// binary on a background thread. This can speed things up later when
    /// we invoke `link_program()`.
    pub fn create_program(
        &mut self,
        base_filename: &'static str,
        features: &[&'static str],
    ) -> Result<Program, ShaderError> {
        debug_assert!(self.inside_frame);

        let source_info = ProgramSourceInfo::new(self, base_filename, features);

        // Create program
        let pid = self.gl.create_program();

        // Attempt to load a cached binary if possible.
        if let Some(ref cached_programs) = self.cached_programs {
            if let Some(entry) = cached_programs.entries.borrow().get(&source_info.digest) {
                self.gl.program_binary(pid, entry.binary.format, &entry.binary.bytes);
            }
        }

        // Use 0 for the uniforms as they are initialized by link_program.
        let program = Program {
            id: pid,
            u_transform: 0,
            u_mode: 0,
            u_texture_size: 0,
            source_info,
            is_initialized: false,
        };

        Ok(program)
    }

    fn build_shader_string<F: FnMut(&str)>(
        &self,
        features: &[&'static str],
        kind: ShaderKind,
        base_filename: &str,
        output: F,
    ) {
        do_build_shader_string(
            get_shader_version(&*self.gl),
            features,
            kind,
            base_filename,
            &|f| get_unoptimized_shader_source(f, self.resource_override_path.as_ref()),
            output,
        )
    }

    pub fn bind_shader_samplers<S>(&mut self, program: &Program, bindings: &[(&'static str, S)])
    where
        S: Into<TextureSlot> + Copy,
    {
        // bind_program() must be called before calling bind_shader_samplers
        assert_eq!(self.bound_program, program.id);

        for binding in bindings {
            let u_location = self.gl.get_uniform_location(program.id, binding.0);
            if u_location != -1 {
                self.bind_program(program);
                self.gl
                    .uniform_1i(u_location, binding.1.into().0 as gl::GLint);
            }
        }
    }

    pub fn get_uniform_location(&self, program: &Program, name: &str) -> UniformLocation {
        UniformLocation(self.gl.get_uniform_location(program.id, name))
    }

    pub fn set_uniforms(
        &self,
        program: &Program,
        transform: &Transform3D<f32>,
    ) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        self.gl
            .uniform_matrix_4fv(program.u_transform, false, &transform.to_array());
    }

    pub fn switch_mode(&self, mode: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        self.gl.uniform_1i(self.program_mode_id.0, mode);
    }

    /// Sets the uTextureSize uniform. Most shaders do not require this to be called
    /// as they use the textureSize GLSL function instead.
    pub fn set_shader_texture_size(
        &self,
        program: &Program,
        texture_size: DeviceSize,
    ) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        if program.u_texture_size != -1 {
            self.gl.uniform_2f(program.u_texture_size, texture_size.width, texture_size.height);
        }
    }

    pub fn create_pbo(&mut self) -> PBO {
        let id = self.gl.gen_buffers(1)[0];
        PBO {
            id,
            reserved_size: 0,
        }
    }

    pub fn create_pbo_with_size(&mut self, size: usize) -> PBO {
        let mut pbo = self.create_pbo();

        self.gl.bind_buffer(gl::PIXEL_PACK_BUFFER, pbo.id);
        self.gl.pixel_store_i(gl::PACK_ALIGNMENT, 1);
        self.gl.buffer_data_untyped(
            gl::PIXEL_PACK_BUFFER,
            size as _,
            ptr::null(),
            gl::STREAM_READ,
        );
        self.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);

        pbo.reserved_size = size;
        pbo
    }

    pub fn read_pixels_into_pbo(
        &mut self,
        read_target: ReadTarget,
        rect: DeviceIntRect,
        format: ImageFormat,
        pbo: &PBO,
    ) {
        let byte_size = rect.size.area() as usize * format.bytes_per_pixel() as usize;

        assert!(byte_size <= pbo.reserved_size);

        self.bind_read_target(read_target);

        self.gl.bind_buffer(gl::PIXEL_PACK_BUFFER, pbo.id);
        self.gl.pixel_store_i(gl::PACK_ALIGNMENT, 1);

        let gl_format = self.gl_describe_format(format);

        unsafe {
            self.gl.read_pixels_into_pbo(
                rect.origin.x as _,
                rect.origin.y as _,
                rect.size.width as _,
                rect.size.height as _,
                gl_format.read,
                gl_format.pixel_type,
            );
        }

        self.gl.bind_buffer(gl::PIXEL_PACK_BUFFER, 0);
    }

    pub fn map_pbo_for_readback<'a>(&'a mut self, pbo: &'a PBO) -> Option<BoundPBO<'a>> {
        self.gl.bind_buffer(gl::PIXEL_PACK_BUFFER, pbo.id);

        let buf_ptr = match self.gl.get_type() {
            gl::GlType::Gl => {
                self.gl.map_buffer(gl::PIXEL_PACK_BUFFER, gl::READ_ONLY)
            }

            gl::GlType::Gles => {
                self.gl.map_buffer_range(
                    gl::PIXEL_PACK_BUFFER,
                    0,
                    pbo.reserved_size as _,
                    gl::MAP_READ_BIT)
            }
        };

        if buf_ptr.is_null() {
            return None;
        }

        let buffer = unsafe { slice::from_raw_parts(buf_ptr as *const u8, pbo.reserved_size) };

        Some(BoundPBO {
            device: self,
            data: buffer,
        })
    }

    pub fn delete_pbo(&mut self, mut pbo: PBO) {
        self.gl.delete_buffers(&[pbo.id]);
        pbo.id = 0;
        pbo.reserved_size = 0
    }

    /// Returns the size and stride in bytes required to upload an area of pixels
    /// of the specified size, to a texture of the specified format.
    pub fn required_upload_size_and_stride(&self, size: DeviceIntSize, format: ImageFormat) -> (usize, usize) {
        assert!(size.width >= 0);
        assert!(size.height >= 0);

        let bytes_pp = format.bytes_per_pixel() as usize;
        let width_bytes = size.width as usize * bytes_pp;

        let dst_stride = round_up_to_multiple(width_bytes, self.required_pbo_stride.num_bytes(format));

        // The size of the chunk should only need to be (height - 1) * dst_stride + width_bytes,
        // however, the android emulator will error unless it is height * dst_stride.
        // See bug 1587047 for details.
        // Using the full final row also ensures that the offset of the next chunk is
        // optimally aligned.
        let dst_size = dst_stride * size.height as usize;

        (dst_size, dst_stride)
    }

    /// Returns a `TextureUploader` which can be used to upload texture data to `texture`.
    /// Once uploads have been performed the uploader must be flushed with `TextureUploader::flush()`.
    pub fn upload_texture<'a>(
        &mut self,
        pbo_pool: &'a mut UploadPBOPool,
    ) -> TextureUploader<'a> {
        debug_assert!(self.inside_frame);

        pbo_pool.begin_frame(self);

        TextureUploader {
            buffers: Vec::new(),
            pbo_pool,
        }
    }

    /// Performs an immediate (non-PBO) texture upload.
    pub fn upload_texture_immediate<T: Texel>(
        &mut self,
        texture: &Texture,
        pixels: &[T]
    ) {
        self.bind_texture(DEFAULT_TEXTURE, texture, Swizzle::default());
        let desc = self.gl_describe_format(texture.format);
        self.gl.tex_sub_image_2d(
            texture.target,
            0,
            0,
            0,
            texture.size.width as gl::GLint,
            texture.size.height as gl::GLint,
            desc.external,
            desc.pixel_type,
            texels_to_u8_slice(pixels),
        );
    }

    pub fn read_pixels(&mut self, img_desc: &ImageDescriptor) -> Vec<u8> {
        let desc = self.gl_describe_format(img_desc.format);
        self.gl.read_pixels(
            0, 0,
            img_desc.size.width as i32,
            img_desc.size.height as i32,
            desc.read,
            desc.pixel_type,
        )
    }

    /// Read rectangle of pixels into the specified output slice.
    pub fn read_pixels_into(
        &mut self,
        rect: FramebufferIntRect,
        format: ImageFormat,
        output: &mut [u8],
    ) {
        let bytes_per_pixel = format.bytes_per_pixel();
        let desc = self.gl_describe_format(format);
        let size_in_bytes = (bytes_per_pixel * rect.size.width * rect.size.height) as usize;
        assert_eq!(output.len(), size_in_bytes);

        self.gl.flush();
        self.gl.read_pixels_into_buffer(
            rect.origin.x as _,
            rect.origin.y as _,
            rect.size.width as _,
            rect.size.height as _,
            desc.read,
            desc.pixel_type,
            output,
        );
    }

    /// Get texels of a texture into the specified output slice.
    pub fn get_tex_image_into(
        &mut self,
        texture: &Texture,
        format: ImageFormat,
        output: &mut [u8],
    ) {
        self.bind_texture(DEFAULT_TEXTURE, texture, Swizzle::default());
        let desc = self.gl_describe_format(format);
        self.gl.get_tex_image_into_buffer(
            texture.target,
            0,
            desc.external,
            desc.pixel_type,
            output,
        );
    }

    /// Attaches the provided texture to the current Read FBO binding.
    fn attach_read_texture_raw(&mut self, texture_id: gl::GLuint, target: gl::GLuint) {
        self.gl.framebuffer_texture_2d(
            gl::READ_FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            target,
            texture_id,
            0,
        )
    }

    pub fn attach_read_texture_external(
        &mut self, texture_id: gl::GLuint, target: ImageBufferKind
    ) {
        self.attach_read_texture_raw(texture_id, get_gl_target(target))
    }

    pub fn attach_read_texture(&mut self, texture: &Texture) {
        self.attach_read_texture_raw(texture.id, texture.target)
    }

    fn bind_vao_impl(&mut self, id: gl::GLuint) {
        debug_assert!(self.inside_frame);

        if self.bound_vao != id {
            self.bound_vao = id;
            self.gl.bind_vertex_array(id);
        }
    }

    pub fn bind_vao(&mut self, vao: &VAO) {
        self.bind_vao_impl(vao.id)
    }

    pub fn bind_custom_vao(&mut self, vao: &CustomVAO) {
        self.bind_vao_impl(vao.id)
    }

    fn create_vao_with_vbos(
        &mut self,
        descriptor: &VertexDescriptor,
        main_vbo_id: VBOId,
        instance_vbo_id: VBOId,
        instance_divisor: u32,
        ibo_id: IBOId,
        owns_vertices_and_indices: bool,
    ) -> VAO {
        let instance_stride = descriptor.instance_stride() as usize;
        let vao_id = self.gl.gen_vertex_arrays(1)[0];

        self.bind_vao_impl(vao_id);

        descriptor.bind(self.gl(), main_vbo_id, instance_vbo_id, instance_divisor);
        ibo_id.bind(self.gl()); // force it to be a part of VAO

        VAO {
            id: vao_id,
            ibo_id,
            main_vbo_id,
            instance_vbo_id,
            instance_stride,
            instance_divisor,
            owns_vertices_and_indices,
        }
    }

    pub fn create_custom_vao(
        &mut self,
        streams: &[Stream],
    ) -> CustomVAO {
        debug_assert!(self.inside_frame);

        let vao_id = self.gl.gen_vertex_arrays(1)[0];
        self.bind_vao_impl(vao_id);

        let mut attrib_index = 0;
        for stream in streams {
            VertexDescriptor::bind_attributes(
                stream.attributes,
                attrib_index,
                0,
                self.gl(),
                stream.vbo,
            );
            attrib_index += stream.attributes.len();
        }

        CustomVAO {
            id: vao_id,
        }
    }

    pub fn delete_custom_vao(&mut self, mut vao: CustomVAO) {
        self.gl.delete_vertex_arrays(&[vao.id]);
        vao.id = 0;
    }

    pub fn create_vbo<T>(&mut self) -> VBO<T> {
        let ids = self.gl.gen_buffers(1);
        VBO {
            id: ids[0],
            target: gl::ARRAY_BUFFER,
            allocated_count: 0,
            marker: PhantomData,
        }
    }

    pub fn delete_vbo<T>(&mut self, mut vbo: VBO<T>) {
        self.gl.delete_buffers(&[vbo.id]);
        vbo.id = 0;
    }

    pub fn create_vao(&mut self, descriptor: &VertexDescriptor, instance_divisor: u32) -> VAO {
        debug_assert!(self.inside_frame);

        let buffer_ids = self.gl.gen_buffers(3);
        let ibo_id = IBOId(buffer_ids[0]);
        let main_vbo_id = VBOId(buffer_ids[1]);
        let intance_vbo_id = VBOId(buffer_ids[2]);

        self.create_vao_with_vbos(descriptor, main_vbo_id, intance_vbo_id, instance_divisor, ibo_id, true)
    }

    pub fn delete_vao(&mut self, mut vao: VAO) {
        self.gl.delete_vertex_arrays(&[vao.id]);
        vao.id = 0;

        if vao.owns_vertices_and_indices {
            self.gl.delete_buffers(&[vao.ibo_id.0]);
            self.gl.delete_buffers(&[vao.main_vbo_id.0]);
        }

        self.gl.delete_buffers(&[vao.instance_vbo_id.0])
    }

    pub fn allocate_vbo<V>(
        &mut self,
        vbo: &mut VBO<V>,
        count: usize,
        usage_hint: VertexUsageHint,
    ) {
        debug_assert!(self.inside_frame);
        vbo.allocated_count = count;

        self.gl.bind_buffer(vbo.target, vbo.id);
        self.gl.buffer_data_untyped(
            vbo.target,
            (count * mem::size_of::<V>()) as _,
            ptr::null(),
            usage_hint.to_gl(),
        );
    }

    pub fn fill_vbo<V>(
        &mut self,
        vbo: &VBO<V>,
        data: &[V],
        offset: usize,
    ) {
        debug_assert!(self.inside_frame);
        assert!(offset + data.len() <= vbo.allocated_count);
        let stride = mem::size_of::<V>();

        self.gl.bind_buffer(vbo.target, vbo.id);
        self.gl.buffer_sub_data_untyped(
            vbo.target,
            (offset * stride) as _,
            (data.len() * stride) as _,
            data.as_ptr() as _,
        );
    }

    fn update_vbo_data<V>(
        &mut self,
        vbo: VBOId,
        vertices: &[V],
        usage_hint: VertexUsageHint,
    ) {
        debug_assert!(self.inside_frame);

        vbo.bind(self.gl());
        gl::buffer_data(self.gl(), gl::ARRAY_BUFFER, vertices, usage_hint.to_gl());
    }

    pub fn create_vao_with_new_instances(
        &mut self,
        descriptor: &VertexDescriptor,
        base_vao: &VAO,
    ) -> VAO {
        debug_assert!(self.inside_frame);

        let buffer_ids = self.gl.gen_buffers(1);
        let intance_vbo_id = VBOId(buffer_ids[0]);

        self.create_vao_with_vbos(
            descriptor,
            base_vao.main_vbo_id,
            intance_vbo_id,
            base_vao.instance_divisor,
            base_vao.ibo_id,
            false,
        )
    }

    pub fn update_vao_main_vertices<V>(
        &mut self,
        vao: &VAO,
        vertices: &[V],
        usage_hint: VertexUsageHint,
    ) {
        debug_assert_eq!(self.bound_vao, vao.id);
        self.update_vbo_data(vao.main_vbo_id, vertices, usage_hint)
    }

    pub fn update_vao_instances<V: Clone>(
        &mut self,
        vao: &VAO,
        instances: &[V],
        usage_hint: VertexUsageHint,
        // if `Some(count)`, each instance is repeated `count` times
        repeat: Option<NonZeroUsize>,
    ) {
        debug_assert_eq!(self.bound_vao, vao.id);
        debug_assert_eq!(vao.instance_stride as usize, mem::size_of::<V>());

        match repeat {
            Some(count) => {
                let target = gl::ARRAY_BUFFER;
                self.gl.bind_buffer(target, vao.instance_vbo_id.0);
                let size = instances.len() * count.get() * mem::size_of::<V>();
                self.gl.buffer_data_untyped(
                    target,
                    size as _,
                    ptr::null(),
                    usage_hint.to_gl(),
                );

                let ptr = match self.gl.get_type() {
                    gl::GlType::Gl => {
                        self.gl.map_buffer(target, gl::WRITE_ONLY)
                    }
                    gl::GlType::Gles => {
                        self.gl.map_buffer_range(target, 0, size as _, gl::MAP_WRITE_BIT)
                    }
                };
                assert!(!ptr.is_null());

                let buffer_slice = unsafe {
                    slice::from_raw_parts_mut(ptr as *mut V, instances.len() * count.get())
                };
                for (quad, instance) in buffer_slice.chunks_mut(4).zip(instances) {
                    quad[0] = instance.clone();
                    quad[1] = instance.clone();
                    quad[2] = instance.clone();
                    quad[3] = instance.clone();
                }
                self.gl.unmap_buffer(target);
            }
            None => {
                self.update_vbo_data(vao.instance_vbo_id, instances, usage_hint);
            }
        }
    }

    pub fn update_vao_indices<I>(&mut self, vao: &VAO, indices: &[I], usage_hint: VertexUsageHint) {
        debug_assert!(self.inside_frame);
        debug_assert_eq!(self.bound_vao, vao.id);

        vao.ibo_id.bind(self.gl());
        gl::buffer_data(
            self.gl(),
            gl::ELEMENT_ARRAY_BUFFER,
            indices,
            usage_hint.to_gl(),
        );
    }

    pub fn draw_triangles_u16(&mut self, first_vertex: i32, index_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        let _guard = CrashAnnotatorGuard::new(
            &self.crash_annotator,
            CrashAnnotation::DrawShader,
            &self.bound_program_name,
        );

        self.gl.draw_elements(
            gl::TRIANGLES,
            index_count,
            gl::UNSIGNED_SHORT,
            first_vertex as u32 * 2,
        );
    }

    pub fn draw_triangles_u32(&mut self, first_vertex: i32, index_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        let _guard = CrashAnnotatorGuard::new(
            &self.crash_annotator,
            CrashAnnotation::DrawShader,
            &self.bound_program_name,
        );

        self.gl.draw_elements(
            gl::TRIANGLES,
            index_count,
            gl::UNSIGNED_INT,
            first_vertex as u32 * 4,
        );
    }

    pub fn draw_nonindexed_points(&mut self, first_vertex: i32, vertex_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        let _guard = CrashAnnotatorGuard::new(
            &self.crash_annotator,
            CrashAnnotation::DrawShader,
            &self.bound_program_name,
        );

        self.gl.draw_arrays(gl::POINTS, first_vertex, vertex_count);
    }

    pub fn draw_nonindexed_lines(&mut self, first_vertex: i32, vertex_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        let _guard = CrashAnnotatorGuard::new(
            &self.crash_annotator,
            CrashAnnotation::DrawShader,
            &self.bound_program_name,
        );

        self.gl.draw_arrays(gl::LINES, first_vertex, vertex_count);
    }

    pub fn draw_indexed_triangles(&mut self, index_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        let _guard = CrashAnnotatorGuard::new(
            &self.crash_annotator,
            CrashAnnotation::DrawShader,
            &self.bound_program_name,
        );

        self.gl.draw_elements(
            gl::TRIANGLES,
            index_count,
            gl::UNSIGNED_SHORT,
            0,
        );
    }

    pub fn draw_indexed_triangles_instanced_u16(&mut self, index_count: i32, instance_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        let _guard = CrashAnnotatorGuard::new(
            &self.crash_annotator,
            CrashAnnotation::DrawShader,
            &self.bound_program_name,
        );

        self.gl.draw_elements_instanced(
            gl::TRIANGLES,
            index_count,
            gl::UNSIGNED_SHORT,
            0,
            instance_count,
        );
    }

    pub fn end_frame(&mut self) {
        self.reset_draw_target();
        self.reset_read_target();

        debug_assert!(self.inside_frame);
        self.inside_frame = false;

        self.gl.bind_texture(gl::TEXTURE_2D, 0);
        self.gl.use_program(0);

        for i in 0 .. self.bound_textures.len() {
            self.gl.active_texture(gl::TEXTURE0 + i as gl::GLuint);
            self.gl.bind_texture(gl::TEXTURE_2D, 0);
        }

        self.gl.active_texture(gl::TEXTURE0);

        self.frame_id.0 += 1;

        // Save any shaders compiled this frame to disk.
        // If this is the tenth frame then treat startup as complete, meaning the
        // current set of in-use shaders are the ones to load on the next startup.
        if let Some(ref cache) = self.cached_programs {
            cache.update_disk_cache(self.frame_id.0 == 10);
        }
    }

    pub fn clear_target(
        &self,
        color: Option<[f32; 4]>,
        depth: Option<f32>,
        rect: Option<FramebufferIntRect>,
    ) {
        let mut clear_bits = 0;

        if let Some(color) = color {
            self.gl.clear_color(color[0], color[1], color[2], color[3]);
            clear_bits |= gl::COLOR_BUFFER_BIT;
        }

        if let Some(depth) = depth {
            if cfg!(debug_assertions) {
                let mut mask = [0];
                unsafe {
                    self.gl.get_boolean_v(gl::DEPTH_WRITEMASK, &mut mask);
                }
                assert_ne!(mask[0], 0);
            }
            self.gl.clear_depth(depth as f64);
            clear_bits |= gl::DEPTH_BUFFER_BIT;
        }

        if clear_bits != 0 {
            match rect {
                Some(rect) => {
                    self.gl.enable(gl::SCISSOR_TEST);
                    self.gl.scissor(
                        rect.origin.x,
                        rect.origin.y,
                        rect.size.width,
                        rect.size.height,
                    );
                    self.gl.clear(clear_bits);
                    self.gl.disable(gl::SCISSOR_TEST);
                }
                None => {
                    self.gl.clear(clear_bits);
                }
            }
        }
    }

    pub fn enable_depth(&self, depth_func: DepthFunction) {
        assert!(self.depth_available, "Enabling depth test without depth target");
        self.gl.enable(gl::DEPTH_TEST);
        self.gl.depth_func(depth_func as gl::GLuint);
    }

    pub fn disable_depth(&self) {
        self.gl.disable(gl::DEPTH_TEST);
    }

    pub fn enable_depth_write(&self) {
        assert!(self.depth_available, "Enabling depth write without depth target");
        self.gl.depth_mask(true);
    }

    pub fn disable_depth_write(&self) {
        self.gl.depth_mask(false);
    }

    pub fn disable_stencil(&self) {
        self.gl.disable(gl::STENCIL_TEST);
    }

    pub fn set_scissor_rect(&self, rect: FramebufferIntRect) {
        self.gl.scissor(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );
    }

    pub fn enable_scissor(&self) {
        self.gl.enable(gl::SCISSOR_TEST);
    }

    pub fn disable_scissor(&self) {
        self.gl.disable(gl::SCISSOR_TEST);
    }

    pub fn enable_color_write(&self) {
        self.gl.color_mask(true, true, true, true);
    }

    pub fn disable_color_write(&self) {
        self.gl.color_mask(false, false, false, false);
    }

    pub fn set_blend(&mut self, enable: bool) {
        if enable {
            self.gl.enable(gl::BLEND);
        } else {
            self.gl.disable(gl::BLEND);
        }
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = false;
        }
    }

    fn set_blend_factors(
        &mut self,
        color: (gl::GLenum, gl::GLenum),
        alpha: (gl::GLenum, gl::GLenum),
    ) {
        self.gl.blend_equation(gl::FUNC_ADD);
        if color == alpha {
            self.gl.blend_func(color.0, color.1);
        } else {
            self.gl.blend_func_separate(color.0, color.1, alpha.0, alpha.1);
        }
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = false;
        }
    }

    pub fn set_blend_mode_alpha(&mut self) {
        self.set_blend_factors(
            (gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
        );
    }

    pub fn set_blend_mode_premultiplied_alpha(&mut self) {
        self.set_blend_factors(
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
        );
    }

    pub fn set_blend_mode_premultiplied_dest_out(&mut self) {
        self.set_blend_factors(
            (gl::ZERO, gl::ONE_MINUS_SRC_ALPHA),
            (gl::ZERO, gl::ONE_MINUS_SRC_ALPHA),
        );
    }

    pub fn set_blend_mode_multiply(&mut self) {
        self.set_blend_factors(
            (gl::ZERO, gl::SRC_COLOR),
            (gl::ZERO, gl::SRC_ALPHA),
        );
    }
    pub fn set_blend_mode_subpixel_pass0(&mut self) {
        self.set_blend_factors(
            (gl::ZERO, gl::ONE_MINUS_SRC_COLOR),
            (gl::ZERO, gl::ONE_MINUS_SRC_ALPHA),
        );
    }
    pub fn set_blend_mode_subpixel_pass1(&mut self) {
        self.set_blend_factors(
            (gl::ONE, gl::ONE),
            (gl::ONE, gl::ONE),
        );
    }
    pub fn set_blend_mode_subpixel_with_bg_color_pass0(&mut self) {
        self.set_blend_factors(
            (gl::ZERO, gl::ONE_MINUS_SRC_COLOR),
            (gl::ZERO, gl::ONE),
        );
    }
    pub fn set_blend_mode_subpixel_with_bg_color_pass1(&mut self) {
        self.set_blend_factors(
            (gl::ONE_MINUS_DST_ALPHA, gl::ONE),
            (gl::ZERO, gl::ONE),
        );
    }
    pub fn set_blend_mode_subpixel_with_bg_color_pass2(&mut self) {
        self.set_blend_factors(
            (gl::ONE, gl::ONE),
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
        );
    }
    pub fn set_blend_mode_subpixel_constant_text_color(&mut self, color: ColorF) {
        // color is an unpremultiplied color.
        self.gl.blend_color(color.r, color.g, color.b, 1.0);
        self.set_blend_factors(
            (gl::CONSTANT_COLOR, gl::ONE_MINUS_SRC_COLOR),
            (gl::CONSTANT_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
        );
    }
    pub fn set_blend_mode_subpixel_dual_source(&mut self) {
        self.set_blend_factors(
            (gl::ONE, gl::ONE_MINUS_SRC1_COLOR),
            (gl::ONE, gl::ONE_MINUS_SRC1_ALPHA),
        );
    }
    pub fn set_blend_mode_multiply_dual_source(&mut self) {
        self.set_blend_factors(
            (gl::ONE_MINUS_DST_ALPHA, gl::ONE_MINUS_SRC1_COLOR),
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
        );
    }
    pub fn set_blend_mode_screen(&mut self) {
        self.set_blend_factors(
            (gl::ONE, gl::ONE_MINUS_SRC_COLOR),
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
        );
    }
    pub fn set_blend_mode_exclusion(&mut self) {
        self.set_blend_factors(
            (gl::ONE_MINUS_DST_COLOR, gl::ONE_MINUS_SRC_COLOR),
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
        );
    }
    pub fn set_blend_mode_show_overdraw(&mut self) {
        self.set_blend_factors(
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
            (gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
        );
    }

    pub fn set_blend_mode_max(&mut self) {
        self.gl
            .blend_func_separate(gl::ONE, gl::ONE, gl::ONE, gl::ONE);
        self.gl.blend_equation_separate(gl::MAX, gl::FUNC_ADD);
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = false;
        }
    }
    pub fn set_blend_mode_min(&mut self) {
        self.gl
            .blend_func_separate(gl::ONE, gl::ONE, gl::ONE, gl::ONE);
        self.gl.blend_equation_separate(gl::MIN, gl::FUNC_ADD);
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = false;
        }
    }
    pub fn set_blend_mode_advanced(&mut self, mode: MixBlendMode) {
        self.gl.blend_equation(match mode {
            MixBlendMode::Normal => {
                // blend factor only make sense for the normal mode
                self.gl.blend_func_separate(gl::ZERO, gl::SRC_COLOR, gl::ZERO, gl::SRC_ALPHA);
                gl::FUNC_ADD
            },
            MixBlendMode::Multiply => gl::MULTIPLY_KHR,
            MixBlendMode::Screen => gl::SCREEN_KHR,
            MixBlendMode::Overlay => gl::OVERLAY_KHR,
            MixBlendMode::Darken => gl::DARKEN_KHR,
            MixBlendMode::Lighten => gl::LIGHTEN_KHR,
            MixBlendMode::ColorDodge => gl::COLORDODGE_KHR,
            MixBlendMode::ColorBurn => gl::COLORBURN_KHR,
            MixBlendMode::HardLight => gl::HARDLIGHT_KHR,
            MixBlendMode::SoftLight => gl::SOFTLIGHT_KHR,
            MixBlendMode::Difference => gl::DIFFERENCE_KHR,
            MixBlendMode::Exclusion => gl::EXCLUSION_KHR,
            MixBlendMode::Hue => gl::HSL_HUE_KHR,
            MixBlendMode::Saturation => gl::HSL_SATURATION_KHR,
            MixBlendMode::Color => gl::HSL_COLOR_KHR,
            MixBlendMode::Luminosity => gl::HSL_LUMINOSITY_KHR,
        });
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = false;
        }
    }

    pub fn supports_extension(&self, extension: &str) -> bool {
        supports_extension(&self.extensions, extension)
    }

    pub fn echo_driver_messages(&self) {
        if self.capabilities.supports_khr_debug {
            Device::log_driver_messages(self.gl());
        }
    }

    fn log_driver_messages(gl: &dyn gl::Gl) {
        for msg in gl.get_debug_messages() {
            let level = match msg.severity {
                gl::DEBUG_SEVERITY_HIGH => Level::Error,
                gl::DEBUG_SEVERITY_MEDIUM => Level::Warn,
                gl::DEBUG_SEVERITY_LOW => Level::Info,
                gl::DEBUG_SEVERITY_NOTIFICATION => Level::Debug,
                _ => Level::Trace,
            };
            let ty = match msg.ty {
                gl::DEBUG_TYPE_ERROR => "error",
                gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated",
                gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined",
                gl::DEBUG_TYPE_PORTABILITY => "portability",
                gl::DEBUG_TYPE_PERFORMANCE => "perf",
                gl::DEBUG_TYPE_MARKER => "marker",
                gl::DEBUG_TYPE_PUSH_GROUP => "group push",
                gl::DEBUG_TYPE_POP_GROUP => "group pop",
                gl::DEBUG_TYPE_OTHER => "other",
                _ => "?",
            };
            log!(level, "({}) {}", ty, msg.message);
        }
    }

    pub fn gl_describe_format(&self, format: ImageFormat) -> FormatDesc {
        match format {
            ImageFormat::R8 => FormatDesc {
                internal: gl::R8,
                external: gl::RED,
                read: gl::RED,
                pixel_type: gl::UNSIGNED_BYTE,
            },
            ImageFormat::R16 => FormatDesc {
                internal: gl::R16,
                external: gl::RED,
                read: gl::RED,
                pixel_type: gl::UNSIGNED_SHORT,
            },
            ImageFormat::BGRA8 => {
                FormatDesc {
                    internal: self.bgra_formats.internal,
                    external: self.bgra_formats.external,
                    read: gl::BGRA,
                    pixel_type: self.bgra_pixel_type,
                }
            },
            ImageFormat::RGBA8 => {
                FormatDesc {
                    internal: gl::RGBA8,
                    external: gl::RGBA,
                    read: gl::RGBA,
                    pixel_type: gl::UNSIGNED_BYTE,
                }
            },
            ImageFormat::RGBAF32 => FormatDesc {
                internal: gl::RGBA32F,
                external: gl::RGBA,
                read: gl::RGBA,
                pixel_type: gl::FLOAT,
            },
            ImageFormat::RGBAI32 => FormatDesc {
                internal: gl::RGBA32I,
                external: gl::RGBA_INTEGER,
                read: gl::RGBA_INTEGER,
                pixel_type: gl::INT,
            },
            ImageFormat::RG8 => FormatDesc {
                internal: gl::RG8,
                external: gl::RG,
                read: gl::RG,
                pixel_type: gl::UNSIGNED_BYTE,
            },
            ImageFormat::RG16 => FormatDesc {
                internal: gl::RG16,
                external: gl::RG,
                read: gl::RG,
                pixel_type: gl::UNSIGNED_SHORT,
            },
        }
    }

    /// Generates a memory report for the resources managed by the device layer.
    pub fn report_memory(&self, size_op_funs: &MallocSizeOfOps) -> MemoryReport {
        let mut report = MemoryReport::default();
        for dim in self.depth_targets.keys() {
            report.depth_target_textures += depth_target_size_in_bytes(dim);
        }
        #[cfg(feature = "sw_compositor")]
        {
            report.swgl += swgl::Context::report_memory(size_op_funs.size_of_op);
        }
        // unconditionally use size_op_funs
        let _ = size_op_funs;
        report
    }
}

pub struct FormatDesc {
    /// Format the texel data is internally stored in within a texture.
    pub internal: gl::GLenum,
    /// Format that we expect the data to be provided when filling the texture.
    pub external: gl::GLuint,
    /// Format to read the texels as, so that they can be uploaded as `external`
    /// later on.
    pub read: gl::GLuint,
    /// Associated pixel type.
    pub pixel_type: gl::GLuint,
}

#[derive(Debug)]
struct UploadChunk<'a> {
    rect: DeviceIntRect,
    stride: Option<i32>,
    offset: usize,
    format_override: Option<ImageFormat>,
    texture: &'a Texture,
}

#[derive(Debug)]
struct PixelBuffer<'a> {
    size_used: usize,
    // small vector avoids heap allocation for a single chunk
    chunks: SmallVec<[UploadChunk<'a>; 1]>,
    inner: UploadPBO,
    mapping: &'a mut [mem::MaybeUninit<u8>],
}

impl<'a> PixelBuffer<'a> {
    fn new(
        pbo: UploadPBO,
    ) -> Self {
        let mapping = unsafe {
            slice::from_raw_parts_mut(pbo.mapping.get_ptr().as_ptr(), pbo.pbo.reserved_size)
        };
        Self {
            size_used: 0,
            chunks: SmallVec::new(),
            inner: pbo,
            mapping,
        }
    }

    fn flush_chunks(&mut self, device: &mut Device) {
        for chunk in self.chunks.drain(..) {
            TextureUploader::update_impl(device, chunk);
        }
    }
}

impl<'a> Drop for PixelBuffer<'a> {
    fn drop(&mut self) {
        assert_eq!(self.chunks.len(), 0, "PixelBuffer must be flushed before dropping.");
    }
}

#[derive(Debug)]
enum PBOMapping {
    Unmapped,
    Transient(ptr::NonNull<mem::MaybeUninit<u8>>),
    Persistent(ptr::NonNull<mem::MaybeUninit<u8>>),
}

impl PBOMapping {
    fn get_ptr(&self) -> ptr::NonNull<mem::MaybeUninit<u8>> {
        match self {
            PBOMapping::Unmapped => unreachable!("Cannot get pointer to unmapped PBO."),
            PBOMapping::Transient(ptr) => *ptr,
            PBOMapping::Persistent(ptr) => *ptr,
        }
    }
}

/// A PBO for uploading texture data, managed by UploadPBOPool.
#[derive(Debug)]
struct UploadPBO {
    pbo: PBO,
    mapping: PBOMapping,
    can_recycle: bool,
}

impl UploadPBO {
    fn empty() -> Self {
        Self {
            pbo: PBO {
                id: 0,
                reserved_size: 0,
            },
            mapping: PBOMapping::Unmapped,
            can_recycle: false,
        }
    }
}

/// Allocates and recycles PBOs used for uploading texture data.
/// Tries to allocate and recycle PBOs of a fixed size, but will make exceptions when
/// a larger buffer is required or to work around driver bugs.
pub struct UploadPBOPool {
    /// Usage hint to provide to the driver for optimizations.
    usage_hint: VertexUsageHint,
    /// The preferred size, in bytes, of the buffers to allocate.
    default_size: usize,
    /// List of allocated PBOs ready to be re-used.
    available_buffers: Vec<UploadPBO>,
    /// PBOs which have been returned during the current frame,
    /// and do not yet have an associated sync object.
    returned_buffers: Vec<UploadPBO>,
    /// PBOs which are waiting until their sync object is signalled,
    /// indicating they can are ready to be re-used.
    waiting_buffers: Vec<(gl::GLsync, Vec<UploadPBO>)>,
    /// PBOs which have been orphaned.
    /// We can recycle their IDs but must reallocate their storage.
    orphaned_buffers: Vec<PBO>,
}

impl UploadPBOPool {
    pub fn new(device: &mut Device, default_size: usize) -> Self {
        let usage_hint = match device.upload_method {
            UploadMethod::Immediate => VertexUsageHint::Stream,
            UploadMethod::PixelBuffer(usage_hint) => usage_hint,
        };
        Self {
            usage_hint,
            default_size,
            available_buffers: Vec::new(),
            returned_buffers: Vec::new(),
            waiting_buffers: Vec::new(),
            orphaned_buffers: Vec::new(),
        }
    }

    /// To be called at the beginning of a series of uploads.
    /// Moves any buffers which are now ready to be used from the waiting list to the ready list.
    pub fn begin_frame(&mut self, device: &mut Device) {
        // Iterate through the waiting buffers and check if each fence has been signalled.
        // If a fence is signalled, move its corresponding buffers to the available list.
        // On error, delete the buffers. Stop when we find the first non-signalled fence,
        // and clean up the signalled fences.
        let mut first_not_signalled = self.waiting_buffers.len();
        for (i, (sync, buffers)) in self.waiting_buffers.iter_mut().enumerate() {
            match device.gl.client_wait_sync(*sync, 0, 0) {
                gl::TIMEOUT_EXPIRED => {
                    first_not_signalled = i;
                    break;
                },
                gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => {
                    self.available_buffers.extend(buffers.drain(..));
                }
                gl::WAIT_FAILED | _ => {
                    warn!("glClientWaitSync error in UploadPBOPool::begin_frame()");
                    for buffer in buffers.drain(..) {
                        device.delete_pbo(buffer.pbo);
                    }
                }
            }
        }

        // Delete signalled fences, and remove their now-empty Vecs from waiting_buffers.
        for (sync, _) in self.waiting_buffers.drain(0..first_not_signalled) {
            device.gl.delete_sync(sync);
        }
    }

    // To be called at the end of a series of uploads.
    // Creates a sync object, and adds the buffers returned during this frame to waiting_buffers.
    pub fn end_frame(&mut self, device: &mut Device) {
        if !self.returned_buffers.is_empty() {
            let sync = device.gl.fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            if !sync.is_null() {
                self.waiting_buffers.push((sync, mem::replace(&mut self.returned_buffers, Vec::new())))
            } else {
                warn!("glFenceSync error in UploadPBOPool::end_frame()");

                for buffer in self.returned_buffers.drain(..) {
                    device.delete_pbo(buffer.pbo);
                }
            }
        }
    }

    /// Obtain a PBO, either by reusing an existing PBO or allocating a new one.
    /// min_size specifies the minimum required size of the PBO. The returned PBO
    /// may be larger than required.
    fn get_pbo(&mut self, device: &mut Device, min_size: usize) -> Result<UploadPBO, ()> {

        // If min_size is smaller than our default size, then use the default size.
        // The exception to this is when due to driver bugs we cannot upload from
        // offsets other than zero within a PBO. In this case, there is no point in
        // allocating buffers larger than required, as they cannot be shared.
        let (can_recycle, size) = if min_size <= self.default_size && device.capabilities.supports_nonzero_pbo_offsets {
            (true, self.default_size)
        } else {
            (false, min_size)
        };

        // Try to recycle an already allocated PBO.
        if can_recycle {
            if let Some(mut buffer) = self.available_buffers.pop() {
                assert_eq!(buffer.pbo.reserved_size, size);
                assert!(buffer.can_recycle);

                device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, buffer.pbo.id);

                match buffer.mapping {
                    PBOMapping::Unmapped => {
                        // If buffer was unmapped then transiently map it.
                        let ptr = device.gl.map_buffer_range(
                            gl::PIXEL_UNPACK_BUFFER,
                            0,
                            buffer.pbo.reserved_size as _,
                            gl::MAP_WRITE_BIT | gl::MAP_UNSYNCHRONIZED_BIT,
                        ) as *mut _;

                        let ptr = ptr::NonNull::new(ptr).ok_or_else(|| {
                            error!("Failed to transiently map PBO of size {} bytes", buffer.pbo.reserved_size);
                        })?;

                        buffer.mapping = PBOMapping::Transient(ptr);
                    }
                    PBOMapping::Transient(_) => {
                        unreachable!("Transiently mapped UploadPBO must be unmapped before returning to pool.");
                    }
                    PBOMapping::Persistent(_) => {
                    }
                }

                return Ok(buffer);
            }
        }

        // Try to recycle a PBO ID (but not its allocation) from a previously allocated PBO.
        // If there are none available, create a new PBO.
        let mut pbo = match self.orphaned_buffers.pop() {
            Some(pbo) => pbo,
            None => device.create_pbo(),
        };

        assert_eq!(pbo.reserved_size, 0);
        pbo.reserved_size = size;

        device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, pbo.id);
        let mapping = if device.capabilities.supports_buffer_storage && can_recycle {
            device.gl.buffer_storage(
                gl::PIXEL_UNPACK_BUFFER,
                pbo.reserved_size as _,
                ptr::null(),
                gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT,
            );
            let ptr = device.gl.map_buffer_range(
                gl::PIXEL_UNPACK_BUFFER,
                0,
                pbo.reserved_size as _,
                // GL_MAP_COHERENT_BIT doesn't seem to work on Adreno, so use glFlushMappedBufferRange.
                // kvark notes that coherent memory can be faster on some platforms, such as nvidia,
                // so in the future we could choose which to use at run time.
                gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_FLUSH_EXPLICIT_BIT,
            ) as *mut _;

            let ptr = ptr::NonNull::new(ptr).ok_or_else(|| {
                error!("Failed to persistently map PBO of size {} bytes", pbo.reserved_size);
            })?;

            PBOMapping::Persistent(ptr)
        } else {
            device.gl.buffer_data_untyped(
                gl::PIXEL_UNPACK_BUFFER,
                pbo.reserved_size as _,
                ptr::null(),
                self.usage_hint.to_gl(),
            );
            let ptr = device.gl.map_buffer_range(
                gl::PIXEL_UNPACK_BUFFER,
                0,
                pbo.reserved_size as _,
                // Unlike the above code path, where we are re-mapping a buffer that has previously been unmapped,
                // this buffer has just been created there is no need for GL_MAP_UNSYNCHRONIZED_BIT.
                gl::MAP_WRITE_BIT,
            ) as *mut _;

            let ptr = ptr::NonNull::new(ptr).ok_or_else(|| {
                error!("Failed to transiently map PBO of size {} bytes", pbo.reserved_size);
            })?;

            PBOMapping::Transient(ptr)
        };

        Ok(UploadPBO { pbo, mapping, can_recycle })
    }

    /// Returns a PBO to the pool. If the PBO is recyclable it is placed in the waiting list.
    /// Otherwise we orphan the allocation immediately, and will subsequently reuse just the ID.
    fn return_pbo(&mut self, device: &mut Device, mut buffer: UploadPBO) {
        assert!(
            !matches!(buffer.mapping, PBOMapping::Transient(_)),
            "Transiently mapped UploadPBO must be unmapped before returning to pool.",
        );

        if buffer.can_recycle {
            self.returned_buffers.push(buffer);
        } else {
            device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, buffer.pbo.id);
            device.gl.buffer_data_untyped(
                gl::PIXEL_UNPACK_BUFFER,
                0,
                ptr::null(),
                gl::STREAM_DRAW,
            );
            buffer.pbo.reserved_size = 0;
            self.orphaned_buffers.push(buffer.pbo);
        }

        device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);
    }

    /// Frees all allocated buffers in response to a memory pressure event.
    pub fn on_memory_pressure(&mut self, device: &mut Device) {
        for buffer in self.available_buffers.drain(..) {
            device.delete_pbo(buffer.pbo);
        }
        for buffer in self.returned_buffers.drain(..) {
            device.delete_pbo(buffer.pbo)
        }
        for (sync, buffers) in self.waiting_buffers.drain(..) {
            device.gl.delete_sync(sync);
            for buffer in buffers {
                device.delete_pbo(buffer.pbo)
            }
        }
        // There is no need to delete orphaned PBOs on memory pressure.
    }

    /// Generates a memory report.
    pub fn report_memory(&self) -> MemoryReport {
        let mut report = MemoryReport::default();
        for buffer in &self.available_buffers {
            report.texture_upload_pbos += buffer.pbo.reserved_size;
        }
        for buffer in &self.returned_buffers {
            report.texture_upload_pbos += buffer.pbo.reserved_size;
        }
        for (_, buffers) in &self.waiting_buffers {
            for buffer in buffers {
                report.texture_upload_pbos += buffer.pbo.reserved_size;
            }
        }
        report
    }

    pub fn deinit(&mut self, device: &mut Device) {
        for buffer in self.available_buffers.drain(..) {
            device.delete_pbo(buffer.pbo);
        }
        for buffer in self.returned_buffers.drain(..) {
            device.delete_pbo(buffer.pbo)
        }
        for (sync, buffers) in self.waiting_buffers.drain(..) {
            device.gl.delete_sync(sync);
            for buffer in buffers {
                device.delete_pbo(buffer.pbo)
            }
        }
        for pbo in self.orphaned_buffers.drain(..) {
            device.delete_pbo(pbo);
        }
    }
}

/// Used to perform a series of texture uploads.
/// Create using Device::upload_texture(). Perform a series of uploads using either
/// upload(), or stage() and upload_staged(), then call flush().
pub struct TextureUploader<'a> {
    /// A list of buffers containing uploads that need to be flushed.
    buffers: Vec<PixelBuffer<'a>>,
    /// Pool used to obtain PBOs to fill with texture data.
    pub pbo_pool: &'a mut UploadPBOPool,
}

impl<'a> Drop for TextureUploader<'a> {
    fn drop(&mut self) {
        assert!(
            thread::panicking() || self.buffers.is_empty(),
            "TextureUploader must be flushed before it is dropped."
        );
    }
}

/// A buffer used to manually stage data to be uploaded to a texture.
/// Created by calling TextureUploader::stage(), the data can then be written to via get_mapping().
#[derive(Debug)]
pub struct UploadStagingBuffer<'a> {
    /// The PixelBuffer containing this upload.
    buffer: PixelBuffer<'a>,
    /// The offset of this upload within the PixelBuffer.
    offset: usize,
    /// The size of this upload.
    size: usize,
    /// The stride of the data within the buffer.
    stride: usize,
}

impl<'a> UploadStagingBuffer<'a> {
    /// Returns the required stride of the data to be written to the buffer.
    pub fn get_stride(&self) -> usize {
        self.stride
    }

    /// Returns a mapping of the data in the buffer, to be written to.
    pub fn get_mapping(&mut self) -> &mut [mem::MaybeUninit<u8>] {
        &mut self.buffer.mapping[self.offset..self.offset + self.size]
    }
}

impl<'a> TextureUploader<'a> {
    /// Returns an UploadStagingBuffer which can be used to manually stage data to be uploaded.
    /// Once the data has been staged, it can be uploaded with upload_staged().
    pub fn stage(
        &mut self,
        device: &mut Device,
        format: ImageFormat,
        size: DeviceIntSize,
    ) -> Result<UploadStagingBuffer<'a>, ()> {
        assert!(matches!(device.upload_method, UploadMethod::PixelBuffer(_)), "Texture uploads should only be staged when using pixel buffers.");

        // for optimal PBO texture uploads the offset and stride of the data in
        // the buffer may have to be a multiple of a certain value.
        let (dst_size, dst_stride) = device.required_upload_size_and_stride(
            size,
            format,
        );

        // Find a pixel buffer with enough space remaining, creating a new one if required.
        let buffer_index = self.buffers.iter().position(|buffer| {
            buffer.size_used + dst_size <= buffer.inner.pbo.reserved_size
        });
        let buffer = match buffer_index {
            Some(i) => self.buffers.swap_remove(i),
            None => PixelBuffer::new(self.pbo_pool.get_pbo(device, dst_size)?),
        };

        if !device.capabilities.supports_nonzero_pbo_offsets {
            assert_eq!(buffer.size_used, 0, "PBO uploads from non-zero offset are not supported.");
        }
        assert!(buffer.size_used + dst_size <= buffer.inner.pbo.reserved_size, "PixelBuffer is too small");

        let offset = buffer.size_used;

        Ok(UploadStagingBuffer {
            buffer,
            offset,
            size: dst_size,
            stride: dst_stride,
        })
    }

    /// Uploads manually staged texture data to the specified texture.
    pub fn upload_staged(
        &mut self,
        device: &mut Device,
        texture: &'a Texture,
        rect: DeviceIntRect,
        format_override: Option<ImageFormat>,
        mut staging_buffer: UploadStagingBuffer<'a>,
    ) -> usize {
        let size = staging_buffer.size;

        staging_buffer.buffer.chunks.push(UploadChunk {
            rect,
            stride: Some(staging_buffer.stride as i32),
            offset: staging_buffer.offset,
            format_override,
            texture,
        });
        staging_buffer.buffer.size_used += staging_buffer.size;

        // Flush the buffer if it is full, otherwise return it to the uploader for further use.
        if staging_buffer.buffer.size_used < staging_buffer.buffer.inner.pbo.reserved_size {
            self.buffers.push(staging_buffer.buffer);
        } else {
            Self::flush_buffer(device, self.pbo_pool, staging_buffer.buffer);
        }

        size
    }

    /// Uploads texture data to the specified texture.
    pub fn upload<T>(
        &mut self,
        device: &mut Device,
        texture: &'a Texture,
        mut rect: DeviceIntRect,
        stride: Option<i32>,
        format_override: Option<ImageFormat>,
        data: *const T,
        len: usize,
    ) -> usize {
        // Textures dimensions may have been clamped by the hardware. Crop the
        // upload region to match.
        let cropped = rect.intersection(
            &DeviceIntRect::new(DeviceIntPoint::zero(), texture.get_dimensions())
        );
        if cfg!(debug_assertions) && cropped.map_or(true, |r| r != rect) {
            warn!("Cropping texture upload {:?} to {:?}", rect, cropped);
        }
        rect = match cropped {
            None => return 0,
            Some(r) => r,
        };

        let bytes_pp = texture.format.bytes_per_pixel() as usize;
        let width_bytes = rect.size.width as usize * bytes_pp;

        let src_stride = stride.map_or(width_bytes, |stride| {
            assert!(stride >= 0);
            stride as usize
        });
        let src_size = (rect.size.height as usize - 1) * src_stride + width_bytes;
        assert!(src_size <= len * mem::size_of::<T>());

        match device.upload_method {
            UploadMethod::Immediate => {
                if cfg!(debug_assertions) {
                    let mut bound_buffer = [0];
                    unsafe {
                        device.gl.get_integer_v(gl::PIXEL_UNPACK_BUFFER_BINDING, &mut bound_buffer);
                    }
                    assert_eq!(bound_buffer[0], 0, "GL_PIXEL_UNPACK_BUFFER must not be bound for immediate uploads.");
                }

                Self::update_impl(device, UploadChunk {
                    rect,
                    stride: Some(src_stride as i32),
                    offset: data as _,
                    format_override,
                    texture,
                });

                width_bytes * rect.size.height as usize
            }
            UploadMethod::PixelBuffer(_) => {
                let mut staging_buffer = match self.stage(device, texture.format, rect.size) {
                    Ok(staging_buffer) => staging_buffer,
                    Err(_) => return 0,
                };
                let dst_stride = staging_buffer.get_stride();

                unsafe {
                    let src: &[mem::MaybeUninit<u8>] = slice::from_raw_parts(data as *const _, src_size);

                    if src_stride == dst_stride {
                        // the stride is already optimal, so simply copy
                        // the data as-is in to the buffer
                        staging_buffer.get_mapping()[..src_size].copy_from_slice(src);
                    } else {
                        // copy the data line-by-line in to the buffer so
                        // that it has an optimal stride
                        for y in 0..rect.size.height as usize {
                            let src_start = y * src_stride;
                            let src_end = src_start + width_bytes;
                            let dst_start = y * staging_buffer.get_stride();
                            let dst_end = dst_start + width_bytes;

                            staging_buffer.get_mapping()[dst_start..dst_end].copy_from_slice(&src[src_start..src_end])
                        }
                    }
                }

                self.upload_staged(device, texture, rect, format_override, staging_buffer)
            }
        }
    }

    fn flush_buffer(device: &mut Device, pbo_pool: &mut UploadPBOPool, mut buffer: PixelBuffer) {
        device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, buffer.inner.pbo.id);
        match buffer.inner.mapping {
            PBOMapping::Unmapped => unreachable!("UploadPBO should be mapped at this stage."),
            PBOMapping::Transient(_) => {
                device.gl.unmap_buffer(gl::PIXEL_UNPACK_BUFFER);
                buffer.inner.mapping = PBOMapping::Unmapped;
            }
            PBOMapping::Persistent(_) => {
                device.gl.flush_mapped_buffer_range(gl::PIXEL_UNPACK_BUFFER, 0, buffer.size_used as _);
            }
        }
        buffer.flush_chunks(device);
        let pbo = mem::replace(&mut buffer.inner, UploadPBO::empty());
        pbo_pool.return_pbo(device, pbo);
    }

    /// Flushes all pending texture uploads. Must be called after all
    /// required upload() or upload_staged() calls have been made.
    pub fn flush(mut self, device: &mut Device) {
        for buffer in self.buffers.drain(..) {
            Self::flush_buffer(device, self.pbo_pool, buffer);
        }

        device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);
    }

    fn update_impl(device: &mut Device, chunk: UploadChunk) {
        device.bind_texture(DEFAULT_TEXTURE, chunk.texture, Swizzle::default());

        let format = chunk.format_override.unwrap_or(chunk.texture.format);
        let (gl_format, bpp, data_type) = match format {
            ImageFormat::R8 => (gl::RED, 1, gl::UNSIGNED_BYTE),
            ImageFormat::R16 => (gl::RED, 2, gl::UNSIGNED_SHORT),
            ImageFormat::BGRA8 => (device.bgra_formats.external, 4, device.bgra_pixel_type),
            ImageFormat::RGBA8 => (gl::RGBA, 4, gl::UNSIGNED_BYTE),
            ImageFormat::RG8 => (gl::RG, 2, gl::UNSIGNED_BYTE),
            ImageFormat::RG16 => (gl::RG, 4, gl::UNSIGNED_SHORT),
            ImageFormat::RGBAF32 => (gl::RGBA, 16, gl::FLOAT),
            ImageFormat::RGBAI32 => (gl::RGBA_INTEGER, 16, gl::INT),
        };

        let row_length = match chunk.stride {
            Some(value) => value / bpp,
            None => chunk.texture.size.width,
        };

        if chunk.stride.is_some() {
            device.gl.pixel_store_i(
                gl::UNPACK_ROW_LENGTH,
                row_length as _,
            );
        }

        let pos = chunk.rect.origin;
        let size = chunk.rect.size;

        match chunk.texture.target {
            gl::TEXTURE_2D | gl::TEXTURE_RECTANGLE | gl::TEXTURE_EXTERNAL_OES => {
                device.gl.tex_sub_image_2d_pbo(
                    chunk.texture.target,
                    0,
                    pos.x as _,
                    pos.y as _,
                    size.width as _,
                    size.height as _,
                    gl_format,
                    data_type,
                    chunk.offset,
                );
            }
            _ => panic!("BUG: Unexpected texture target!"),
        }

        // If using tri-linear filtering, build the mip-map chain for this texture.
        if chunk.texture.filter == TextureFilter::Trilinear {
            device.gl.generate_mipmap(chunk.texture.target);
        }

        // Reset row length to 0, otherwise the stride would apply to all texture uploads.
        if chunk.stride.is_some() {
            device.gl.pixel_store_i(gl::UNPACK_ROW_LENGTH, 0 as _);
        }
    }
}

fn texels_to_u8_slice<T: Texel>(texels: &[T]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(texels.as_ptr() as *const u8, texels.len() * mem::size_of::<T>())
    }
}
