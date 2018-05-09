/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::Size2D;
use gleam::gl;
use nonzero::NonZeroU32;
use offscreen_gl_context::{GLContextAttributes, GLLimits};
use serde_bytes::ByteBuf;
use std::fmt;
use webrender_api::{DocumentId, ImageKey, PipelineId};

/// Sender type used in WebGLCommands.
pub use ::webgl_channel::WebGLSender;
/// Receiver type used in WebGLCommands.
pub use ::webgl_channel::WebGLReceiver;
/// Result type for send()/recv() calls in in WebGLCommands.
pub use ::webgl_channel::WebGLSendResult;
/// Helper function that creates a WebGL channel (WebGLSender, WebGLReceiver) to be used in WebGLCommands.
pub use ::webgl_channel::webgl_channel;
/// Entry point type used in a Script Pipeline to get the WebGLChan to be used in that thread.
pub use ::webgl_channel::WebGLPipeline;
/// Entry point channel type used for sending WebGLMsg messages to the WebGL renderer.
pub use ::webgl_channel::WebGLChan;

/// WebGL Message API
#[derive(Clone, Deserialize, Serialize)]
pub enum WebGLMsg {
    /// Creates a new WebGLContext.
    CreateContext(WebGLVersion, Size2D<i32>, GLContextAttributes,
                  WebGLSender<Result<(WebGLCreateContextResult), String>>),
    /// Resizes a WebGLContext.
    ResizeContext(WebGLContextId, Size2D<i32>, WebGLSender<Result<(), String>>),
    /// Drops a WebGLContext.
    RemoveContext(WebGLContextId),
    /// Runs a WebGLCommand in a specific WebGLContext.
    WebGLCommand(WebGLContextId, WebGLCommand),
    /// Runs a WebVRCommand in a specific WebGLContext.
    WebVRCommand(WebGLContextId, WebVRCommand),
    /// Locks a specific WebGLContext. Lock messages are used for a correct synchronization
    /// with WebRender external image API.
    /// WR locks a external texture when it wants to use the shared texture contents.
    /// The WR client should not change the shared texture content until the Unlock call.
    /// Currently OpenGL Sync Objects are used to implement the synchronization mechanism.
    Lock(WebGLContextId, WebGLSender<(u32, Size2D<i32>, usize)>),
    /// Unlocks a specific WebGLContext. Unlock messages are used for a correct synchronization
    /// with WebRender external image API.
    /// The WR unlocks a context when it finished reading the shared texture contents.
    /// Unlock messages are always sent after a Lock message.
    Unlock(WebGLContextId),
    /// Creates or updates the image keys required for WebRender.
    UpdateWebRenderImage(WebGLContextId, WebGLSender<ImageKey>),
    /// Commands used for the DOMToTexture feature.
    DOMToTextureCommand(DOMToTextureCommand),
    /// Frees all resources and closes the thread.
    Exit,
}

/// Contains the WebGLCommand sender and information about a WebGLContext
#[derive(Clone, Deserialize, Serialize)]
pub struct WebGLCreateContextResult {
    /// Sender instance to send commands to the specific WebGLContext
    pub sender: WebGLMsgSender,
    /// Information about the internal GL Context.
    pub limits: GLLimits,
    /// How the WebGLContext is shared with WebRender.
    pub share_mode: WebGLContextShareMode,
    /// The GLSL version supported by the context.
    pub glsl_version: WebGLSLVersion
}

#[derive(Clone, Copy, Deserialize, MallocSizeOf, Serialize)]
pub enum WebGLContextShareMode {
    /// Fast: a shared texture_id is used in WebRender.
    SharedTexture,
    /// Slow: glReadPixels is used to send pixels to WebRender each frame.
    Readback,
}

/// Defines the WebGL version
#[derive(Clone, Copy, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum WebGLVersion {
    /// https://www.khronos.org/registry/webgl/specs/1.0.2/
    /// Conforms closely to the OpenGL ES 2.0 API
    WebGL1,
    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/
    /// Conforms closely to the OpenGL ES 3.0 API
    WebGL2,
}

