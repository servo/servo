/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

use euclid::{size2, Rect, num::Zero};
use peek_poke::PeekPoke;
use std::ops::{Add, Sub};
use std::sync::Arc;
// local imports
use crate::{IdNamespace, TileSize};
use crate::display_item::ImageRendering;
use crate::font::{FontInstanceKey, FontInstanceData, FontKey, FontTemplate};
use crate::units::*;

/// The default tile size for blob images and regular images larger than
/// the maximum texture size.
pub const DEFAULT_TILE_SIZE: TileSize = 512;

/// An opaque identifier describing an image registered with WebRender.
/// This is used as a handle to reference images, and is used as the
/// hash map key for the actual image storage in the `ResourceCache`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct ImageKey(pub IdNamespace, pub u32);

impl Default for ImageKey {
    fn default() -> Self {
        ImageKey::DUMMY
    }
}

impl ImageKey {
    /// Placeholder Image key, used to represent None.
    pub const DUMMY: Self = ImageKey(IdNamespace(0), 0);

    /// Mints a new ImageKey. The given ID must be unique.
    pub fn new(namespace: IdNamespace, key: u32) -> Self {
        ImageKey(namespace, key)
    }
}

/// An opaque identifier describing a blob image registered with WebRender.
/// This is used as a handle to reference blob images, and can be used as an
/// image in display items.
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BlobImageKey(pub ImageKey);

impl BlobImageKey {
    /// Interpret this blob image as an image for a display item.
    pub fn as_image(self) -> ImageKey {
        self.0
    }
}

/// An arbitrary identifier for an external image provided by the
/// application. It must be a unique identifier for each external
/// image.
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ExternalImageId(pub u64);

/// The source for an external image.
pub enum ExternalImageSource<'a> {
    /// A raw pixel buffer.
    RawData(&'a [u8]),
    /// A gl::GLuint texture handle.
    NativeTexture(u32),
    /// An invalid source.
    Invalid,
}

/// The data that an external client should provide about
/// an external image. For instance, if providing video frames,
/// the application could call wr.render() whenever a new
/// video frame is ready. Note that the UV coords are supplied
/// in texel-space!
pub struct ExternalImage<'a> {
    /// UV coordinates for the image.
    pub uv: TexelRect,
    /// The source for this image's contents.
    pub source: ExternalImageSource<'a>,
}

/// The interfaces that an application can implement to support providing
/// external image buffers.
/// When the application passes an external image to WR, it should keep that
/// external image life time. People could check the epoch id in RenderNotifier
/// at the client side to make sure that the external image is not used by WR.
/// Then, do the clean up for that external image.
pub trait ExternalImageHandler {
    /// Lock the external image. Then, WR could start to read the image content.
    /// The WR client should not change the image content until the unlock()
    /// call. Provide ImageRendering for NativeTexture external images.
    fn lock(&mut self, key: ExternalImageId, channel_index: u8, rendering: ImageRendering) -> ExternalImage;
    /// Unlock the external image. WR should not read the image content
    /// after this call.
    fn unlock(&mut self, key: ExternalImageId, channel_index: u8);
}

/// Specifies the type of texture target in driver terms.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ImageBufferKind {
    /// Standard texture. This maps to GL_TEXTURE_2D in OpenGL.
    Texture2D = 0,
    /// Rectangle texture. This maps to GL_TEXTURE_RECTANGLE in OpenGL. This
    /// is similar to a standard texture, with a few subtle differences
    /// (no mipmaps, non-power-of-two dimensions, different coordinate space)
    /// that make it useful for representing the kinds of textures we use
    /// in WebRender. See https://www.khronos.org/opengl/wiki/Rectangle_Texture
    /// for background on Rectangle textures.
    TextureRect = 1,
    /// External texture. This maps to GL_TEXTURE_EXTERNAL_OES in OpenGL, which
    /// is an extension. This is used for image formats that OpenGL doesn't
    /// understand, particularly YUV. See
    /// https://www.khronos.org/registry/OpenGL/extensions/OES/OES_EGL_image_external.txt
    TextureExternal = 2,
}

/// Storage format identifier for externally-managed images.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ExternalImageType {
    /// The image is texture-backed.
    TextureHandle(ImageBufferKind),
    /// The image is heap-allocated by the embedding.
    Buffer,
}

/// Descriptor for external image resources. See `ImageData`.
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExternalImageData {
    /// The identifier of this external image, provided by the embedding.
    pub id: ExternalImageId,
    /// For multi-plane images (i.e. YUV), indicates the plane of the
    /// original image that this struct represents. 0 for single-plane images.
    pub channel_index: u8,
    /// Storage format identifier.
    pub image_type: ExternalImageType,
}

