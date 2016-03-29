/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::{CanvasMsg, CanvasWebGLMsg, WebGLError, WebGLResult};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLTextureBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::webglobject::WebGLObject;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use std::cmp;

pub enum TexParameterValue {
    Float(f32),
    Int(i32),
}

const MAX_LEVEL_COUNT: usize = 31;
const MAX_FACE_COUNT: usize = 6;

#[dom_struct]
pub struct WebGLTexture {
    webgl_object: WebGLObject,
    id: u32,
    /// The target to which this texture was bound the first time
    target: Cell<Option<u32>>,
    is_deleted: Cell<bool>,
    is_resolved: Cell<bool>,
    is_initialized: Cell<bool>,
    #[ignore_heap_size_of = "Arrays are cumbersome"]
    image_info_array: DOMRefCell<[ImageInfo; MAX_LEVEL_COUNT * MAX_FACE_COUNT]>,
    /// Face count can only be 1 or 6
    face_count: u8,
    base_mipmap_level: u32,
    max_mipmap_level: u32,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLTexture {
    fn new_inherited(renderer: IpcSender<CanvasMsg>, id: u32) -> WebGLTexture {
        WebGLTexture {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            target: Cell::new(None),
            is_deleted: Cell::new(false),
            is_resolved: Cell::new(false),
            is_initialized: Cell::new(false),
            face_count: 0,
            base_mipmap_level: 0,
            max_mipmap_level: 100,
            image_info_array: DOMRefCell::new([ImageInfo::new(); MAX_LEVEL_COUNT * MAX_FACE_COUNT]),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: GlobalRef, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLTexture>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateTexture(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|texture_id| WebGLTexture::new(global, renderer, *texture_id))
    }

    pub fn new(global: GlobalRef, renderer: IpcSender<CanvasMsg>, id: u32) -> Root<WebGLTexture> {
        reflect_dom_object(box WebGLTexture::new_inherited(renderer, id), global, WebGLTextureBinding::Wrap)
    }
}


impl WebGLTexture {
    pub fn id(&self) -> u32 {
        self.id
    }

    // NB: Only valid texture targets come here
    pub fn bind(&self, target: u32) -> WebGLResult<()> {
        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        } else {
            self.target.set(Some(target));
        }

        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BindTexture(target, self.id))).unwrap();