/// Defines the GLSL version supported by the WebGL backend contexts.
#[derive(Clone, Copy, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub struct WebGLSLVersion {
    /// Major GLSL version
    pub major: u32,
    /// Minor GLSL version
    pub minor: u32,
}

/// Helper struct to send WebGLCommands to a specific WebGLContext.
#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct WebGLMsgSender {
    ctx_id: WebGLContextId,
    #[ignore_malloc_size_of = "channels are hard"]
    sender: WebGLChan,
}

impl WebGLMsgSender {
    pub fn new(id: WebGLContextId, sender: WebGLChan) -> Self {
        WebGLMsgSender {
            ctx_id: id,
            sender: sender,
        }
    }

    /// Returns the WebGLContextId associated to this sender
    pub fn context_id(&self) -> WebGLContextId {
        self.ctx_id
    }

    /// Send a WebGLCommand message
    #[inline]
    pub fn send(&self, command: WebGLCommand) -> WebGLSendResult {
        self.sender.send(WebGLMsg::WebGLCommand(self.ctx_id, command))
    }

    /// Send a WebVRCommand message
    #[inline]
    pub fn send_vr(&self, command: WebVRCommand) -> WebGLSendResult {
        self.sender.send(WebGLMsg::WebVRCommand(self.ctx_id, command))
    }

    /// Send a resize message
    #[inline]
    pub fn send_resize(&self,
                       size: Size2D<i32>,
                       sender: WebGLSender<Result<(), String>>)
                       -> WebGLSendResult {
        self.sender.send(WebGLMsg::ResizeContext(self.ctx_id, size, sender))
    }

    #[inline]
    pub fn send_remove(&self) -> WebGLSendResult {
        self.sender.send(WebGLMsg::RemoveContext(self.ctx_id))
    }

    #[inline]
    pub fn send_update_wr_image(&self, sender: WebGLSender<ImageKey>) -> WebGLSendResult {
        self.sender.send(WebGLMsg::UpdateWebRenderImage(self.ctx_id, sender))
    }

    pub fn send_dom_to_texture(&self, command: DOMToTextureCommand) -> WebGLSendResult {
        self.sender.send(WebGLMsg::DOMToTextureCommand(command))
    }
}

