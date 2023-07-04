/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::path::{Path, PathBuf};

use api::{CaptureBits, ExternalImageData, ImageDescriptor};
#[cfg(feature = "png")]
use api::ImageFormat;
use api::units::TexelRect;
#[cfg(feature = "png")]
use api::units::DeviceIntSize;
#[cfg(feature = "capture")]
use crate::print_tree::{PrintableTree, PrintTree};
use ron;
use serde;


#[derive(Clone)]
pub struct CaptureConfig {
    pub root: PathBuf,
    pub bits: CaptureBits,
    /// Scene sequence ID when capturing multiple frames. Zero for a single frame capture.
    pub scene_id: u32,
    /// Frame sequence ID when capturing multiple frames. Zero for a single frame capture.
    pub frame_id: u32,
    /// Resource sequence ID when capturing multiple frames. Zero for a single frame capture.
    pub resource_id: u32,
    #[cfg(feature = "capture")]
    pretty: ron::ser::PrettyConfig,
}

impl CaptureConfig {
    #[cfg(any(feature = "capture", feature = "replay"))]
    pub fn new(root: PathBuf, bits: CaptureBits) -> Self {
        CaptureConfig {
            root,
            bits,
            scene_id: 0,
            frame_id: 0,
            resource_id: 0,
            #[cfg(feature = "capture")]
            pretty: ron::ser::PrettyConfig {
                enumerate_arrays: true,
                .. ron::ser::PrettyConfig::default()
            },
        }
    }

    #[cfg(feature = "capture")]
    pub fn prepare_scene(&mut self) {
        use std::fs::create_dir_all;
        self.scene_id += 1;
        let _ = create_dir_all(&self.scene_root());
    }

    #[cfg(feature = "capture")]
    pub fn prepare_frame(&mut self) {
        use std::fs::create_dir_all;
        self.frame_id += 1;
        let _ = create_dir_all(&self.frame_root());
    }

    #[cfg(feature = "capture")]
    pub fn prepare_resource(&mut self) {
        use std::fs::create_dir_all;
        self.resource_id += 1;
        let _ = create_dir_all(&self.resource_root());
    }

    #[cfg(any(feature = "capture", feature = "replay"))]
    pub fn scene_root(&self) -> PathBuf {
        if self.scene_id > 0 {
            let path = format!("scenes/{:05}", self.scene_id);
            self.root.join(path)
        } else {
            self.root.clone()
        }
    }

    #[cfg(any(feature = "capture", feature = "replay"))]
    pub fn frame_root(&self) -> PathBuf {
        if self.frame_id > 0 {
            let path = format!("frames/{:05}", self.frame_id);
            self.scene_root().join(path)
        } else {
            self.root.clone()
        }
    }

    #[cfg(any(feature = "capture", feature = "replay"))]
    pub fn resource_root(&self) -> PathBuf {
        if self.resource_id > 0 {
            let path = format!("resources/{:05}", self.resource_id);
            self.root.join(path)
        } else {
            self.root.clone()
        }
    }

    #[cfg(feature = "capture")]
    pub fn serialize_for_scene<T, P>(&self, data: &T, name: P)
    where
        T: serde::Serialize,
        P: AsRef<Path>,
    {
        self.serialize(data, self.scene_root(), name)
    }

    #[cfg(feature = "capture")]
    pub fn serialize_for_frame<T, P>(&self, data: &T, name: P)
    where
        T: serde::Serialize,
        P: AsRef<Path>,
    {
        self.serialize(data, self.frame_root(), name)
    }

    #[cfg(feature = "capture")]
    pub fn serialize_for_resource<T, P>(&self, data: &T, name: P)
    where
        T: serde::Serialize,
        P: AsRef<Path>,
    {
        self.serialize(data, self.resource_root(), name)
    }

    #[cfg(feature = "capture")]
    pub fn file_path_for_frame<P>(&self, name: P, ext: &str) -> PathBuf
    where P: AsRef<Path> {
        self.frame_root().join(name).with_extension(ext)
    }

    #[cfg(feature = "capture")]
    fn serialize<T, P>(&self, data: &T, path: PathBuf, name: P)
    where
        T: serde::Serialize,
        P: AsRef<Path>,
    {
        use std::io::Write;
        let ron = ron::ser::to_string_pretty(data, self.pretty.clone())
            .unwrap();
        let mut file = File::create(path.join(name).with_extension("ron"))
            .unwrap();
        write!(file, "{}\n", ron)
            .unwrap();
    }

