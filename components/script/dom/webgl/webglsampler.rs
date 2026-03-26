/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::weakref::WeakRef;
use servo_canvas_traits::webgl::WebGLError::*;
use servo_canvas_traits::webgl::{WebGLCommand, WebGLSamplerId, webgl_channel};

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl::webglobject::WebGLObject;
use crate::dom::webgl::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::dom::webglrenderingcontext::capture_webgl_backtrace;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableWebGLSampler {
    context: WeakRef<WebGLRenderingContext>,
    #[no_trace]
    gl_id: WebGLSamplerId,
    marked_for_deletion: Cell<bool>,
}

impl DroppableWebGLSampler {
    fn send_with_fallibility(&self, command: WebGLCommand, fallibility: Operation) {
        if let Some(root) = self.context.root() {
            let result = root.sender().send(command, capture_webgl_backtrace());
            if matches!(fallibility, Operation::Infallible) {
                result.expect("Operation failed");
            }
        }
    }

    fn delete(&self, operation_fallibility: Operation) {
        if !self.marked_for_deletion.get() {
            self.marked_for_deletion.set(true);
            self.send_with_fallibility(
                WebGLCommand::DeleteSampler(self.gl_id),
                operation_fallibility,
            );
        }
    }
}

impl Drop for DroppableWebGLSampler {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}

#[dom_struct(associated_memory)]
pub(crate) struct WebGLSampler {
    webgl_object: WebGLObject,
    droppable: DroppableWebGLSampler,
}

#[derive(Clone, Copy)]
pub(crate) enum WebGLSamplerValue {
    Float(f32),
    GLenum(u32),
}

fn validate_params(pname: u32, value: WebGLSamplerValue) -> bool {
    match value {
        WebGLSamplerValue::GLenum(value) => {
            let allowed_values = match pname {
                constants::TEXTURE_MIN_FILTER => &[
                    constants::NEAREST,
                    constants::LINEAR,
                    constants::NEAREST_MIPMAP_NEAREST,
                    constants::LINEAR_MIPMAP_NEAREST,
                    constants::NEAREST_MIPMAP_LINEAR,
                    constants::LINEAR_MIPMAP_LINEAR,
                ][..],
                constants::TEXTURE_MAG_FILTER => &[constants::NEAREST, constants::LINEAR][..],
                constants::TEXTURE_WRAP_R |
                constants::TEXTURE_WRAP_S |
                constants::TEXTURE_WRAP_T => &[
                    constants::CLAMP_TO_EDGE,
                    constants::MIRRORED_REPEAT,
                    constants::REPEAT,
                ][..],
                constants::TEXTURE_COMPARE_MODE => {
                    &[constants::NONE, constants::COMPARE_REF_TO_TEXTURE][..]
                },
                constants::TEXTURE_COMPARE_FUNC => &[
                    constants::LEQUAL,
                    constants::GEQUAL,
                    constants::LESS,
                    constants::GREATER,
                    constants::EQUAL,
                    constants::NOTEQUAL,
                    constants::ALWAYS,
                    constants::NEVER,
                ][..],
                _ => &[][..],
            };
            allowed_values.contains(&value)
        },
        WebGLSamplerValue::Float(_) => matches!(
            pname,
            constants::TEXTURE_MIN_LOD | constants::TEXTURE_MAX_LOD
        ),
    }
}

impl WebGLSampler {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLSamplerId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            droppable: DroppableWebGLSampler {
                context: WeakRef::new(context),
                gl_id: id,
                marked_for_deletion: Cell::new(false),
            },
        }
    }

    pub(crate) fn new(context: &WebGLRenderingContext, can_gc: CanGc) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::GenerateSampler(sender));
        let id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(Self::new_inherited(context, id)),
            &*context.global(),
            can_gc,
        )
    }

    fn id(&self) -> WebGLSamplerId {
        self.droppable.gl_id
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        self.droppable.delete(operation_fallibility);
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.droppable.marked_for_deletion.get()
    }

    pub(crate) fn bind(
        &self,
        context: &WebGLRenderingContext,
        unit: u32,
    ) -> Result<(), servo_canvas_traits::webgl::WebGLError> {
        if !self.is_valid() {
            return Err(InvalidOperation);
        }
        context.send_command(WebGLCommand::BindSampler(unit, self.id()));
        Ok(())
    }

    pub(crate) fn set_parameter(
        &self,
        context: &WebGLRenderingContext,
        pname: u32,
        value: WebGLSamplerValue,
    ) -> Result<(), servo_canvas_traits::webgl::WebGLError> {
        if !self.is_valid() {
            return Err(InvalidOperation);
        }
        if !validate_params(pname, value) {
            return Err(InvalidEnum);
        }
        let command = match value {
            WebGLSamplerValue::GLenum(value) => {
                WebGLCommand::SetSamplerParameterInt(self.id(), pname, value as i32)
            },
            WebGLSamplerValue::Float(value) => {
                WebGLCommand::SetSamplerParameterFloat(self.id(), pname, value)
            },
        };
        context.send_command(command);
        Ok(())
    }

    pub(crate) fn get_parameter(
        &self,
        context: &WebGLRenderingContext,
        pname: u32,
    ) -> Result<WebGLSamplerValue, servo_canvas_traits::webgl::WebGLError> {
        if !self.is_valid() {
            return Err(InvalidOperation);
        }
        match pname {
            constants::TEXTURE_MIN_FILTER |
            constants::TEXTURE_MAG_FILTER |
            constants::TEXTURE_WRAP_R |
            constants::TEXTURE_WRAP_S |
            constants::TEXTURE_WRAP_T |
            constants::TEXTURE_COMPARE_FUNC |
            constants::TEXTURE_COMPARE_MODE => {
                let (sender, receiver) = webgl_channel().unwrap();
                context.send_command(WebGLCommand::GetSamplerParameterInt(
                    self.id(),
                    pname,
                    sender,
                ));
                Ok(WebGLSamplerValue::GLenum(receiver.recv().unwrap() as u32))
            },
            constants::TEXTURE_MIN_LOD | constants::TEXTURE_MAX_LOD => {
                let (sender, receiver) = webgl_channel().unwrap();
                context.send_command(WebGLCommand::GetSamplerParameterFloat(
                    self.id(),
                    pname,
                    sender,
                ));
                Ok(WebGLSamplerValue::Float(receiver.recv().unwrap()))
            },
            _ => Err(InvalidEnum),
        }
    }
}