/// Specifies the format of a series of pixels, in driver terms.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ImageFormat {
    /// One-channel, byte storage. The "red" doesn't map to the color
    /// red per se, and is just the way that OpenGL has historically referred
    /// to single-channel buffers.
    R8 = 1,
    /// One-channel, short storage
    R16 = 2,
    /// Four channels, byte storage.
    BGRA8 = 3,
    /// Four channels, float storage.
    RGBAF32 = 4,
    /// Two-channels, byte storage. Similar to `R8`, this just means
    /// "two channels" rather than "red and green".
    RG8 = 5,
    /// Two-channels, byte storage. Similar to `R16`, this just means
    /// "two channels" rather than "red and green".
    RG16 = 6,

    /// Four channels, signed integer storage.
    RGBAI32 = 7,
    /// Four channels, byte storage.
    RGBA8 = 8,
}

impl ImageFormat {
    /// Returns the number of bytes per pixel for the given format.
    pub fn bytes_per_pixel(self) -> i32 {
        match self {
            ImageFormat::R8 => 1,
            ImageFormat::R16 => 2,
            ImageFormat::BGRA8 => 4,
            ImageFormat::RGBAF32 => 16,
            ImageFormat::RG8 => 2,
            ImageFormat::RG16 => 4,
            ImageFormat::RGBAI32 => 16,
            ImageFormat::RGBA8 => 4,
        }
    }
}

/// Specifies the color depth of an image. Currently only used for YUV images.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum ColorDepth {
    /// 8 bits image (most common)
    Color8,
    /// 10 bits image
    Color10,
    /// 12 bits image
    Color12,
    /// 16 bits image
    Color16,
}

impl Default for ColorDepth {
    fn default() -> Self {
        ColorDepth::Color8
    }
}

impl ColorDepth {
    /// Return the numerical bit depth value for the type.
    pub fn bit_depth(self) -> u32 {
        match self {
            ColorDepth::Color8 => 8,
            ColorDepth::Color10 => 10,
            ColorDepth::Color12 => 12,
            ColorDepth::Color16 => 16,
        }
    }
    /// 10 and 12 bits images are encoded using 16 bits integer, we need to
    /// rescale the 10 or 12 bits value to extend to 16 bits.
    pub fn rescaling_factor(self) -> f32 {
        match self {
            ColorDepth::Color8 => 1.0,
            ColorDepth::Color10 => 64.0,
            ColorDepth::Color12 => 16.0,
            ColorDepth::Color16 => 1.0,
        }
    }
}

bitflags! {
    /// Various flags that are part of an image descriptor.
    #[derive(Deserialize, Serialize)]
    pub struct ImageDescriptorFlags: u32 {
        /// Whether this image is opaque, or has an alpha channel. Avoiding blending
        /// for opaque surfaces is an important optimization.
        const IS_OPAQUE = 1;
        /// Whether to allow the driver to automatically generate mipmaps. If images
        /// are already downscaled appropriately, mipmap generation can be wasted
        /// work, and cause performance problems on some cards/drivers.
        ///
        /// See https://github.com/servo/webrender/pull/2555/
        const ALLOW_MIPMAPS = 2;
    }
}

/// Metadata (but not storage) describing an image In WebRender.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ImageDescriptor {
    /// Format of the image data.
    pub format: ImageFormat,
    /// Width and length of the image data, in pixels.
    pub size: DeviceIntSize,
    /// The number of bytes from the start of one row to the next. If non-None,
    /// `compute_stride` will return this value, otherwise it returns
    /// `width * bpp`. Different source of images have different alignment
    /// constraints for rows, so the stride isn't always equal to width * bpp.
    pub stride: Option<i32>,
    /// Offset in bytes of the first pixel of this image in its backing buffer.
    /// This is used for tiling, wherein WebRender extracts chunks of input images
    /// in order to cache, manipulate, and render them individually. This offset
    /// tells the texture upload machinery where to find the bytes to upload for
    /// this tile. Non-tiled images generally set this to zero.
    pub offset: i32,
    /// Various bool flags related to this descriptor.
    pub flags: ImageDescriptorFlags,
}

impl ImageDescriptor {
    /// Mints a new ImageDescriptor.
    pub fn new(
        width: i32,
        height: i32,
        format: ImageFormat,
        flags: ImageDescriptorFlags,
    ) -> Self {
        ImageDescriptor {
            size: size2(width, height),
            format,
            stride: None,
            offset: 0,
            flags,
        }
    }

    /// Returns the stride, either via an explicit stride stashed on the object
    /// or by the default computation.
    pub fn compute_stride(&self) -> i32 {
        self.stride.unwrap_or(self.size.width * self.format.bytes_per_pixel())
    }

    /// Computes the total size of the image, in bytes.
    pub fn compute_total_size(&self) -> i32 {
        self.compute_stride() * self.size.height
    }