    #[cfg(feature = "capture")]
    fn serialize_tree<T, P>(data: &T, root: PathBuf, name: P)
    where
        T: PrintableTree,
        P: AsRef<Path>
    {
        let path = root
            .join(name)
            .with_extension("tree");
        let file = File::create(path)
            .unwrap();
        let mut pt = PrintTree::new_with_sink("", file);
        data.print_with(&mut pt);
    }

    #[cfg(feature = "capture")]
    pub fn serialize_tree_for_frame<T, P>(&self, data: &T, name: P)
    where
        T: PrintableTree,
        P: AsRef<Path>
    {
        Self::serialize_tree(data, self.frame_root(), name)
    }

    #[cfg(feature = "replay")]
    fn deserialize<T, P>(root: &PathBuf, name: P) -> Option<T>
    where
        T: for<'a> serde::Deserialize<'a>,
        P: AsRef<Path>,
    {
        use std::io::Read;

        let mut string = String::new();
        let path = root
            .join(name.as_ref())
            .with_extension("ron");
        File::open(path)
            .ok()?
            .read_to_string(&mut string)
            .unwrap();
        match ron::de::from_str(&string) {
            Ok(out) => Some(out),
            Err(e) => panic!("File {:?} deserialization failed: {:?}", name.as_ref(), e),
        }
    }

    #[cfg(feature = "replay")]
    pub fn deserialize_for_scene<T, P>(&self, name: P) -> Option<T>
    where
        T: for<'a> serde::Deserialize<'a>,
        P: AsRef<Path>,
    {
        Self::deserialize(&self.scene_root(), name)
    }

    #[cfg(feature = "replay")]
    pub fn deserialize_for_frame<T, P>(&self, name: P) -> Option<T>
    where
        T: for<'a> serde::Deserialize<'a>,
        P: AsRef<Path>,
    {
        Self::deserialize(&self.frame_root(), name)
    }

    #[cfg(feature = "replay")]
    pub fn deserialize_for_resource<T, P>(&self, name: P) -> Option<T>
    where
        T: for<'a> serde::Deserialize<'a>,
        P: AsRef<Path>,
    {
        Self::deserialize(&self.resource_root(), name)
    }

    #[cfg(feature = "png")]
    pub fn save_png(
        path: PathBuf, size: DeviceIntSize, format: ImageFormat, stride: Option<i32>, data: &[u8],
    ) {
        use png::{BitDepth, ColorType, Encoder};
        use std::io::BufWriter;
        use std::borrow::Cow;

        // `png` expects
        let data = match stride {
            Some(stride) if stride != format.bytes_per_pixel() * size.width => {
                let mut unstrided = Vec::new();
                for y in 0..size.height {
                    let start = (y * stride) as usize;
                    unstrided.extend_from_slice(&data[start..start+(size.width * format.bytes_per_pixel()) as usize]);
                }
                Cow::from(unstrided)
            }
            _ => Cow::from(data),
        };

        let color_type = match format {
            ImageFormat::RGBA8 => ColorType::RGBA,
            ImageFormat::BGRA8 => {
                warn!("Unable to swizzle PNG of BGRA8 type");
                ColorType::RGBA
            },
            ImageFormat::R8 => ColorType::Grayscale,
            ImageFormat::RG8 => ColorType::GrayscaleAlpha,
            _ => {
                error!("Unable to save PNG of {:?}", format);
                return;
            }
        };
        let w = BufWriter::new(File::create(path).unwrap());
        let mut enc = Encoder::new(w, size.width as u32, size.height as u32);
        enc.set_color(color_type);
        enc.set_depth(BitDepth::Eight);
        enc
            .write_header()
            .unwrap()
            .write_image_data(&*data)
            .unwrap();
    }
}

/// An image that `ResourceCache` is unable to resolve during a capture.
/// The image has to be transferred to `Renderer` and locked with the
/// external image handler to get the actual contents and serialize them.
#[derive(Deserialize, Serialize)]
pub struct ExternalCaptureImage {
    pub short_path: String,
    pub descriptor: ImageDescriptor,
    pub external: ExternalImageData,
}

/// A short description of an external image to be saved separately as
/// "externals/XX.ron", redirecting into a specific texture/blob with
/// the corresponding UV rectangle.
#[derive(Deserialize, Serialize)]
pub struct PlainExternalImage {
    /// Path to the RON file describing the texel data.
    pub data: String,
    /// External image data source.
    pub external: ExternalImageData,
    /// UV sub-rectangle of the image.
    pub uv: TexelRect,
}