/// WebGL Commands for a specific WebGLContext
#[derive(Clone, Deserialize, Serialize)]
pub enum WebGLCommand {
    GetContextAttributes(WebGLSender<GLContextAttributes>),
    ActiveTexture(u32),
    BlendColor(f32, f32, f32, f32),
    BlendEquation(u32),
    BlendEquationSeparate(u32, u32),
    BlendFunc(u32, u32),
    BlendFuncSeparate(u32, u32, u32, u32),
    AttachShader(WebGLProgramId, WebGLShaderId),
    DetachShader(WebGLProgramId, WebGLShaderId),
    BindAttribLocation(WebGLProgramId, u32, String),
    BufferData(u32, ByteBuf, u32),
    BufferSubData(u32, isize, ByteBuf),
    Clear(u32),
    ClearColor(f32, f32, f32, f32),
    ClearDepth(f32),
    ClearStencil(i32),
    ColorMask(bool, bool, bool, bool),
    CullFace(u32),
    FrontFace(u32),
    DepthFunc(u32),
    DepthMask(bool),
    DepthRange(f32, f32),
    Enable(u32),
    Disable(u32),
    CompileShader(WebGLShaderId, String),
    CopyTexImage2D(u32, i32, u32, i32, i32, i32, i32, i32),
    CopyTexSubImage2D(u32, i32, i32, i32, i32, i32, i32, i32),
    CreateBuffer(WebGLSender<Option<WebGLBufferId>>),
    CreateFramebuffer(WebGLSender<Option<WebGLFramebufferId>>),
    CreateRenderbuffer(WebGLSender<Option<WebGLRenderbufferId>>),
    CreateTexture(WebGLSender<Option<WebGLTextureId>>),
    CreateProgram(WebGLSender<Option<WebGLProgramId>>),
    CreateShader(u32, WebGLSender<Option<WebGLShaderId>>),
    DeleteBuffer(WebGLBufferId),
    DeleteFramebuffer(WebGLFramebufferId),
    DeleteRenderbuffer(WebGLRenderbufferId),
    DeleteTexture(WebGLTextureId),
    DeleteProgram(WebGLProgramId),
    DeleteShader(WebGLShaderId),
    BindBuffer(u32, Option<WebGLBufferId>),
    BindFramebuffer(u32, WebGLFramebufferBindingRequest),
    BindRenderbuffer(u32, Option<WebGLRenderbufferId>),
    BindTexture(u32, Option<WebGLTextureId>),
    DisableVertexAttribArray(u32),
    DrawArrays(u32, i32, i32),
    DrawElements(u32, i32, u32, i64),
    EnableVertexAttribArray(u32),
    FramebufferRenderbuffer(u32, u32, u32, Option<WebGLRenderbufferId>),
    FramebufferTexture2D(u32, u32, u32, Option<WebGLTextureId>, i32),
    GetExtensions(WebGLSender<String>),
    GetTexParameter(u32, u32, WebGLSender<i32>),
    GetShaderPrecisionFormat(u32, u32, WebGLSender<(i32, i32, i32)>),
    GetActiveAttrib(WebGLProgramId, u32, WebGLSender<WebGLResult<(i32, u32, String)>>),
    GetActiveUniform(WebGLProgramId, u32, WebGLSender<WebGLResult<(i32, u32, String)>>),
    GetAttribLocation(WebGLProgramId, String, WebGLSender<Option<i32>>),
    GetUniformLocation(WebGLProgramId, String, WebGLSender<Option<i32>>),
    GetVertexAttribOffset(u32, u32, WebGLSender<isize>),
    GetShaderInfoLog(WebGLShaderId, WebGLSender<String>),
    GetProgramInfoLog(WebGLProgramId, WebGLSender<String>),
    GetFramebufferAttachmentParameter(u32, u32, u32, WebGLSender<i32>),
    GetRenderbufferParameter(u32, u32, WebGLSender<i32>),
    PolygonOffset(f32, f32),
    RenderbufferStorage(u32, u32, i32, i32),
    ReadPixels(i32, i32, i32, i32, u32, u32, WebGLSender<ByteBuf>),
    SampleCoverage(f32, bool),
    Scissor(i32, i32, i32, i32),
    StencilFunc(u32, i32, u32),
    StencilFuncSeparate(u32, u32, i32, u32),
    StencilMask(u32),
    StencilMaskSeparate(u32, u32),
    StencilOp(u32, u32, u32),
    StencilOpSeparate(u32, u32, u32, u32),
    Hint(u32, u32),
    IsEnabled(u32, WebGLSender<bool>),
    LineWidth(f32),
    PixelStorei(u32, i32),
    LinkProgram(WebGLProgramId),
    Uniform1f(i32, f32),
    Uniform1fv(i32, Vec<f32>),
    Uniform1i(i32, i32),
    Uniform1iv(i32, Vec<i32>),
    Uniform2f(i32, f32, f32),
    Uniform2fv(i32, Vec<f32>),
    Uniform2i(i32, i32, i32),
    Uniform2iv(i32, Vec<i32>),
    Uniform3f(i32, f32, f32, f32),
    Uniform3fv(i32, Vec<f32>),
    Uniform3i(i32, i32, i32, i32),
    Uniform3iv(i32, Vec<i32>),
    Uniform4f(i32, f32, f32, f32, f32),
    Uniform4fv(i32, Vec<f32>),
    Uniform4i(i32, i32, i32, i32, i32),
    Uniform4iv(i32, Vec<i32>),
    UniformMatrix2fv(i32, bool, Vec<f32>),
    UniformMatrix3fv(i32, bool, Vec<f32>),
    UniformMatrix4fv(i32, bool, Vec<f32>),
    UseProgram(WebGLProgramId),
    ValidateProgram(WebGLProgramId),
    VertexAttrib(u32, f32, f32, f32, f32),
    VertexAttribPointer(u32, i32, u32, bool, i32, u32),
    VertexAttribPointer2f(u32, i32, bool, i32, u32),
    SetViewport(i32, i32, i32, i32),
    TexImage2D(u32, i32, i32, i32, i32, u32, u32, ByteBuf),
    TexParameteri(u32, u32, i32),
    TexParameterf(u32, u32, f32),
    TexSubImage2D(u32, i32, i32, i32, i32, i32, u32, u32, ByteBuf),
    DrawingBufferWidth(WebGLSender<i32>),
    DrawingBufferHeight(WebGLSender<i32>),
    Finish(WebGLSender<()>),
    Flush,
    GenerateMipmap(u32),
    CreateVertexArray(WebGLSender<Option<WebGLVertexArrayId>>),
    DeleteVertexArray(WebGLVertexArrayId),
    BindVertexArray(Option<WebGLVertexArrayId>),
    GetParameterBool(ParameterBool, WebGLSender<bool>),
    GetParameterInt(ParameterInt, WebGLSender<i32>),
    GetParameterInt4(ParameterInt4, WebGLSender<[i32; 4]>),
    GetParameterFloat(ParameterFloat, WebGLSender<f32>),
    GetParameterFloat2(ParameterFloat2, WebGLSender<[f32; 2]>),
    GetProgramParameterBool(WebGLProgramId, ProgramParameterBool, WebGLSender<bool>),
    GetProgramParameterInt(WebGLProgramId, ProgramParameterInt, WebGLSender<i32>),
    GetShaderParameterBool(WebGLShaderId, ShaderParameterBool, WebGLSender<bool>),
    GetShaderParameterInt(WebGLShaderId, ShaderParameterInt, WebGLSender<i32>),
    GetVertexAttribBool(u32, VertexAttribBool, WebGLSender<WebGLResult<bool>>),
    GetVertexAttribInt(u32, VertexAttribInt, WebGLSender<WebGLResult<i32>>),
    GetVertexAttribFloat4(u32, VertexAttribFloat4, WebGLSender<WebGLResult<[f32; 4]>>),
}