        Ok(())
    }

    pub fn initialize(&self, width: u32, height: u32, internal_format: u32, level: i32) {
        // TODO(ConnorGBrewster): Add support for cubic textures
        let image_info = ImageInfo {
            width: width,
            height: height,
            // TODO: Support depth
            depth: 0,
            internal_format: internal_format,
            is_initialized: true,
        };
        self.set_image_infos_at_level(level as u32, image_info);

        // TODO: ZeroTextureData
        self.is_initialized.set(true);
    }

    pub fn generate_mipmap(&self) -> WebGLResult<()> {
        if self.target.get().is_none() {
            error!("Cannot generate mipmap on texture that has no target!");
            return Err(WebGLError::InvalidOperation);
        }
        // TODO: Check
        // GL_INVALID_OPERATION is generated if the texture bound to target is a cube map, but its
        // six faces do not share indentical widths, heights, formats, and types.
        //
        // GL_INVALID_OPERATION is generated if the zero level array is stored in a compressed internal format.
        let base_image_info = self.base_image_info().unwrap();

        if !base_image_info.is_initialized() {
            return Err(WebGLError::InvalidOperation);
        }

        if !base_image_info.is_power_of_two() {
            return Err(WebGLError::InvalidOperation);
        }

        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GenerateMipmap(self.target.get().unwrap()))).unwrap();

        let last_level = self.base_mipmap_level + base_image_info.get_max_mimap_levels() - 1;
        self.populate_mip_chain(self.base_mipmap_level, last_level)
    }

    pub fn populate_mip_chain(&self, first_level: u32, last_level: u32) -> WebGLResult<()> {
        let base_image_info = self.image_info_at_face(0, first_level);
        if !base_image_info.is_initialized() {
            return Err(WebGLError::InvalidOperation);
        }

        let mut ref_width = base_image_info.width;
        let mut ref_height = base_image_info.height;

        for level in (first_level + 1)..last_level {
            if ref_width == 1 && ref_height == 1 {
                break;
            }

            ref_width = cmp::max(1, ref_width / 2);
            ref_height = cmp::max(1, ref_height / 2);

            let image_info = ImageInfo {
                width: ref_width,
                height: ref_height,
                depth: 0,
                internal_format: base_image_info.internal_format,
                is_initialized: base_image_info.is_initialized(),
            };

            self.set_image_infos_at_level(level, image_info);
        }
        Ok(())
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteTexture(self.id)));
        }
    }

    /// We have to follow the conversion rules for GLES 2.0. See:
    ///   https://www.khronos.org/webgl/public-mailing-list/archives/1008/msg00014.html
    ///
    pub fn tex_parameter(&self,
                     target: u32,
                     name: u32,
                     value: TexParameterValue) -> WebGLResult<()> {
        let (int_value, _float_value) = match value {
            TexParameterValue::Int(int_value) => (int_value, int_value as f32),
            TexParameterValue::Float(float_value) => (float_value as i32, float_value),
        };

        match name {
            constants::TEXTURE_MIN_FILTER => {
                match int_value as u32 {
                    constants::NEAREST |
                    constants::LINEAR |
                    constants::NEAREST_MIPMAP_NEAREST |
                    constants::LINEAR_MIPMAP_NEAREST |
                    constants::NEAREST_MIPMAP_LINEAR |
                    constants::LINEAR_MIPMAP_LINEAR => {
                        self.renderer
                            .send(CanvasMsg::WebGL(CanvasWebGLMsg::TexParameteri(target, name, int_value)))
                            .unwrap();
                        Ok(())
                    },

                    _ => Err(WebGLError::InvalidEnum),
                }
            },
            constants::TEXTURE_MAG_FILTER => {
                match int_value as u32 {
                    constants::NEAREST |
                    constants::LINEAR => {
                        self.renderer
                            .send(CanvasMsg::WebGL(CanvasWebGLMsg::TexParameteri(target, name, int_value)))
                            .unwrap();
                        Ok(())
                    },

                    _ => Err(WebGLError::InvalidEnum),
                }
            },
            constants::TEXTURE_WRAP_S |
            constants::TEXTURE_WRAP_T => {
                match int_value as u32 {
                    constants::CLAMP_TO_EDGE |
                    constants::MIRRORED_REPEAT |
                    constants::REPEAT => {
                        self.renderer
                            .send(CanvasMsg::WebGL(CanvasWebGLMsg::TexParameteri(target, name, int_value)))
                            .unwrap();
                        Ok(())
                    },

                    _ => Err(WebGLError::InvalidEnum),
                }
            },

            _ => Err(WebGLError::InvalidEnum),
        }
    }

    fn image_info_at_face(&self, face: u8, level: u32) -> ImageInfo {
        let pos = (level * self.face_count as u32) + face as u32;
        self.image_info_array.borrow()[pos as usize]
    }

    fn image_info_at(&self, level: u32) -> ImageInfo {
        // TODO: Support Cubic Textures
        let face = 0;
        self.image_info_at_face(face, level)
    }

    fn set_image_infos_at_level(&self, level: u32, image_info: ImageInfo) {
        for face in 0..self.face_count {
            let pos = (level * self.face_count as u32) + face as u32;
            self.image_info_array.borrow_mut()[pos as usize] = image_info;
        }

        self.invalidate_resolve_cache();
    }

    fn base_image_info(&self) -> Option<ImageInfo> {
        if self.base_mipmap_level >= MAX_LEVEL_COUNT as u32 {
            return None;
        }
        Some(self.image_info_at_face(0, self.base_mipmap_level))
    }

    fn invalidate_resolve_cache(&self) {
        self.is_resolved.set(false);
    }
}

impl Drop for WebGLTexture {
    fn drop(&mut self) {
        self.delete();
    }
}

#[derive(Clone, Copy, PartialEq, Debug, JSTraceable, HeapSizeOf)]
struct ImageInfo {
    width: u32,
    height: u32,
    depth: u32,
    internal_format: u32,
    is_initialized: bool,
}

impl ImageInfo {
    fn new() -> ImageInfo {
        ImageInfo {
            width: 0,
            height: 0,
            depth: 0,
            internal_format: 0,
            is_initialized: false,
        }
    }

    fn is_power_of_two(&self) -> bool {
        let width = self.width;
        let height = self.height;
        let width_is_power_of_two = ((width * width) as f64).sqrt() as u32 == width;
        let height_is_power_of_two = ((height * height) as f64).sqrt() as u32 == height;
        width_is_power_of_two && height_is_power_of_two
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn get_max_mimap_levels(&self) -> u32 {
        let largest = cmp::max(cmp::max(self.width, self.height), self.depth);
        if largest == 0 {
            return 0;
        }
        // FloorLog2(largest) + 1
        (largest as f64).log2() as u32 + 1
    }
}
