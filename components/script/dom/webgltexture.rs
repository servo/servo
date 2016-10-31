/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::CanvasMsg;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLTextureBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::globalscope::GlobalScope;
use dom::webgl_validations::types::{TexImageTarget, TexFormat, TexDataType};
use dom::webglobject::WebGLObject;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use std::cmp;
use webrender_traits::{WebGLCommand, WebGLError, WebGLResult, WebGLTextureId};

pub enum TexParameterValue {
    Float(f32),
    Int(i32),
}

const MAX_LEVEL_COUNT: usize = 31;
const MAX_FACE_COUNT: usize = 6;

no_jsmanaged_fields!([ImageInfo; MAX_LEVEL_COUNT * MAX_FACE_COUNT]);

#[dom_struct]
pub struct WebGLTexture {
    webgl_object: WebGLObject,
    id: WebGLTextureId,
    /// The target to which this texture was bound the first time
    target: Cell<Option<u32>>,
    is_deleted: Cell<bool>,
    /// Stores information about mipmap levels and cubemap faces.
    #[ignore_heap_size_of = "Arrays are cumbersome"]
    image_info_array: DOMRefCell<[ImageInfo; MAX_LEVEL_COUNT * MAX_FACE_COUNT]>,
    /// Face count can only be 1 or 6
    face_count: Cell<u8>,
    base_mipmap_level: u32,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLTexture {
    fn new_inherited(renderer: IpcSender<CanvasMsg>,
                     id: WebGLTextureId)
                     -> WebGLTexture {
        WebGLTexture {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            target: Cell::new(None),
            is_deleted: Cell::new(false),
            face_count: Cell::new(0),
            base_mipmap_level: 0,
            image_info_array: DOMRefCell::new([ImageInfo::new(); MAX_LEVEL_COUNT * MAX_FACE_COUNT]),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: &GlobalScope, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLTexture>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateTexture(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|texture_id| WebGLTexture::new(global, renderer, texture_id))
    }

    pub fn new(global: &GlobalScope,
               renderer: IpcSender<CanvasMsg>,
               id: WebGLTextureId)
               -> Root<WebGLTexture> {
        reflect_dom_object(box WebGLTexture::new_inherited(renderer, id),
                           global,
                           WebGLTextureBinding::Wrap)
    }
}


impl WebGLTexture {
    pub fn id(&self) -> WebGLTextureId {
        self.id
    }

    // NB: Only valid texture targets come here
    pub fn bind(&self, target: u32) -> WebGLResult<()> {
        if self.is_deleted.get() {
            return Err(WebGLError::InvalidOperation);
        }

        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        } else {
            // This is the first time binding
            let face_count = match target {
                constants::TEXTURE_2D => 1,
                constants::TEXTURE_CUBE_MAP => 6,
                _ => return Err(WebGLError::InvalidOperation)
            };
            self.face_count.set(face_count);
            self.target.set(Some(target));
        }

        let msg = CanvasMsg::WebGL(WebGLCommand::BindTexture(target, Some(self.id)));
        self.renderer.send(msg).unwrap();

        Ok(())
    }

    pub fn initialize(&self,
                      target: TexImageTarget,
                      width: u32,
                      height: u32,
                      depth: u32,
                      internal_format: TexFormat,
                      level: u32,
                      data_type: Option<TexDataType>) -> WebGLResult<()> {
        let image_info = ImageInfo {
            width: width,
            height: height,
            depth: depth,
            internal_format: Some(internal_format),
            is_initialized: true,
            data_type: data_type,
        };

        let face_index = self.face_index_for_target(&target);
        self.set_image_infos_at_level_and_face(level, face_index, image_info);
        Ok(())
    }

    pub fn generate_mipmap(&self) -> WebGLResult<()> {
        let target = match self.target.get() {
            Some(target) => target,
            None => {
                error!("Cannot generate mipmap on texture that has no target!");
                return Err(WebGLError::InvalidOperation);
            }
        };

        let base_image_info = self.base_image_info().unwrap();
        if !base_image_info.is_initialized() {
            return Err(WebGLError::InvalidOperation);
        }

        let is_cubic = target == constants::TEXTURE_CUBE_MAP;
        if is_cubic && !self.is_cube_complete() {
            return Err(WebGLError::InvalidOperation);
        }

        if !base_image_info.is_power_of_two() {
            return Err(WebGLError::InvalidOperation);
        }

        if base_image_info.is_compressed_format() {
            return Err(WebGLError::InvalidOperation);
        }

        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::GenerateMipmap(target))).unwrap();

        if self.base_mipmap_level + base_image_info.get_max_mimap_levels() == 0 {
            return Err(WebGLError::InvalidOperation);
        }