macro_rules! define_resource_id_struct {
    ($name:ident) => {
        #[derive(Clone, Copy, Eq, Hash, PartialEq)]
        pub struct $name(NonZeroU32);

        impl $name {
            #[allow(unsafe_code)]
            #[inline]
            pub unsafe fn new(id: u32) -> Self {
                $name(NonZeroU32::new_unchecked(id))
            }

            #[inline]
            pub fn get(self) -> u32 {
                self.0.get()
            }
        }

    };
}

macro_rules! define_resource_id {
    ($name:ident) => {
        define_resource_id_struct!($name);

        #[allow(unsafe_code)]
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                let id = try!(u32::deserialize(deserializer));
                if id == 0 {
                    Err(::serde::de::Error::custom("expected a non-zero value"))
                } else {
                    Ok(unsafe { $name::new(id) })
                }
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                self.get().serialize(serializer)
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter)
                  -> Result<(), ::std::fmt::Error> {
                fmt.debug_tuple(stringify!($name))
                   .field(&self.get())
                   .finish()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter)
                  -> Result<(), ::std::fmt::Error> {
                write!(fmt, "{}", self.get())
            }
        }

        impl ::malloc_size_of::MallocSizeOf for $name {
            fn size_of(&self, _ops: &mut ::malloc_size_of::MallocSizeOfOps) -> usize { 0 }
        }
    }
}