    /// Computes the bounding rectangle for the image, rooted at (0, 0).
    pub fn full_rect(&self) -> DeviceIntRect {
        DeviceIntRect::new(
            DeviceIntPoint::zero(),
            self.size,
        )
    }

    /// Returns true if this descriptor is opaque
    pub fn is_opaque(&self) -> bool {
        self.flags.contains(ImageDescriptorFlags::IS_OPAQUE)
    }

    /// Returns true if this descriptor allows mipmaps
    pub fn allow_mipmaps(&self) -> bool {
        self.flags.contains(ImageDescriptorFlags::ALLOW_MIPMAPS)
    }
}

/// Represents the backing store of an arbitrary series of pixels for display by
/// WebRender. This storage can take several forms.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ImageData {
    /// A simple series of bytes, provided by the embedding and owned by WebRender.
    /// The format is stored out-of-band, currently in ImageDescriptor.
    Raw(#[serde(with = "serde_image_data_raw")] Arc<Vec<u8>>),
    /// An image owned by the embedding, and referenced by WebRender. This may
    /// take the form of a texture or a heap-allocated buffer.
    External(ExternalImageData),
}

mod serde_image_data_raw {
    use serde::{Deserializer, Serializer};
    use serde_bytes;
    use std::sync::Arc;

    pub fn serialize<S: Serializer>(bytes: &Arc<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error> {
        serde_bytes::serialize(bytes.as_slice(), serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Arc<Vec<u8>>, D::Error> {
        serde_bytes::deserialize(deserializer).map(Arc::new)
    }
}

impl ImageData {
    /// Mints a new raw ImageData, taking ownership of the bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        ImageData::Raw(Arc::new(bytes))
    }

    /// Mints a new raw ImageData from Arc-ed bytes.
    pub fn new_shared(bytes: Arc<Vec<u8>>) -> Self {
        ImageData::Raw(bytes)
    }
}

/// The resources exposed by the resource cache available for use by the blob rasterizer.
pub trait BlobImageResources {
    /// Returns the `FontTemplate` for the given key.
    fn get_font_data(&self, key: FontKey) -> &FontTemplate;
    /// Returns the `FontInstanceData` for the given key, if found.
    fn get_font_instance_data(&self, key: FontInstanceKey) -> Option<FontInstanceData>;
}

/// A handler on the render backend that can create rasterizer objects which will
/// be sent to the scene builder thread to execute the rasterization.
///
/// The handler is responsible for collecting resources, managing/updating blob commands
/// and creating the rasterizer objects, but isn't expected to do any rasterization itself.
pub trait BlobImageHandler: Send {
    /// Creates a snapshot of the current state of blob images in the handler.
    fn create_blob_rasterizer(&mut self) -> Box<dyn AsyncBlobImageRasterizer>;

    /// Creates an empty blob handler of the same type.
    ///
    /// This is used to allow creating new API endpoints with blob handlers installed on them.
    fn create_similar(&self) -> Box<dyn BlobImageHandler>;

    /// A hook to let the blob image handler update any state related to resources that
    /// are not bundled in the blob recording itself.
    fn prepare_resources(
        &mut self,
        services: &dyn BlobImageResources,
        requests: &[BlobImageParams],
    );

    /// Register a blob image.
    fn add(&mut self, key: BlobImageKey, data: Arc<BlobImageData>, visible_rect: &DeviceIntRect,
           tile_size: TileSize);

    /// Update an already registered blob image.
    fn update(&mut self, key: BlobImageKey, data: Arc<BlobImageData>, visible_rect: &DeviceIntRect,
              dirty_rect: &BlobDirtyRect);

    /// Delete an already registered blob image.
    fn delete(&mut self, key: BlobImageKey);

    /// A hook to let the handler clean up any state related to a font which the resource
    /// cache is about to delete.
    fn delete_font(&mut self, key: FontKey);

    /// A hook to let the handler clean up any state related to a font instance which the
    /// resource cache is about to delete.
    fn delete_font_instance(&mut self, key: FontInstanceKey);

    /// A hook to let the handler clean up any state related a given namespace before the
    /// resource cache deletes them.
    fn clear_namespace(&mut self, namespace: IdNamespace);

    /// Whether to allow rendering blobs on multiple threads.
    fn enable_multithreading(&mut self, enable: bool);
}

/// A group of rasterization requests to execute synchronously on the scene builder thread.
pub trait AsyncBlobImageRasterizer : Send {
    /// Rasterize the requests.
    ///
    /// Gecko uses te priority hint to schedule work in a way that minimizes the risk
    /// of high priority work being blocked by (or enqued behind) low priority work.
    fn rasterize(
        &mut self,
        requests: &[BlobImageParams],
        low_priority: bool
    ) -> Vec<(BlobImageRequest, BlobImageResult)>;
}


/// Input parameters for the BlobImageRasterizer.
#[derive(Copy, Clone, Debug)]
pub struct BlobImageParams {
    /// A key that identifies the blob image rasterization request.
    pub request: BlobImageRequest,
    /// Description of the format of the blob's output image.
    pub descriptor: BlobImageDescriptor,
    /// An optional sub-rectangle of the image to avoid re-rasterizing
    /// the entire image when only a portion is updated.
    ///
    /// If set to None the entire image is rasterized.
    pub dirty_rect: BlobDirtyRect,
}

/// The possible states of a Dirty rect.
///
/// This exists because people kept getting confused with `Option<Rect>`.
#[derive(Debug, Serialize, Deserialize)]
pub enum DirtyRect<T: Copy, U> {
    /// Everything is Dirty, equivalent to Partial(image_bounds)
    All,
    /// Some specific amount is dirty
    Partial(Rect<T, U>)
}

impl<T, U> DirtyRect<T, U>
where
    T: Copy + Clone
        + PartialOrd + PartialEq
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Zero
{
    /// Creates an empty DirtyRect (indicating nothing is invalid)
    pub fn empty() -> Self {
        DirtyRect::Partial(Rect::zero())
    }

    /// Returns whether the dirty rect is empty
    pub fn is_empty(&self) -> bool {
        match self {
            DirtyRect::All => false,
            DirtyRect::Partial(rect) => rect.is_empty(),
        }
    }

    /// Replaces self with the empty rect and returns the old value.
    pub fn replace_with_empty(&mut self) -> Self {
        ::std::mem::replace(self, DirtyRect::empty())
    }

    /// Maps over the contents of Partial.
    pub fn map<F>(self, func: F) -> Self
        where F: FnOnce(Rect<T, U>) -> Rect<T, U>,
    {
        use crate::DirtyRect::*;

        match self {
            All        => All,
            Partial(rect) => Partial(func(rect)),
        }
    }

    /// Unions the dirty rects.
    pub fn union(&self, other: &Self) -> Self {
        use crate::DirtyRect::*;

        match (*self, *other) {
            (All, _) | (_, All)        => All,
            (Partial(rect1), Partial(rect2)) => Partial(rect1.union(&rect2)),
        }
    }

    /// Intersects the dirty rects.
    pub fn intersection(&self, other: &Self) -> Self {
        use crate::DirtyRect::*;

        match (*self, *other) {
            (All, rect) | (rect, All)  => rect,
            (Partial(rect1), Partial(rect2)) => {
                Partial(rect1.intersection(&rect2).unwrap_or_else(Rect::zero))
            }
        }
    }

    /// Converts the dirty rect into a subrect of the given one via intersection.
    pub fn to_subrect_of(&self, rect: &Rect<T, U>) -> Rect<T, U> {
        use crate::DirtyRect::*;

        match *self {
            All => *rect,
            Partial(dirty_rect) => {
                dirty_rect.intersection(rect).unwrap_or_else(Rect::zero)
            }
        }
    }
}

impl<T: Copy, U> Copy for DirtyRect<T, U> {}
impl<T: Copy, U> Clone for DirtyRect<T, U> {
    fn clone(&self) -> Self { *self }
}

impl<T: Copy, U> From<Rect<T, U>> for DirtyRect<T, U> {
    fn from(rect: Rect<T, U>) -> Self {
        DirtyRect::Partial(rect)
    }
}

/// Backing store for blob image command streams.
pub type BlobImageData = Vec<u8>;

/// Result type for blob raserization.
pub type BlobImageResult = Result<RasterizedBlobImage, BlobImageError>;

/// Metadata (but not storage) for a blob image.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BlobImageDescriptor {
    /// Surface of the image or tile to render in the same coordinate space as
    /// the drawing commands.
    pub rect: LayoutIntRect,
    /// Format for the data in the backing store.
    pub format: ImageFormat,
}

/// Representation of a rasterized blob image. This is obtained by passing
/// `BlobImageData` to the embedding via the rasterization callback.
pub struct RasterizedBlobImage {
    /// The rectangle that was rasterized in device pixels, relative to the
    /// image or tile.
    pub rasterized_rect: DeviceIntRect,
    /// Backing store. The format is stored out of band in `BlobImageDescriptor`.
    pub data: Arc<Vec<u8>>,
}

/// Error code for when blob rasterization failed.
#[derive(Clone, Debug)]
pub enum BlobImageError {
    /// Out of memory.
    Oom,
    /// Other failure, embedding-specified.
    Other(String),
}



/// A key identifying blob image rasterization work requested from the blob
/// image rasterizer.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlobImageRequest {
    /// Unique handle to the image.
    pub key: BlobImageKey,
    /// Tiling offset in number of tiles.
    pub tile: TileOffset,
}