        let last_level = self.base_mipmap_level + base_image_info.get_max_mimap_levels() - 1;
        self.populate_mip_chain(self.base_mipmap_level, last_level)
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DeleteTexture(self.id)));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn target(&self) -> Option<u32> {
        self.target.get()
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
                            .send(CanvasMsg::WebGL(WebGLCommand::TexParameteri(target, name, int_value)))
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
                            .send(CanvasMsg::WebGL(WebGLCommand::TexParameteri(target, name, int_value)))
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
                            .send(CanvasMsg::WebGL(WebGLCommand::TexParameteri(target, name, int_value)))
                            .unwrap();
                        Ok(())
                    },

                    _ => Err(WebGLError::InvalidEnum),
                }
            },

            _ => Err(WebGLError::InvalidEnum),
        }
    }

    pub fn populate_mip_chain(&self, first_level: u32, last_level: u32) -> WebGLResult<()> {
        let base_image_info = self.image_info_at_face(0, first_level);
        if !base_image_info.is_initialized() {
            return Err(WebGLError::InvalidOperation);
        }

        let mut ref_width = base_image_info.width;
        let mut ref_height = base_image_info.height;

        if ref_width == 0 || ref_height == 0 {
            return Err(WebGLError::InvalidOperation);
        }

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
                data_type: base_image_info.data_type,
            };

            self.set_image_infos_at_level(level, image_info);
        }
        Ok(())
    }

    fn is_cube_complete(&self) -> bool {
        debug_assert!(self.face_count.get() == 6);

        let image_info = self.base_image_info().unwrap();
        if !image_info.is_defined() {
            return false;
        }

        let ref_width = image_info.width;
        let ref_format = image_info.internal_format;

        for face in 0..self.face_count.get() {
            let current_image_info = self.image_info_at_face(face, self.base_mipmap_level);
            if !current_image_info.is_defined() {
                return false;
            }

            // Compares height with width to enforce square dimensions
            if current_image_info.internal_format != ref_format ||
               current_image_info.width != ref_width ||
               current_image_info.height != ref_width {
                return false;
            }
        }

        true
    }

    fn face_index_for_target(&self,
                             target: &TexImageTarget) -> u8 {
        match *target {
            TexImageTarget::Texture2D => 0,
            TexImageTarget::CubeMapPositiveX => 0,
            TexImageTarget::CubeMapNegativeX => 1,
            TexImageTarget::CubeMapPositiveY => 2,
            TexImageTarget::CubeMapNegativeY => 3,
            TexImageTarget::CubeMapPositiveZ => 4,
            TexImageTarget::CubeMapNegativeZ => 5,
        }
    }

    pub fn image_info_for_target(&self,
                                 target: &TexImageTarget,
                                 level: u32) -> ImageInfo {
        let face_index = self.face_index_for_target(&target);
        self.image_info_at_face(face_index, level)
    }

    pub fn image_info_at_face(&self, face: u8, level: u32) -> ImageInfo {
        let pos = (level * self.face_count.get() as u32) + face as u32;
        self.image_info_array.borrow()[pos as usize]
    }

    fn set_image_infos_at_level(&self, level: u32, image_info: ImageInfo) {
        for face in 0..self.face_count.get() {
            self.set_image_infos_at_level_and_face(level, face, image_info);
        }
    }

    fn set_image_infos_at_level_and_face(&self, level: u32, face: u8, image_info: ImageInfo) {
        debug_assert!(face < self.face_count.get());
        let pos = (level * self.face_count.get() as u32) + face as u32;
        self.image_info_array.borrow_mut()[pos as usize] = image_info;
    }

    fn base_image_info(&self) -> Option<ImageInfo> {
        assert!((self.base_mipmap_level as usize) < MAX_LEVEL_COUNT);

        Some(self.image_info_at_face(0, self.base_mipmap_level))
    }
}

impl Drop for WebGLTexture {
    fn drop(&mut self) {
        self.delete();
    }
}

#[derive(Clone, Copy, PartialEq, Debug, JSTraceable, HeapSizeOf)]
pub struct ImageInfo {
    width: u32,
    height: u32,
    depth: u32,
    internal_format: Option<TexFormat>,
    is_initialized: bool,
    data_type: Option<TexDataType>,
}

impl ImageInfo {
    fn new() -> ImageInfo {
        ImageInfo {
            width: 0,
            height: 0,
            depth: 0,
            internal_format: None,
            is_initialized: false,
            data_type: None,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn internal_format(&self) -> Option<TexFormat> {
        self.internal_format
    }

    pub fn data_type(&self) -> Option<TexDataType> {
        self.data_type
    }

    fn is_power_of_two(&self) -> bool {
        self.width.is_power_of_two() &&
        self.height.is_power_of_two() &&
        self.depth.is_power_of_two()
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn is_defined(&self) -> bool {
        self.internal_format.is_some()
    }

    fn get_max_mimap_levels(&self) -> u32 {
        let largest = cmp::max(cmp::max(self.width, self.height), self.depth);
        if largest == 0 {
            return 0;
        }
        // FloorLog2(largest) + 1
        (largest as f64).log2() as u32 + 1
    }

    fn is_compressed_format(&self) -> bool {
        // TODO: Once Servo supports compressed formats, check for them here
        false
    }
}