define_resource_id!(WebGLBufferId);
define_resource_id!(WebGLFramebufferId);
define_resource_id!(WebGLRenderbufferId);
define_resource_id!(WebGLTextureId);
define_resource_id!(WebGLProgramId);
define_resource_id!(WebGLShaderId);
define_resource_id!(WebGLVertexArrayId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, Ord)]
#[derive(PartialEq, PartialOrd, Serialize)]
pub struct WebGLContextId(pub usize);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum WebGLError {
    InvalidEnum,
    InvalidFramebufferOperation,
    InvalidOperation,
    InvalidValue,
    OutOfMemory,
    ContextLost,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebGLFramebufferBindingRequest {
    Explicit(WebGLFramebufferId),
    Default,
}

pub type WebGLResult<T> = Result<T, WebGLError>;

pub type WebVRDeviceId = u32;

// WebVR commands that must be called in the WebGL render thread.
#[derive(Clone, Deserialize, Serialize)]
pub enum WebVRCommand {
    /// Start presenting to a VR device.
    Create(WebVRDeviceId),
    /// Synchronize the pose information to be used in the frame.
    SyncPoses(WebVRDeviceId, f64, f64, WebGLSender<Result<ByteBuf, ()>>),
    /// Submit the frame to a VR device using the specified texture coordinates.
    SubmitFrame(WebVRDeviceId, [f32; 4], [f32; 4]),
    /// Stop presenting to a VR device
    Release(WebVRDeviceId)
}

// Trait object that handles WebVR commands.
// Receives the texture id and size associated to the WebGLContext.
pub trait WebVRRenderHandler: Send {
    fn handle(&mut self, command: WebVRCommand, texture: Option<(u32, Size2D<i32>)>);
}

/// WebGL commands required to implement DOMToTexture feature.
#[derive(Clone, Deserialize, Serialize)]
pub enum DOMToTextureCommand {
    /// Attaches a HTMLIFrameElement to a WebGLTexture.
    Attach(WebGLContextId, WebGLTextureId, DocumentId, PipelineId, Size2D<i32>),
    /// Releases the HTMLIFrameElement to WebGLTexture attachment.
    Detach(WebGLTextureId),
    /// Lock message used for a correct synchronization with WebRender GL flow.
    Lock(PipelineId, usize, WebGLSender<Option<(u32, Size2D<i32>)>>),
}

impl fmt::Debug for WebGLCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::WebGLCommand::*;
        let name = match *self {
            GetContextAttributes(..) => "GetContextAttributes",
            ActiveTexture(..) => "ActiveTexture",
            BlendColor(..) => "BlendColor",
            BlendEquation(..) => "BlendEquation",
            BlendEquationSeparate(..) => "BlendEquationSeparate",
            BlendFunc(..) => "BlendFunc",
            BlendFuncSeparate(..) => "BlendFuncSeparate",
            AttachShader(..) => "AttachShader",
            DetachShader(..) => "DetachShader",
            BindAttribLocation(..) => "BindAttribLocation",
            BufferData(..) => "BufferData",
            BufferSubData(..) => "BufferSubData",
            Clear(..) => "Clear",
            ClearColor(..) => "ClearColor",
            ClearDepth(..) => "ClearDepth",
            ClearStencil(..) => "ClearStencil",
            ColorMask(..) => "ColorMask",
            CopyTexImage2D(..) => "CopyTexImage2D",
            CopyTexSubImage2D(..) => "CopyTexSubImage2D",
            CullFace(..) => "CullFace",
            FrontFace(..) => "FrontFace",
            DepthFunc(..) => "DepthFunc",
            DepthMask(..) => "DepthMask",
            DepthRange(..) => "DepthRange",
            Enable(..) => "Enable",
            Disable(..) => "Disable",
            CompileShader(..) => "CompileShader",
            CreateBuffer(..) => "CreateBuffer",
            CreateFramebuffer(..) => "CreateFramebuffer",
            CreateRenderbuffer(..) => "CreateRenderbuffer",
            CreateTexture(..) => "CreateTexture",
            CreateProgram(..) => "CreateProgram",
            CreateShader(..) => "CreateShader",
            DeleteBuffer(..) => "DeleteBuffer",
            DeleteFramebuffer(..) => "DeleteFramebuffer",
            DeleteRenderbuffer(..) => "DeleteRenderBuffer",
            DeleteTexture(..) => "DeleteTexture",
            DeleteProgram(..) => "DeleteProgram",
            DeleteShader(..) => "DeleteShader",
            BindBuffer(..) => "BindBuffer",
            BindFramebuffer(..) => "BindFramebuffer",
            BindRenderbuffer(..) => "BindRenderbuffer",
            BindTexture(..) => "BindTexture",
            DisableVertexAttribArray(..) => "DisableVertexAttribArray",
            DrawArrays(..) => "DrawArrays",
            DrawElements(..) => "DrawElements",
            EnableVertexAttribArray(..) => "EnableVertexAttribArray",
            FramebufferRenderbuffer(..) => "FramebufferRenderbuffer",
            FramebufferTexture2D(..) => "FramebufferTexture2D",
            GetExtensions(..) => "GetExtensions",
            GetTexParameter(..) => "GetTexParameter",
            GetShaderPrecisionFormat(..) => "GetShaderPrecisionFormat",
            GetActiveAttrib(..) => "GetActiveAttrib",
            GetActiveUniform(..) => "GetActiveUniform",
            GetAttribLocation(..) => "GetAttribLocation",
            GetUniformLocation(..) => "GetUniformLocation",
            GetShaderInfoLog(..) => "GetShaderInfoLog",
            GetProgramInfoLog(..) => "GetProgramInfoLog",
            GetVertexAttribOffset(..) => "GetVertexAttribOffset",
            GetFramebufferAttachmentParameter(..) => "GetFramebufferAttachmentParameter",
            GetRenderbufferParameter(..) => "GetRenderbufferParameter",
            PolygonOffset(..) => "PolygonOffset",
            ReadPixels(..) => "ReadPixels",
            RenderbufferStorage(..) => "RenderbufferStorage",
            SampleCoverage(..) => "SampleCoverage",
            Scissor(..) => "Scissor",
            StencilFunc(..) => "StencilFunc",
            StencilFuncSeparate(..) => "StencilFuncSeparate",
            StencilMask(..) => "StencilMask",
            StencilMaskSeparate(..) => "StencilMaskSeparate",
            StencilOp(..) => "StencilOp",
            StencilOpSeparate(..) => "StencilOpSeparate",
            Hint(..) => "Hint",
            IsEnabled(..) => "IsEnabled",
            LineWidth(..) => "LineWidth",
            PixelStorei(..) => "PixelStorei",
            LinkProgram(..) => "LinkProgram",
            Uniform1f(..) => "Uniform1f",
            Uniform1fv(..) => "Uniform1fv",
            Uniform1i(..) => "Uniform1i",
            Uniform1iv(..) => "Uniform1iv",
            Uniform2f(..) => "Uniform2f",
            Uniform2fv(..) => "Uniform2fv",
            Uniform2i(..) => "Uniform2i",
            Uniform2iv(..) => "Uniform2iv",
            Uniform3f(..) => "Uniform3f",
            Uniform3fv(..) => "Uniform3fv",
            Uniform3i(..) => "Uniform3i",
            Uniform3iv(..) => "Uniform3iv",
            Uniform4f(..) => "Uniform4f",
            Uniform4fv(..) => "Uniform4fv",
            Uniform4i(..) => "Uniform4i",
            Uniform4iv(..) => "Uniform4iv",
            UniformMatrix2fv(..) => "UniformMatrix2fv",
            UniformMatrix3fv(..) => "UniformMatrix3fv",
            UniformMatrix4fv(..) => "UniformMatrix4fv",
            UseProgram(..) => "UseProgram",
            ValidateProgram(..) => "ValidateProgram",
            VertexAttrib(..) => "VertexAttrib",
            VertexAttribPointer2f(..) => "VertexAttribPointer2f",
            VertexAttribPointer(..) => "VertexAttribPointer",
            SetViewport(..) => "SetViewport",
            TexImage2D(..) => "TexImage2D",
            TexParameteri(..) => "TexParameteri",
            TexParameterf(..) => "TexParameterf",
            TexSubImage2D(..) => "TexSubImage2D",
            DrawingBufferWidth(..) => "DrawingBufferWidth",
            DrawingBufferHeight(..) => "DrawingBufferHeight",
            Finish(..) => "Finish",
            Flush => "Flush",
            GenerateMipmap(..) => "GenerateMipmap",
            CreateVertexArray(..) => "CreateVertexArray",
            DeleteVertexArray(..) => "DeleteVertexArray",
            BindVertexArray(..) => "BindVertexArray",
            GetParameterBool(..) => "GetParameterBool",
            GetParameterInt(..) => "GetParameterInt",
            GetParameterInt4(..) => "GetParameterInt4",
            GetParameterFloat(..) => "GetParameterFloat",
            GetParameterFloat2(..) => "GetParameterFloat2",
            GetProgramParameterBool(..) => "GetProgramParameterBool",
            GetProgramParameterInt(..) => "GetProgramParameterInt",
            GetShaderParameterBool(..) => "GetShaderParameterBool",
            GetShaderParameterInt(..) => "GetShaderParameterInt",
            GetVertexAttribBool(..) => "GetVertexAttribBool",
            GetVertexAttribInt(..) => "GetVertexAttribInt",
            GetVertexAttribFloat4(..) => "GetVertexAttribFloat4",
        };

