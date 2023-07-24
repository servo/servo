/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::super::shader_source::{OPTIMIZED_SHADERS, UNOPTIMIZED_SHADERS};
use api::{ColorF, ImageDescriptor, ImageFormat, MemoryReport};
use api::{MixBlendMode, TextureTarget, VoidPtrToSizeFn};
use api::units::*;
use euclid::default::Transform3D;
use gleam::gl;
use crate::internal_types::{FastHashMap, LayerIndex, RenderTargetInfo, Swizzle, SwizzleSettings};
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
#[derive(Copy, Clone, Debug, PartialEq)]
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

pub fn get_gl_target(target: TextureTarget) -> gl::GLuint {
    match target {
        TextureTarget::Default => gl::TEXTURE_2D,
        TextureTarget::Array => gl::TEXTURE_2D_ARRAY,
        TextureTarget::Rect => gl::TEXTURE_RECTANGLE,
        TextureTarget::External => gl::TEXTURE_EXTERNAL_OES,
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

    fn bind(&self, gl: &dyn gl::Gl, main: VBOId, instance: VBOId) {
        Self::bind_attributes(self.vertex_attributes, 0, 0, gl, main);

        if !self.instance_attributes.is_empty() {
            Self::bind_attributes(
                self.instance_attributes,
                self.vertex_attributes.len(),
                1, gl, instance,
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
    _swizzle: Swizzle,
    uv_rect: TexelRect,
}

impl ExternalTexture {
    pub fn new(
        id: u32,
        target: TextureTarget,
        swizzle: Swizzle,
        uv_rect: TexelRect,
    ) -> Self {
        ExternalTexture {
            id,
            target: get_gl_target(target),
            _swizzle: swizzle,
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
    layer_count: i32,
    format: ImageFormat,
    size: DeviceIntSize,
    filter: TextureFilter,
    flags: TextureFlags,
    /// An internally mutable swizzling state that may change between batches.
    active_swizzle: Cell<Swizzle>,
    /// Framebuffer Objects, one for each layer of the texture, allowing this
    /// texture to be rendered to. Empty if this texture is not used as a render
    /// target.
    fbos: Vec<FBOId>,
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
    /// Note that we always fill fbos, and then lazily create fbos_with_depth
    /// when needed. We could make both lazy (i.e. render targets would have one
    /// or the other, but not both, unless they were actually used in both
    /// configurations). But that would complicate a lot of logic in this module,
    /// and FBOs are cheap enough to create.
    fbos_with_depth: Vec<FBOId>,
    /// If we are unable to blit directly to a texture array then we need
    /// an intermediate renderbuffer.
    blit_workaround_buffer: Option<(RBOId, FBOId)>,
    last_frame_used: GpuFrameId,
}

impl Texture {
    pub fn get_dimensions(&self) -> DeviceIntSize {
        self.size
    }

    pub fn get_layer_count(&self) -> i32 {
        self.layer_count
    }

    pub fn get_format(&self) -> ImageFormat {
        self.format
    }

    pub fn get_filter(&self) -> TextureFilter {
        self.filter
    }

    pub fn supports_depth(&self) -> bool {
        !self.fbos_with_depth.is_empty()
    }

    pub fn last_frame_used(&self) -> GpuFrameId {
        self.last_frame_used
    }

    pub fn used_in_frame(&self, frame_id: GpuFrameId) -> bool {
        self.last_frame_used == frame_id
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

    /// Returns the number of bytes (generally in GPU memory) that each layer of
    /// this texture consumes.
    pub fn layer_size_in_bytes(&self) -> usize {
        assert!(self.layer_count > 0 || self.size.width + self.size.height == 0);
        let bpp = self.format.bytes_per_pixel() as usize;
        let w = self.size.width as usize;
        let h = self.size.height as usize;
        bpp * w * h
    }

    /// Returns the number of bytes (generally in GPU memory) that this texture
    /// consumes.
    pub fn size_in_bytes(&self) -> usize {
        self.layer_size_in_bytes() * (self.layer_count as usize)
    }

    #[cfg(feature = "replay")]
    pub fn into_external(mut self) -> ExternalTexture {
        let ext = ExternalTexture {
            id: self.id,
            target: self.target,
            _swizzle: Swizzle::default(),
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
            "renderer::deinit not called"
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

        let full_name = &Self::full_name(name, features);

        let optimized_source = if device.use_optimized_shaders {
            OPTIMIZED_SHADERS.get(&(gl_version, full_name)).or_else(|| {
                warn!("Missing optimized shader source for {}", full_name);
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
            source_type,
            digest: hasher.into(),
        }
    }

    fn compute_source(&self, device: &Device, kind: ShaderKind) -> String {
        let full_name = Self::full_name(self.base_filename, &self.features);
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

    fn full_name(base_filename: &'static str, features: &[&'static str]) -> String {
        if features.is_empty() {
            base_filename.to_string()
        } else {
            format!("{}_{}", base_filename, features.join("_"))
        }
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
    /// Whether we are able to use `glBlitFramebuffers` with the draw fbo
    /// bound to a non-0th layer of a texture array. This is buggy on
    /// Adreno devices.
    pub supports_blit_to_texture_array: bool,
    /// Whether we can use the pixel local storage functionality that
    /// is available on some mobile GPUs. This allows fast access to
    /// the per-pixel tile memory.
    pub supports_pixel_local_storage: bool,
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
    bound_vao: gl::GLuint,
    bound_read_fbo: FBOId,
    bound_draw_fbo: FBOId,
    program_mode_id: UniformLocation,
    default_read_fbo: FBOId,
    default_draw_fbo: FBOId,

    /// Track depth state for assertions. Note that the default FBO has depth,
    /// so this defaults to true.
    depth_available: bool,

    upload_method: UploadMethod,

    // HW or API capabilities
    capabilities: Capabilities,

    color_formats: TextureFormatPair<ImageFormat>,
    bgra_formats: TextureFormatPair<gl::GLuint>,
    swizzle_settings: SwizzleSettings,
    depth_format: gl::GLuint,

    /// Map from texture dimensions to shared depth buffers for render targets.
    ///
    /// Render targets often have the same width/height, so we can save memory
    /// by sharing these across targets.
    depth_targets: FastHashMap<DeviceIntSize, SharedDepthTarget>,

    // debug
    inside_frame: bool,

    // resources
    resource_override_path: Option<PathBuf>,

    /// Whether to use shaders that have been optimized at build time.
    use_optimized_shaders: bool,

    max_texture_size: i32,
    max_texture_layers: u32,
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

    optimal_pbo_stride: StrideAlignment,

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
        /// The slice within the texture array to draw to
        layer: LayerIndex,
        /// Whether to draw with the texture's associated depth target
        with_depth: bool,
        /// Workaround buffers for devices with broken texture array copy implementation
        blit_workaround_buffer: Option<(RBOId, FBOId)>,
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
        layer: usize,
        with_depth: bool,
    ) -> Self {
        let fbo_id = if with_depth {
            texture.fbos_with_depth[layer]
        } else {
            texture.fbos[layer]
        };

        DrawTarget::Texture {
            dimensions: texture.get_dimensions(),
            fbo_id,
            with_depth,
            layer,
            blit_workaround_buffer: texture.blit_workaround_buffer,
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
            DrawTarget::Texture { .. } | DrawTarget::External { .. } => (),
            DrawTarget::NativeSurface { .. } => {
                panic!("bug: is this ever used for native surfaces?");
            }
        }
        fb_rect
    }

    /// Given a scissor rect, convert it to the right coordinate space
    /// depending on the draw target kind. If no scissor rect was supplied,
    /// returns a scissor rect that encloses the entire render target.
    pub fn build_scissor_rect(
        &self,
        scissor_rect: Option<DeviceIntRect>,
        content_origin: DeviceIntPoint,
    ) -> FramebufferIntRect {
        let dimensions = self.dimensions();

        match scissor_rect {
            Some(scissor_rect) => match *self {
                DrawTarget::Default { ref rect, .. } => {
                    self.to_framebuffer_rect(scissor_rect.translate(-content_origin.to_vector()))
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
}

impl ReadTarget {
    pub fn from_texture(
        texture: &Texture,
        layer: usize,
    ) -> Self {
        ReadTarget::Texture {
            fbo_id: texture.fbos[layer],
        }
    }
}

impl From<DrawTarget> for ReadTarget {
    fn from(t: DrawTarget) -> Self {
        match t {
            DrawTarget::Default { .. } => ReadTarget::Default,
            DrawTarget::NativeSurface { .. } => {
                unreachable!("bug: native surfaces cannot be read targets");
            }
            DrawTarget::Texture { fbo_id, .. } =>
                ReadTarget::Texture { fbo_id },
            DrawTarget::External { fbo, .. } =>
                ReadTarget::External { fbo },
        }
    }
}

impl Device {
    pub fn new(
        mut gl: Rc<dyn gl::Gl>,
        resource_override_path: Option<PathBuf>,
        use_optimized_shaders: bool,
        upload_method: UploadMethod,
        cached_programs: Option<Rc<ProgramCache>>,
        allow_pixel_local_storage_support: bool,
        allow_texture_storage_support: bool,
        allow_texture_swizzling: bool,
        dump_shader_source: Option<String>,
        surface_origin_is_top_left: bool,
        panic_on_gl_error: bool,
    ) -> Device {
        let mut max_texture_size = [0];
        let mut max_texture_layers = [0];
        unsafe {
            gl.get_integer_v(gl::MAX_TEXTURE_SIZE, &mut max_texture_size);
            gl.get_integer_v(gl::MAX_ARRAY_TEXTURE_LAYERS, &mut max_texture_layers);
        }

        let max_texture_size = max_texture_size[0];
        let max_texture_layers = max_texture_layers[0] as u32;
        let renderer_name = gl.get_string(gl::RENDERER);

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
        let gl_version = gl.get_string(gl::VERSION);

        let supports_texture_storage = allow_texture_storage_support &&
            match gl.get_type() {
                gl::GlType::Gl => supports_extension(&extensions, "GL_ARB_texture_storage"),
                // ES 3 technically always supports glTexStorage, but only check here for the extension
                // necessary to interact with BGRA.
                gl::GlType::Gles => supports_extension(&extensions, "GL_EXT_texture_storage"),
            };
        let supports_texture_swizzle = allow_texture_swizzling &&
            match gl.get_type() {
                // see https://www.g-truc.net/post-0734.html
                gl::GlType::Gl => gl_version.as_str() >= "3.3" ||
                    supports_extension(&extensions, "GL_ARB_texture_swizzle"),
                gl::GlType::Gles => true,
            };

        let (color_formats, bgra_formats, bgra8_sampling_swizzle, texture_storage_usage) = match gl.get_type() {
            // There is `glTexStorage`, use it and expect RGBA on the input.
            gl::GlType::Gl if supports_texture_storage && supports_texture_swizzle => (
                TextureFormatPair::from(ImageFormat::RGBA8),
                TextureFormatPair { internal: gl::RGBA8, external: gl::RGBA },
                Swizzle::Bgra, // pretend it's RGBA, rely on swizzling
                TexStorageUsage::Always
            ),
            // There is no `glTexStorage`, upload as `glTexImage` with BGRA input.
            gl::GlType::Gl => (
                TextureFormatPair { internal: ImageFormat::RGBA8, external: ImageFormat::BGRA8 },
                TextureFormatPair { internal: gl::RGBA, external: gl::BGRA },
                Swizzle::Rgba, // converted on uploads by the driver, no swizzling needed
                TexStorageUsage::Never
            ),
            // We can use glTexStorage with BGRA8 as the internal format.
            gl::GlType::Gles if supports_gles_bgra && supports_texture_storage => (
                TextureFormatPair::from(ImageFormat::BGRA8),
                TextureFormatPair { internal: gl::BGRA8_EXT, external: gl::BGRA_EXT },
                Swizzle::Rgba, // no conversion needed
                TexStorageUsage::Always,
            ),
            // For BGRA8 textures we must use the unsized BGRA internal
            // format and glTexImage. If texture storage is supported we can
            // use it for other formats, which is always the case for ES 3.
            // We can't use glTexStorage with BGRA8 as the internal format.
            gl::GlType::Gles if supports_gles_bgra && !avoid_tex_image => (
                TextureFormatPair::from(ImageFormat::RGBA8),
                TextureFormatPair::from(gl::BGRA_EXT),
                Swizzle::Rgba, // no conversion needed
                TexStorageUsage::NonBGRA8,
            ),
            // BGRA is not supported as an internal format, therefore we will
            // use RGBA. The swizzling will happen at the texture unit.
            gl::GlType::Gles if supports_texture_swizzle => (
                TextureFormatPair::from(ImageFormat::RGBA8),
                TextureFormatPair { internal: gl::RGBA8, external: gl::RGBA },
                Swizzle::Bgra, // pretend it's RGBA, rely on swizzling
                TexStorageUsage::Always,
            ),
            // BGRA and swizzling are not supported. We force the conversion done by the driver.
            gl::GlType::Gles => (
                TextureFormatPair::from(ImageFormat::RGBA8),
                TextureFormatPair { internal: gl::RGBA8, external: gl::BGRA },
                Swizzle::Rgba,
                TexStorageUsage::Always,
            ),
        };

        let is_software_webrender = renderer_name.starts_with("Software WebRender");
        let (depth_format, upload_method) = if is_software_webrender {
            (gl::DEPTH_COMPONENT16, UploadMethod::Immediate)
        } else {
            (gl::DEPTH_COMPONENT24, upload_method)
        };

        info!("GL texture cache {:?}, bgra {:?} swizzle {:?}, texture storage {:?}, depth {:?}",
            color_formats, bgra_formats, bgra8_sampling_swizzle, texture_storage_usage, depth_format);
        let supports_copy_image_sub_data = supports_extension(&extensions, "GL_EXT_copy_image") ||
            supports_extension(&extensions, "GL_ARB_copy_image");

        // Due to a bug on Adreno devices, blitting to an fbo bound to
        // a non-0th layer of a texture array is not supported.
        let supports_blit_to_texture_array = !renderer_name.starts_with("Adreno");

        // Check if the device supports the two extensions needed in order to use
        // pixel local storage.
        // TODO(gw): Consider if we can remove fb fetch / init, by using PLS for opaque pass too.
        // TODO(gw): Support EXT_shader_framebuffer_fetch as well.
        let ext_pixel_local_storage = supports_extension(&extensions, "GL_EXT_shader_pixel_local_storage");
        let ext_framebuffer_fetch = supports_extension(&extensions, "GL_ARM_shader_framebuffer_fetch");
        let supports_pixel_local_storage =
            allow_pixel_local_storage_support &&
            ext_framebuffer_fetch &&
            ext_pixel_local_storage;

        let is_adreno = renderer_name.starts_with("Adreno");

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

        let is_amd_macos = cfg!(target_os = "macos") && renderer_name.starts_with("AMD");

        // On certain GPUs PBO texture upload is only performed asynchronously
        // if the stride of the data is a multiple of a certain value.
        // On Adreno it must be a multiple of 64 pixels, meaning value in bytes
        // varies with the texture format.
        // On AMD Mac, it must always be a multiple of 256 bytes.
        // Other platforms may have similar requirements and should be added
        // here.
        // The default value should be 4 bytes.
        let optimal_pbo_stride = if is_adreno {
            StrideAlignment::Pixels(NonZeroUsize::new(64).unwrap())
        } else if is_amd_macos {
            StrideAlignment::Bytes(NonZeroUsize::new(256).unwrap())
        } else {
            StrideAlignment::Bytes(NonZeroUsize::new(4).unwrap())
        };

        // On AMD Macs there is a driver bug which causes some texture uploads
        // from a non-zero offset within a PBO to fail. See bug 1603783.
        let supports_nonzero_pbo_offsets = !is_amd_macos;

        Device {
            gl,
            base_gl: None,
            resource_override_path,
            use_optimized_shaders,
            upload_method,
            inside_frame: false,

            capabilities: Capabilities {
                supports_multisampling: false, //TODO
                supports_copy_image_sub_data,
                supports_blit_to_texture_array,
                supports_pixel_local_storage,
                supports_advanced_blend_equation,
                supports_dual_source_blending,
                supports_khr_debug,
                supports_texture_swizzle,
                supports_nonzero_pbo_offsets,
                supports_texture_usage,
                renderer_name,
            },

            color_formats,
            bgra_formats,
            swizzle_settings: SwizzleSettings {
                bgra8_sampling_swizzle,
            },
            depth_format,

            depth_targets: FastHashMap::default(),

            bound_textures: [0; 16],
            bound_program: 0,
            bound_vao: 0,
            bound_read_fbo: FBOId(0),
            bound_draw_fbo: FBOId(0),
            program_mode_id: UniformLocation::INVALID,
            default_read_fbo: FBOId(0),
            default_draw_fbo: FBOId(0),

            depth_available: true,

            max_texture_size,
            max_texture_layers,
            cached_programs,
            frame_id: GpuFrameId(0),
            extensions,
            texture_storage_usage,
            requires_null_terminated_shader_source,
            requires_texture_external_unbind,
            optimal_pbo_stride,
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

    /// Returns the limit on texture array layers.
    pub fn max_texture_layers(&self) -> usize {
        self.max_texture_layers as usize
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

    pub fn optimal_pbo_stride(&self) -> StrideAlignment {
        self.optimal_pbo_stride
    }

    pub fn reset_state(&mut self) {
        for i in 0 .. self.bound_textures.len() {
            self.bound_textures[i] = 0;
            self.gl.active_texture(gl::TEXTURE0 + i as gl::GLuint);
            self.gl.bind_texture(gl::TEXTURE_2D, 0);
        }

        self.bound_vao = 0;
        self.gl.bind_vertex_array(0);

        self.bound_read_fbo = self.default_read_fbo;
        self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, self.bound_read_fbo.0);

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
        gl: &dyn gl::Gl,
        name: &str,
        shader_type: gl::GLenum,
        source: &String,
        requires_null_terminated_shader_source: bool,
    ) -> Result<gl::GLuint, ShaderError> {
        debug!("compile {}", name);
        let id = gl.create_shader(shader_type);
        if requires_null_terminated_shader_source {
            // Ensure the source strings we pass to glShaderSource are
            // null-terminated on buggy platforms.
            use std::ffi::CString;
            let terminated_source = CString::new(source.as_bytes()).unwrap();
            gl.shader_source(id, &[terminated_source.as_bytes_with_nul()]);
        } else {
            gl.shader_source(id, &[source.as_bytes()]);
        }
        gl.compile_shader(id);
        let log = gl.get_shader_info_log(id);
        let mut status = [0];
        unsafe {
            gl.get_shader_iv(id, gl::COMPILE_STATUS, &mut status);
        }
        if status[0] == 0 {
            error!("Failed to compile shader: {}\n{}", name, log);
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
        if being_profiled && !using_wrapper {
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

    pub fn bind_read_target_impl(&mut self, fbo_id: FBOId) {
        debug_assert!(self.inside_frame);

        if self.bound_read_fbo != fbo_id {
            self.bound_read_fbo = fbo_id;
            fbo_id.bind(self.gl(), FBOTarget::Read);
        }
    }

    pub fn bind_read_target(&mut self, target: ReadTarget) {
        let fbo_id = match target {
            ReadTarget::Default => self.default_read_fbo,
            ReadTarget::Texture { fbo_id } => fbo_id,
            ReadTarget::External { fbo } => fbo,
        };

        self.bind_read_target_impl(fbo_id)
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
        self.bind_read_target_impl(fbo);
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
                (self.default_draw_fbo, rect, true)
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
            let vs_id = match Device::compile_shader(&*self.gl, &info.base_filename, gl::VERTEX_SHADER, &vs_source, self.requires_null_terminated_shader_source) {
                    Ok(vs_id) => vs_id,
                    Err(err) => return Err(err),
                };

            // Compile the fragment shader
            let fs_source = info.compute_source(self, ShaderKind::Fragment);
            let fs_id =
                match Device::compile_shader(&*self.gl, &info.base_filename, gl::FRAGMENT_SHADER, &fs_source, self.requires_null_terminated_shader_source) {
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

        Ok(())
    }

    pub fn bind_program(&mut self, program: &Program) {
        debug_assert!(self.inside_frame);
        debug_assert!(program.is_initialized());
        #[cfg(debug_assertions)]
        {
            self.shader_is_ready = true;
        }

        if self.bound_program != program.id {
            self.gl.use_program(program.id);
            self.bound_program = program.id;
            self.program_mode_id = UniformLocation(program.u_mode);
        }
    }

    pub fn create_texture(
        &mut self,
        target: TextureTarget,
        format: ImageFormat,
        mut width: i32,
        mut height: i32,
        filter: TextureFilter,
        render_target: Option<RenderTargetInfo>,
        layer_count: i32,
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
            layer_count,
            format,
            filter,
            active_swizzle: Cell::default(),
            fbos: vec![],
            fbos_with_depth: vec![],
            blit_workaround_buffer: None,
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
        let is_array = match texture.target {
            gl::TEXTURE_2D_ARRAY => true,
            gl::TEXTURE_2D | gl::TEXTURE_RECTANGLE | gl::TEXTURE_EXTERNAL_OES => false,
            _ => panic!("BUG: Unexpected texture target!"),
        };
        assert!(is_array || texture.layer_count == 1);

        // Firefox doesn't use mipmaps, but Servo uses them for standalone image
        // textures images larger than 512 pixels. This is the only case where
        // we set the filter to trilinear.
        let mipmap_levels =  if texture.filter == TextureFilter::Trilinear {
            let max_dimension = cmp::max(width, height);
            ((max_dimension) as f64).log2() as gl::GLint + 1
        } else {
            1
        };

        // Use glTexStorage where available, since it avoids allocating
        // unnecessary mipmap storage and generally improves performance with
        // stronger invariants.
        let use_texture_storage = match self.texture_storage_usage {
            TexStorageUsage::Always => true,
            TexStorageUsage::NonBGRA8 => texture.format != ImageFormat::BGRA8,
            TexStorageUsage::Never => false,
        };
        match (use_texture_storage, is_array) {
            (true, true) =>
                self.gl.tex_storage_3d(
                    gl::TEXTURE_2D_ARRAY,
                    mipmap_levels,
                    desc.internal,
                    texture.size.width as gl::GLint,
                    texture.size.height as gl::GLint,
                    texture.layer_count,
                ),
            (true, false) =>
                self.gl.tex_storage_2d(
                    texture.target,
                    mipmap_levels,
                    desc.internal,
                    texture.size.width as gl::GLint,
                    texture.size.height as gl::GLint,
                ),
            (false, true) =>
                self.gl.tex_image_3d(
                    gl::TEXTURE_2D_ARRAY,
                    0,
                    desc.internal as gl::GLint,
                    texture.size.width as gl::GLint,
                    texture.size.height as gl::GLint,
                    texture.layer_count,
                    0,
                    desc.external,
                    desc.pixel_type,
                    None,
                ),
            (false, false) =>
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
                ),
        }

        // Set up FBOs, if required.
        if let Some(rt_info) = render_target {
            self.init_fbos(&mut texture, false);
            if rt_info.has_depth {
                self.init_fbos(&mut texture, true);
            }
        }

        // Set up intermediate buffer for blitting to texture, if required.
        if texture.layer_count > 1 && !self.capabilities.supports_blit_to_texture_array {
            let rbo = RBOId(self.gl.gen_renderbuffers(1)[0]);
            let fbo = FBOId(self.gl.gen_framebuffers(1)[0]);
            self.gl.bind_renderbuffer(gl::RENDERBUFFER, rbo.0);
            self.gl.renderbuffer_storage(
                gl::RENDERBUFFER,
                self.matching_renderbuffer_format(texture.format),
                texture.size.width as _,
                texture.size.height as _
            );

            self.bind_draw_target_impl(fbo);
            self.gl.framebuffer_renderbuffer(
                gl::DRAW_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::RENDERBUFFER,
                rbo.0
            );
            texture.blit_workaround_buffer = Some((rbo, fbo));
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

    /// Copies the contents from one renderable texture to another.
    pub fn blit_renderable_texture(
        &mut self,
        dst: &mut Texture,
        src: &Texture,
    ) {
        debug_assert!(self.inside_frame);
        debug_assert!(dst.size.width >= src.size.width);
        debug_assert!(dst.size.height >= src.size.height);
        debug_assert!(dst.layer_count >= src.layer_count);

        if self.capabilities.supports_copy_image_sub_data {
            assert_ne!(src.id, dst.id,
                    "glCopyImageSubData's behaviour is undefined if src and dst images are identical and the rectangles overlap.");
            unsafe {
                self.gl.copy_image_sub_data(src.id, src.target, 0,
                                            0, 0, 0,
                                            dst.id, dst.target, 0,
                                            0, 0, 0,
                                            src.size.width as _, src.size.height as _, src.layer_count);
            }
        } else {
            let rect = FramebufferIntRect::new(
                FramebufferIntPoint::zero(),
                device_size_as_framebuffer_size(src.get_dimensions()),
            );
            for layer in 0..src.layer_count.min(dst.layer_count) as LayerIndex {
                self.blit_render_target(
                    ReadTarget::from_texture(src, layer),
                    rect,
                    DrawTarget::from_texture(dst, layer, false),
                    rect,
                    TextureFilter::Linear
                );
            }
            self.reset_draw_target();
            self.reset_read_target();
        }
    }

    /// Notifies the device that the contents of a render target are no longer
    /// needed.
    ///
    /// FIXME(bholley): We could/should invalidate the depth targets earlier
    /// than the color targets, i.e. immediately after each pass.
    pub fn invalidate_render_target(&mut self, texture: &Texture) {
        let (fbos, attachments) = if texture.supports_depth() {
            (&texture.fbos_with_depth,
             &[gl::COLOR_ATTACHMENT0, gl::DEPTH_ATTACHMENT] as &[gl::GLenum])
        } else {
            (&texture.fbos, &[gl::COLOR_ATTACHMENT0] as &[gl::GLenum])
        };

        let original_bound_fbo = self.bound_draw_fbo;
        for fbo_id in fbos.iter() {
            // Note: The invalidate extension may not be supported, in which
            // case this is a no-op. That's ok though, because it's just a
            // hint.
            self.bind_external_draw_target(*fbo_id);
            self.gl.invalidate_framebuffer(gl::FRAMEBUFFER, attachments);
        }
        self.bind_external_draw_target(original_bound_fbo);
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
        let (fbos, depth_rb) = if with_depth {
            let depth_target = self.acquire_depth_target(texture.get_dimensions());
            (&mut texture.fbos_with_depth, Some(depth_target))
        } else {
            (&mut texture.fbos, None)
        };

        // Generate the FBOs.
        assert!(fbos.is_empty());
        fbos.extend(self.gl.gen_framebuffers(texture.layer_count).into_iter().map(FBOId));

        // Bind the FBOs.
        let original_bound_fbo = self.bound_draw_fbo;
        for (fbo_index, &fbo_id) in fbos.iter().enumerate() {
            self.bind_external_draw_target(fbo_id);
            match texture.target {
                gl::TEXTURE_2D_ARRAY => {
                    self.gl.framebuffer_texture_layer(
                        gl::DRAW_FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        texture.id,
                        0,
                        fbo_index as _,
                    )
                }
                _ => {
                    assert_eq!(fbo_index, 0);
                    self.gl.framebuffer_texture_2d(
                        gl::DRAW_FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        texture.target,
                        texture.id,
                        0,
                    )
                }
            }

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
        }
        self.bind_external_draw_target(original_bound_fbo);
    }

    fn deinit_fbos(&mut self, fbos: &mut Vec<FBOId>) {
        if !fbos.is_empty() {
            let fbo_ids: SmallVec<[gl::GLuint; 8]> = fbos
                .drain(..)
                .map(|FBOId(fbo_id)| fbo_id)
                .collect();
            self.gl.delete_framebuffers(&fbo_ids[..]);
        }
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

        self.gl.blit_framebuffer(
            src_rect.origin.x,
            src_rect.origin.y,
            src_rect.origin.x + src_rect.size.width,
            src_rect.origin.y + src_rect.size.height,
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

        match dest_target {
            DrawTarget::Texture { layer, blit_workaround_buffer, dimensions, id, target, .. } if layer != 0 &&
                !self.capabilities.supports_blit_to_texture_array =>
            {
                // This should have been initialized in create_texture().
                let (_rbo, fbo) = blit_workaround_buffer.expect("Blit workaround buffer has not been initialized.");

                // Blit from read target to intermediate buffer.
                self.bind_draw_target_impl(fbo);
                self.blit_render_target_impl(
                    src_rect,
                    dest_rect,
                    filter
                );

                // dest_rect may be inverted, so min_x/y() might actually be the
                // bottom-right, max_x/y() might actually be the top-left,
                // and width/height might be negative. See servo/euclid#321.
                // Calculate the non-inverted rect here.
                let dest_bounds = DeviceIntRect::new(
                    DeviceIntPoint::new(
                        dest_rect.min_x().min(dest_rect.max_x()),
                        dest_rect.min_y().min(dest_rect.max_y()),
                    ),
                    DeviceIntSize::new(
                        dest_rect.size.width.abs(),
                        dest_rect.size.height.abs(),
                    ),
                ).intersection(&dimensions.into()).unwrap_or_else(DeviceIntRect::zero);

                self.bind_read_target_impl(fbo);
                self.bind_texture_impl(
                    DEFAULT_TEXTURE,
                    id,
                    target,
                    None, // not depending on swizzle
                );

                // Copy from intermediate buffer to the texture layer.
                self.gl.copy_tex_sub_image_3d(
                    target, 0,
                    dest_bounds.origin.x, dest_bounds.origin.y,
                    layer as _,
                    dest_bounds.origin.x, dest_bounds.origin.y,
                    dest_bounds.size.width, dest_bounds.size.height,
                );

            }
            _ => {
                self.bind_draw_target(dest_target);

                self.blit_render_target_impl(src_rect, dest_rect, filter);
            }
        }
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
        self.deinit_fbos(&mut texture.fbos);
        self.deinit_fbos(&mut texture.fbos_with_depth);
        if had_depth {
            self.release_depth_target(texture.get_dimensions());
        }
        if let Some((rbo, fbo)) = texture.blit_workaround_buffer {
            self.gl.delete_framebuffers(&[fbo.0]);
            self.gl.delete_renderbuffers(&[rbo.0]);
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

        let dst_stride = round_up_to_multiple(width_bytes, self.optimal_pbo_stride.num_bytes(format));

        // The size of the chunk should only need to be (height - 1) * dst_stride + width_bytes,
        // however, the android emulator will error unless it is height * dst_stride.
        // See bug 1587047 for details.
        // Using the full final row also ensures that the offset of the next chunk is
        // optimally aligned.
        let dst_size = dst_stride * size.height as usize;

        (dst_size, dst_stride)
    }

    /// (Re)allocates and maps a PBO, returning a `PixelBuffer` if successful.
    /// The contents can be written to using the `mapping` field.
    /// The buffer must be bound to `GL_PIXEL_UNPACK_BUFFER` before calling this function,
    /// and must be unmapped using `glUnmapBuffer` prior to uploading the contents to a texture.
    fn create_upload_buffer<'a>(&mut self, hint: VertexUsageHint, size: usize) -> Result<PixelBuffer<'a>, ()> {
        self.gl.buffer_data_untyped(
            gl::PIXEL_UNPACK_BUFFER,
            size as _,
            ptr::null(),
            hint.to_gl(),
        );
        let ptr = self.gl.map_buffer_range(
            gl::PIXEL_UNPACK_BUFFER,
            0,
            size as _,
            gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
        );

        if ptr != ptr::null_mut() {
            let mapping = unsafe {
                slice::from_raw_parts_mut(ptr as *mut _, size)
            };
            Ok(PixelBuffer::new(size, mapping))
        } else {
            error!("Failed to map PBO of size {} bytes", size);
            Err(())
        }
    }

    /// Returns a `TextureUploader` which can be used to upload texture data to `texture`.
    /// The total size in bytes is specified by `upload_size`, and must be greater than zero
    /// and at least as large as the sum of the sizes returned from
    /// `required_upload_size_and_stride()` for each subsequent call to `TextureUploader.upload()`.
    pub fn upload_texture<'a, T>(
        &'a mut self,
        texture: &'a Texture,
        pbo: &PBO,
        upload_size: usize,
    ) -> TextureUploader<'a, T> {
        debug_assert!(self.inside_frame);
        assert_ne!(upload_size, 0, "Must specify valid upload size");

        self.bind_texture(DEFAULT_TEXTURE, texture, Swizzle::default());

        let uploader_type = match self.upload_method {
            UploadMethod::Immediate => TextureUploaderType::Immediate,
            UploadMethod::PixelBuffer(hint) => {
                self.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, pbo.id);
                if self.capabilities.supports_nonzero_pbo_offsets {
                    match self.create_upload_buffer(hint, upload_size) {
                        Ok(buffer) => TextureUploaderType::MutliUseBuffer(buffer),
                        Err(_) => {
                            // If allocating the buffer failed, fall back to immediate uploads
                            self.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);
                            TextureUploaderType::Immediate
                        }
                    }
                } else {
                    // If we cannot upload from non-zero offsets, then we must
                    // reallocate a new buffer for each upload.
                    TextureUploaderType::SingleUseBuffers(hint)
                }
            },
        };

        TextureUploader {
            target: UploadTarget {
                device: self,
                texture,
            },
            uploader_type,
            marker: PhantomData,
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
        match texture.target {
            gl::TEXTURE_2D | gl::TEXTURE_RECTANGLE | gl::TEXTURE_EXTERNAL_OES =>
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
                ),
            gl::TEXTURE_2D_ARRAY =>
                self.gl.tex_sub_image_3d(
                    texture.target,
                    0,
                    0,
                    0,
                    0,
                    texture.size.width as gl::GLint,
                    texture.size.height as gl::GLint,
                    texture.layer_count as gl::GLint,
                    desc.external,
                    desc.pixel_type,
                    texels_to_u8_slice(pixels),
                ),
            _ => panic!("BUG: Unexpected texture target!"),
        }
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
    fn attach_read_texture_raw(
        &mut self, texture_id: gl::GLuint, target: gl::GLuint, layer_id: i32
    ) {
        match target {
            gl::TEXTURE_2D_ARRAY => {
                self.gl.framebuffer_texture_layer(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    texture_id,
                    0,
                    layer_id,
                )
            }
            _ => {
                assert_eq!(layer_id, 0);
                self.gl.framebuffer_texture_2d(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    target,
                    texture_id,
                    0,
                )
            }
        }
    }

    pub fn attach_read_texture_external(
        &mut self, texture_id: gl::GLuint, target: TextureTarget, layer_id: i32
    ) {
        self.attach_read_texture_raw(texture_id, get_gl_target(target), layer_id)
    }

    pub fn attach_read_texture(&mut self, texture: &Texture, layer_id: i32) {
        self.attach_read_texture_raw(texture.id, texture.target, layer_id)
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
        ibo_id: IBOId,
        owns_vertices_and_indices: bool,
    ) -> VAO {
        let instance_stride = descriptor.instance_stride() as usize;
        let vao_id = self.gl.gen_vertex_arrays(1)[0];

        self.bind_vao_impl(vao_id);

        descriptor.bind(self.gl(), main_vbo_id, instance_vbo_id);
        ibo_id.bind(self.gl()); // force it to be a part of VAO

        VAO {
            id: vao_id,
            ibo_id,
            main_vbo_id,
            instance_vbo_id,
            instance_stride,
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

    pub fn create_vao(&mut self, descriptor: &VertexDescriptor) -> VAO {
        debug_assert!(self.inside_frame);

        let buffer_ids = self.gl.gen_buffers(3);
        let ibo_id = IBOId(buffer_ids[0]);
        let main_vbo_id = VBOId(buffer_ids[1]);
        let intance_vbo_id = VBOId(buffer_ids[2]);

        self.create_vao_with_vbos(descriptor, main_vbo_id, intance_vbo_id, ibo_id, true)
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

    pub fn update_vao_instances<V>(
        &mut self,
        vao: &VAO,
        instances: &[V],
        usage_hint: VertexUsageHint,
    ) {
        debug_assert_eq!(self.bound_vao, vao.id);
        debug_assert_eq!(vao.instance_stride as usize, mem::size_of::<V>());

        self.update_vbo_data(vao.instance_vbo_id, instances, usage_hint)
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

        self.gl.draw_arrays(gl::POINTS, first_vertex, vertex_count);
    }

    pub fn draw_nonindexed_lines(&mut self, first_vertex: i32, vertex_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

        self.gl.draw_arrays(gl::LINES, first_vertex, vertex_count);
    }

    pub fn draw_indexed_triangles_instanced_u16(&mut self, index_count: i32, instance_count: i32) {
        debug_assert!(self.inside_frame);
        #[cfg(debug_assertions)]
        debug_assert!(self.shader_is_ready);

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
            (gl::ONE, gl::ONE),
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

    /// Enable the pixel local storage functionality. Caller must
    /// have already confirmed the device supports this.
    pub fn enable_pixel_local_storage(&mut self, enable: bool) {
        debug_assert!(self.capabilities.supports_pixel_local_storage);

        if enable {
            self.gl.enable(gl::SHADER_PIXEL_LOCAL_STORAGE_EXT);
        } else {
            self.gl.disable(gl::SHADER_PIXEL_LOCAL_STORAGE_EXT);
        }
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
                    pixel_type: gl::UNSIGNED_BYTE,
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

    /// Returns a GL format matching an ImageFormat suitable for a renderbuffer.
    fn matching_renderbuffer_format(&self, format: ImageFormat) -> gl::GLenum {
        match format {
            ImageFormat::R8 => gl::R8,
            ImageFormat::R16 => gl::R16UI,
            ImageFormat::BGRA8 => panic!("Unable to render to BGRA format!"),
            ImageFormat::RGBAF32 => gl::RGBA32F,
            ImageFormat::RG8 => gl::RG8,
            ImageFormat::RG16 => gl::RG16,
            ImageFormat::RGBAI32 => gl::RGBA32I,
            ImageFormat::RGBA8 => gl::RGBA8,
        }
    }

    /// Generates a memory report for the resources managed by the device layer.
    pub fn report_memory(&self) -> MemoryReport {
        let mut report = MemoryReport::default();
        for dim in self.depth_targets.keys() {
            report.depth_target_textures += depth_target_size_in_bytes(dim);
        }
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

struct UploadChunk {
    rect: DeviceIntRect,
    layer_index: i32,
    stride: Option<i32>,
    offset: usize,
    format_override: Option<ImageFormat>,
}

struct PixelBuffer<'a> {
    size_allocated: usize,
    size_used: usize,
    // small vector avoids heap allocation for a single chunk
    chunks: SmallVec<[UploadChunk; 1]>,
    mapping: &'a mut [mem::MaybeUninit<u8>],
}

impl<'a> PixelBuffer<'a> {
    fn new(
        size_allocated: usize,
        mapping: &'a mut [mem::MaybeUninit<u8>],
    ) -> Self {
        PixelBuffer {
            size_allocated,
            size_used: 0,
            chunks: SmallVec::new(),
            mapping,
        }
    }

    fn flush_chunks(&mut self, target: &mut UploadTarget) {
        for chunk in self.chunks.drain(..) {
            target.update_impl(chunk);
        }
        self.size_used = 0;
    }
}

impl<'a> Drop for PixelBuffer<'a> {
    fn drop(&mut self) {
        assert_eq!(self.chunks.len(), 0, "PixelBuffer must be flushed before dropping.");
    }
}

struct UploadTarget<'a> {
    device: &'a mut Device,
    texture: &'a Texture,
}

enum TextureUploaderType<'a> {
    Immediate,
    SingleUseBuffers(VertexUsageHint),
    MutliUseBuffer(PixelBuffer<'a>)
}

pub struct TextureUploader<'a, T> {
    target: UploadTarget<'a>,
    uploader_type: TextureUploaderType<'a>,
    marker: PhantomData<T>,
}

impl<'a, T> Drop for TextureUploader<'a, T> {
    fn drop(&mut self) {
        match self.uploader_type {
            TextureUploaderType::MutliUseBuffer(ref mut buffer) => {
                self.target.device.gl.unmap_buffer(gl::PIXEL_UNPACK_BUFFER);
                buffer.flush_chunks(&mut self.target);
                self.target.device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);
            }
            TextureUploaderType::SingleUseBuffers(_) => {
                self.target.device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);
            }
            TextureUploaderType::Immediate => {}
        }
    }
}

impl<'a, T> TextureUploader<'a, T> {
    pub fn upload(
        &mut self,
        mut rect: DeviceIntRect,
        layer_index: i32,
        stride: Option<i32>,
        format_override: Option<ImageFormat>,
        data: *const T,
        len: usize,
    ) -> usize {
        // Textures dimensions may have been clamped by the hardware. Crop the
        // upload region to match.
        let cropped = rect.intersection(
            &DeviceIntRect::new(DeviceIntPoint::zero(), self.target.texture.get_dimensions())
        );
        if cfg!(debug_assertions) && cropped.map_or(true, |r| r != rect) {
            warn!("Cropping texture upload {:?} to {:?}", rect, cropped);
        }
        rect = match cropped {
            None => return 0,
            Some(r) => r,
        };

        let bytes_pp = self.target.texture.format.bytes_per_pixel() as usize;
        let width_bytes = rect.size.width as usize * bytes_pp;

        let src_stride = stride.map_or(width_bytes, |stride| {
            assert!(stride >= 0);
            stride as usize
        });
        let src_size = (rect.size.height as usize - 1) * src_stride + width_bytes;
        assert!(src_size <= len * mem::size_of::<T>());

        // for optimal PBO texture uploads the offset and stride of the data in
        // the buffer may have to be a multiple of a certain value.
        let (dst_size, dst_stride) = self.target.device.required_upload_size_and_stride(
            rect.size,
            self.target.texture.format,
        );

        // Choose the buffer to use, if any, allocating a new single-use buffer if required.
        let mut single_use_buffer = None;
        let mut buffer = match self.uploader_type {
            TextureUploaderType::MutliUseBuffer(ref mut buffer) => Some(buffer),
            TextureUploaderType::SingleUseBuffers(hint) => {
                match self.target.device.create_upload_buffer(hint, dst_size) {
                    Ok(buffer) => {
                        single_use_buffer = Some(buffer);
                        single_use_buffer.as_mut()
                    }
                    Err(_) => {
                        // If allocating the buffer failed, fall back to immediate uploads
                        self.target.device.gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, 0);
                        self.uploader_type = TextureUploaderType::Immediate;
                        None
                    }
                }
            }
            TextureUploaderType::Immediate => None,
        };

        match buffer {
            Some(ref mut buffer) => {
                if !self.target.device.capabilities.supports_nonzero_pbo_offsets {
                    assert_eq!(buffer.size_used, 0, "PBO uploads from non-zero offset are not supported.");
                }
                assert!(buffer.size_used + dst_size <= buffer.size_allocated, "PixelBuffer is too small");

                unsafe {
                    let src: &[mem::MaybeUninit<u8>] = slice::from_raw_parts(data as *const _, src_size);

                    if src_stride == dst_stride {
                        // the stride is already optimal, so simply copy
                        // the data as-is in to the buffer
                        let dst_start = buffer.size_used;
                        let dst_end = dst_start + src_size;

                        buffer.mapping[dst_start..dst_end].copy_from_slice(src);
                    } else {
                        // copy the data line-by-line in to the buffer so
                        // that it has an optimal stride
                        for y in 0..rect.size.height as usize {
                            let src_start = y * src_stride;
                            let src_end = src_start + width_bytes;
                            let dst_start = buffer.size_used + y * dst_stride;
                            let dst_end = dst_start + width_bytes;

                            buffer.mapping[dst_start..dst_end].copy_from_slice(&src[src_start..src_end])
                        }
                    }
                }

                buffer.chunks.push(UploadChunk {
                    rect,
                    layer_index,
                    stride: Some(dst_stride as i32),
                    offset: buffer.size_used,
                    format_override,
                });
                buffer.size_used += dst_size;
            }
            None => {
                if cfg!(debug_assertions) {
                    let mut bound_buffer = [0];
                    unsafe {
                        self.target.device.gl.get_integer_v(gl::PIXEL_UNPACK_BUFFER_BINDING, &mut bound_buffer);
                    }
                    assert_eq!(bound_buffer[0], 0, "GL_PIXEL_UNPACK_BUFFER must not be bound for immediate uploads.");
                }

                self.target.update_impl(UploadChunk {
                    rect,
                    layer_index,
                    stride,
                    offset: data as _,
                    format_override,
                });
            }
        }

        // Flush the buffer if it is for single-use.
        if let Some(ref mut buffer) = single_use_buffer {
            self.target.device.gl.unmap_buffer(gl::PIXEL_UNPACK_BUFFER);
            buffer.flush_chunks(&mut self.target);
        }

        dst_size
    }
}

impl<'a> UploadTarget<'a> {
    fn update_impl(&mut self, chunk: UploadChunk) {
        let format = chunk.format_override.unwrap_or(self.texture.format);
        let (gl_format, bpp, data_type) = match format {
            ImageFormat::R8 => (gl::RED, 1, gl::UNSIGNED_BYTE),
            ImageFormat::R16 => (gl::RED, 2, gl::UNSIGNED_SHORT),
            ImageFormat::BGRA8 => (self.device.bgra_formats.external, 4, gl::UNSIGNED_BYTE),
            ImageFormat::RGBA8 => (gl::RGBA, 4, gl::UNSIGNED_BYTE),
            ImageFormat::RG8 => (gl::RG, 2, gl::UNSIGNED_BYTE),
            ImageFormat::RG16 => (gl::RG, 4, gl::UNSIGNED_SHORT),
            ImageFormat::RGBAF32 => (gl::RGBA, 16, gl::FLOAT),
            ImageFormat::RGBAI32 => (gl::RGBA_INTEGER, 16, gl::INT),
        };

        let row_length = match chunk.stride {
            Some(value) => value / bpp,
            None => self.texture.size.width,
        };

        if chunk.stride.is_some() {
            self.device.gl.pixel_store_i(
                gl::UNPACK_ROW_LENGTH,
                row_length as _,
            );
        }

        let pos = chunk.rect.origin;
        let size = chunk.rect.size;

        match self.texture.target {
            gl::TEXTURE_2D_ARRAY => {
                self.device.gl.tex_sub_image_3d_pbo(
                    self.texture.target,
                    0,
                    pos.x as _,
                    pos.y as _,
                    chunk.layer_index,
                    size.width as _,
                    size.height as _,
                    1,
                    gl_format,
                    data_type,
                    chunk.offset,
                );
            }
            gl::TEXTURE_2D | gl::TEXTURE_RECTANGLE | gl::TEXTURE_EXTERNAL_OES => {
                self.device.gl.tex_sub_image_2d_pbo(
                    self.texture.target,
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
        if self.texture.filter == TextureFilter::Trilinear {
            self.device.gl.generate_mipmap(self.texture.target);
        }

        // Reset row length to 0, otherwise the stride would apply to all texture uploads.
        if chunk.stride.is_some() {
            self.device.gl.pixel_store_i(gl::UNPACK_ROW_LENGTH, 0 as _);
        }
    }
}

fn texels_to_u8_slice<T: Texel>(texels: &[T]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(texels.as_ptr() as *const u8, texels.len() * mem::size_of::<T>())
    }
}