        write!(f, "CanvasWebGLMsg::{}(..)", name)
    }
}

macro_rules! parameters {
    ($name:ident { $(
        $variant:ident($kind:ident { $(
            $param:ident = gl::$value:ident,
        )+ }),
    )+ }) => {
        #[derive(Clone, Deserialize, Serialize)]
        pub enum $name { $(
            $variant($kind),
        )+}

        $(
            #[derive(Clone, Deserialize, Serialize)]
            #[repr(u32)]
            pub enum $kind { $(
                $param = gl::$value,
            )+}
        )+

        impl $name {
            pub fn from_u32(value: u32) -> WebGLResult<Self> {
                match value {
                    $($(gl::$value => Ok($name::$variant($kind::$param)),)+)+
                    _ => Err(WebGLError::InvalidEnum)
                }
            }
        }
    }
}

parameters! {
    Parameter {
        Bool(ParameterBool {
            Blend = gl::BLEND,
            CullFace = gl::CULL_FACE,
            DepthTest = gl::DEPTH_TEST,
            DepthWritemask = gl::DEPTH_WRITEMASK,
            Dither = gl::DITHER,
            PolygonOffsetFill = gl::POLYGON_OFFSET_FILL,
            SampleCoverageInvert = gl::SAMPLE_COVERAGE_INVERT,
            StencilTest = gl::STENCIL_TEST,
        }),
        Int(ParameterInt {
            ActiveTexture = gl::ACTIVE_TEXTURE,
            AlphaBits = gl::ALPHA_BITS,
            BlendDstAlpha = gl::BLEND_DST_ALPHA,
            BlendDstRgb = gl::BLEND_DST_RGB,
            BlendEquationAlpha = gl::BLEND_EQUATION_ALPHA,
            BlendEquationRgb = gl::BLEND_EQUATION_RGB,
            BlendSrcAlpha = gl::BLEND_SRC_ALPHA,
            BlendSrcRgb = gl::BLEND_SRC_RGB,
            BlueBits = gl::BLUE_BITS,
            CullFaceMode = gl::CULL_FACE_MODE,
            DepthBits = gl::DEPTH_BITS,
            DepthFunc = gl::DEPTH_FUNC,
            FrontFace = gl::FRONT_FACE,
            GreenBits = gl::GREEN_BITS,
            MaxCombinedTextureImageUnits = gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS,
            MaxCubeMapTextureSize = gl::MAX_CUBE_MAP_TEXTURE_SIZE,
            MaxRenderbufferSize = gl::MAX_RENDERBUFFER_SIZE,
            MaxTextureImageUnits = gl::MAX_TEXTURE_IMAGE_UNITS,
            MaxTextureSize = gl::MAX_TEXTURE_SIZE,
            MaxVertexAttribs = gl::MAX_VERTEX_ATTRIBS,
            MaxVertexTextureImageUnits = gl::MAX_VERTEX_TEXTURE_IMAGE_UNITS,
            PackAlignment = gl::PACK_ALIGNMENT,
            RedBits = gl::RED_BITS,
            SampleBuffers = gl::SAMPLE_BUFFERS,
            Samples = gl::SAMPLES,
            StencilBackFail = gl::STENCIL_BACK_FAIL,
            StencilBackFunc = gl::STENCIL_BACK_FUNC,
            StencilBackPassDepthFail = gl::STENCIL_BACK_PASS_DEPTH_FAIL,
            StencilBackPassDepthPass = gl::STENCIL_BACK_PASS_DEPTH_PASS,
            StencilBackRef = gl::STENCIL_BACK_REF,
            StencilBackValueMask = gl::STENCIL_BACK_VALUE_MASK,
            StencilBackWritemask = gl::STENCIL_BACK_WRITEMASK,
            StencilBits = gl::STENCIL_BITS,
            StencilClearValue = gl::STENCIL_CLEAR_VALUE,
            StencilFail = gl::STENCIL_FAIL,
            StencilFunc = gl::STENCIL_FUNC,
            StencilPassDepthFail = gl::STENCIL_PASS_DEPTH_FAIL,
            StencilPassDepthPass = gl::STENCIL_PASS_DEPTH_PASS,
            StencilRef = gl::STENCIL_REF,
            StencilValueMask = gl::STENCIL_VALUE_MASK,
            StencilWritemask = gl::STENCIL_WRITEMASK,
            SubpixelBits = gl::SUBPIXEL_BITS,
            UnpackAlignment = gl::UNPACK_ALIGNMENT,
            FragmentShaderDerivativeHint = gl::FRAGMENT_SHADER_DERIVATIVE_HINT,
        }),
        Int4(ParameterInt4 {
            Viewport = gl::VIEWPORT,
        }),
        Float(ParameterFloat {
            DepthClearValue = gl::DEPTH_CLEAR_VALUE,
            LineWidth = gl::LINE_WIDTH,
            PolygonOffsetFactor = gl::POLYGON_OFFSET_FACTOR,
            PolygonOffsetUnits = gl::POLYGON_OFFSET_UNITS,
            SampleCoverageValue = gl::SAMPLE_COVERAGE_VALUE,
        }),
        Float2(ParameterFloat2 {
            AliasedPointSizeRange = gl::ALIASED_POINT_SIZE_RANGE,
            AliasedLineWidthRange = gl::ALIASED_LINE_WIDTH_RANGE,
        }),
    }
}

parameters! {
    ProgramParameter {
        Bool(ProgramParameterBool {
            DeleteStatus = gl::DELETE_STATUS,
            LinkStatus = gl::LINK_STATUS,
            ValidateStatus = gl::VALIDATE_STATUS,
        }),
        Int(ProgramParameterInt {
            AttachedShaders = gl::ATTACHED_SHADERS,
            ActiveAttributes = gl::ACTIVE_ATTRIBUTES,
            ActiveUniforms = gl::ACTIVE_UNIFORMS,
        }),
    }
}

parameters! {
    ShaderParameter {
        Bool(ShaderParameterBool {
            DeleteStatus = gl::DELETE_STATUS,
            CompileStatus = gl::COMPILE_STATUS,
        }),
        Int(ShaderParameterInt {
            ShaderType = gl::SHADER_TYPE,
        }),
    }
}

parameters! {
    VertexAttrib {
        Bool(VertexAttribBool {
            VertexAttribArrayEnabled = gl::VERTEX_ATTRIB_ARRAY_ENABLED,
            VertexAttribArrayNormalized = gl::VERTEX_ATTRIB_ARRAY_NORMALIZED,
        }),
        Int(VertexAttribInt {
            VertexAttribArraySize = gl::VERTEX_ATTRIB_ARRAY_SIZE,
            VertexAttribArrayStride = gl::VERTEX_ATTRIB_ARRAY_STRIDE,
            VertexAttribArrayType = gl::VERTEX_ATTRIB_ARRAY_TYPE,
        }),
        Float4(VertexAttribFloat4 {
            CurrentVertexAttrib = gl::CURRENT_VERTEX_ATTRIB,
        }),
    }
}
