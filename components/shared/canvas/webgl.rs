/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::ops::Deref;

/// Receiver type used in WebGLCommands.
pub use base::generic_channel::GenericReceiver as WebGLReceiver;
/// Sender type used in WebGLCommands.
pub use base::generic_channel::GenericSender as WebGLSender;
/// Result type for send()/recv() calls in in WebGLCommands.
pub use base::generic_channel::SendResult as WebGLSendResult;
use euclid::default::{Rect, Size2D};
use ipc_channel::ipc::{IpcBytesReceiver, IpcBytesSender, IpcSharedMemory};
use malloc_size_of_derive::MallocSizeOf;
use pixels::PixelFormat;
use serde::{Deserialize, Serialize};
use sparkle::gl;
use webrender_api::ImageKey;
use webxr_api::{
    ContextId as WebXRContextId, Error as WebXRError, LayerId as WebXRLayerId,
    LayerInit as WebXRLayerInit, SubImages as WebXRSubImages,
};

/// Helper function that creates a WebGL channel (WebGLSender, WebGLReceiver) to be used in WebGLCommands.
pub fn webgl_channel<T>() -> Option<(WebGLSender<T>, WebGLReceiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    base::generic_channel::channel(servo_config::opts::multiprocess())
}

/// Entry point channel type used for sending WebGLMsg messages to the WebGL renderer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGLChan(pub WebGLSender<WebGLMsg>);

impl WebGLChan {
    #[inline]
    pub fn send(&self, msg: WebGLMsg) -> WebGLSendResult {
        self.0.send(msg)
    }
}

/// Entry point type used in a Script Pipeline to get the WebGLChan to be used in that thread.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGLPipeline(pub WebGLChan);

impl WebGLPipeline {
    pub fn channel(&self) -> WebGLChan {
        self.0.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGLCommandBacktrace {
    #[cfg(feature = "webgl_backtrace")]
    pub backtrace: String,
    #[cfg(feature = "webgl_backtrace")]
    pub js_backtrace: Option<String>,
}

/// WebGL Threading API entry point that lives in the constellation.
pub struct WebGLThreads(pub WebGLSender<WebGLMsg>);

impl WebGLThreads {
    /// Gets the WebGLThread handle for each script pipeline.
    pub fn pipeline(&self) -> WebGLPipeline {
        // This mode creates a single thread, so the existing WebGLChan is just cloned.
        WebGLPipeline(WebGLChan(self.0.clone()))
    }

    /// Sends a exit message to close the WebGLThreads and release all WebGLContexts.
    pub fn exit(&self) -> Result<(), &'static str> {
        self.0
            .send(WebGLMsg::Exit)
            .map_err(|_| "Failed to send Exit message")
    }
}

/// WebGL Message API
#[derive(Debug, Deserialize, Serialize)]
pub enum WebGLMsg {
    /// Creates a new WebGLContext.
    CreateContext(
        WebGLVersion,
        Size2D<u32>,
        GLContextAttributes,
        WebGLSender<Result<WebGLCreateContextResult, String>>,
    ),
    /// Resizes a WebGLContext.
    ResizeContext(WebGLContextId, Size2D<u32>, WebGLSender<Result<(), String>>),
    /// Drops a WebGLContext.
    RemoveContext(WebGLContextId),
    /// Runs a WebGLCommand in a specific WebGLContext.
    WebGLCommand(WebGLContextId, WebGLCommand, WebGLCommandBacktrace),
    /// Runs a WebXRCommand (WebXR layers need to be created in the WebGL
    /// thread, as they may have thread affinity).
    WebXRCommand(WebXRCommand),
    /// Performs a buffer swap.
    ///
    /// The third field contains the time (in ns) when the request
    /// was initiated. The u64 in the second field will be the time the
    /// request is fulfilled
    SwapBuffers(Vec<WebGLContextId>, WebGLSender<u64>, u64),
    /// Frees all resources and closes the thread.
    Exit,
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum GlType {
    Gl,
    Gles,
}

/// Contains the WebGLCommand sender and information about a WebGLContext
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGLCreateContextResult {
    /// Sender instance to send commands to the specific WebGLContext
    pub sender: WebGLMsgSender,
    /// Information about the internal GL Context.
    pub limits: GLLimits,
    /// The GLSL version supported by the context.
    pub glsl_version: WebGLSLVersion,
    /// The GL API used by the context.
    pub api_type: GlType,
    /// The WebRender image key.
    pub image_key: ImageKey,
}

/// Defines the WebGL version
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, PartialOrd, Serialize)]
pub enum WebGLVersion {
    /// <https://www.khronos.org/registry/webgl/specs/1.0.2/>
    /// Conforms closely to the OpenGL ES 2.0 API
    WebGL1,
    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/>
    /// Conforms closely to the OpenGL ES 3.0 API
    WebGL2,
}

/// Defines the GLSL version supported by the WebGL backend contexts.
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct WebGLSLVersion {
    /// Major GLSL version
    pub major: u32,
    /// Minor GLSL version
    pub minor: u32,
}

/// Helper struct to send WebGLCommands to a specific WebGLContext.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct WebGLMsgSender {
    ctx_id: WebGLContextId,
    #[ignore_malloc_size_of = "channels are hard"]
    sender: WebGLChan,
}

impl WebGLMsgSender {
    pub fn new(id: WebGLContextId, sender: WebGLChan) -> Self {
        WebGLMsgSender { ctx_id: id, sender }
    }

    /// Returns the WebGLContextId associated to this sender
    pub fn context_id(&self) -> WebGLContextId {
        self.ctx_id
    }

    /// Send a WebGLCommand message
    #[inline]
    pub fn send(&self, command: WebGLCommand, backtrace: WebGLCommandBacktrace) -> WebGLSendResult {
        self.sender
            .send(WebGLMsg::WebGLCommand(self.ctx_id, command, backtrace))
    }

    /// Send a resize message
    #[inline]
    pub fn send_resize(
        &self,
        size: Size2D<u32>,
        sender: WebGLSender<Result<(), String>>,
    ) -> WebGLSendResult {
        self.sender
            .send(WebGLMsg::ResizeContext(self.ctx_id, size, sender))
    }

    #[inline]
    pub fn send_remove(&self) -> WebGLSendResult {
        self.sender.send(WebGLMsg::RemoveContext(self.ctx_id))
    }
}

#[derive(Deserialize, Serialize)]
pub struct TruncatedDebug<T>(T);

impl<T> From<T> for TruncatedDebug<T> {
    fn from(v: T) -> TruncatedDebug<T> {
        TruncatedDebug(v)
    }
}

impl<T: fmt::Debug> fmt::Debug for TruncatedDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = format!("{:?}", self.0);
        if s.len() > 20 {
            s.truncate(20);
            s.push_str("...");
        }
        write!(f, "{}", s)
    }
}

impl<T> Deref for TruncatedDebug<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

/// WebGL Commands for a specific WebGLContext
#[derive(Debug, Deserialize, Serialize)]
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
    BufferData(u32, IpcBytesReceiver, u32),
    BufferSubData(u32, isize, IpcBytesReceiver),
    GetBufferSubData(u32, usize, usize, IpcBytesSender),
    CopyBufferSubData(u32, u32, i64, i64, i64),
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
    EnableVertexAttribArray(u32),
    FramebufferRenderbuffer(u32, u32, u32, Option<WebGLRenderbufferId>),
    FramebufferTexture2D(u32, u32, u32, Option<WebGLTextureId>, i32),
    GetExtensions(WebGLSender<String>),
    GetShaderPrecisionFormat(u32, u32, WebGLSender<(i32, i32, i32)>),
    GetFragDataLocation(WebGLProgramId, String, WebGLSender<i32>),
    GetUniformLocation(WebGLProgramId, String, WebGLSender<i32>),
    GetShaderInfoLog(WebGLShaderId, WebGLSender<String>),
    GetProgramInfoLog(WebGLProgramId, WebGLSender<String>),
    GetFramebufferAttachmentParameter(u32, u32, u32, WebGLSender<i32>),
    GetRenderbufferParameter(u32, u32, WebGLSender<i32>),
    CreateTransformFeedback(WebGLSender<u32>),
    DeleteTransformFeedback(u32),
    IsTransformFeedback(u32, WebGLSender<bool>),
    BindTransformFeedback(u32, u32),
    BeginTransformFeedback(u32),
    EndTransformFeedback(),
    PauseTransformFeedback(),
    ResumeTransformFeedback(),
    GetTransformFeedbackVarying(WebGLProgramId, u32, WebGLSender<(i32, u32, String)>),
    TransformFeedbackVaryings(WebGLProgramId, Vec<String>, u32),
    PolygonOffset(f32, f32),
    RenderbufferStorage(u32, u32, i32, i32),
    RenderbufferStorageMultisample(u32, i32, u32, i32, i32),
    ReadPixels(Rect<u32>, u32, u32, IpcBytesSender),
    ReadPixelsPP(Rect<i32>, u32, u32, usize),
    SampleCoverage(f32, bool),
    Scissor(i32, i32, u32, u32),
    StencilFunc(u32, i32, u32),
    StencilFuncSeparate(u32, u32, i32, u32),
    StencilMask(u32),
    StencilMaskSeparate(u32, u32),
    StencilOp(u32, u32, u32),
    StencilOpSeparate(u32, u32, u32, u32),
    FenceSync(WebGLSender<WebGLSyncId>),
    IsSync(WebGLSyncId, WebGLSender<bool>),
    ClientWaitSync(WebGLSyncId, u32, u64, WebGLSender<u32>),
    WaitSync(WebGLSyncId, u32, i64),
    GetSyncParameter(WebGLSyncId, u32, WebGLSender<u32>),
    DeleteSync(WebGLSyncId),
    Hint(u32, u32),
    LineWidth(f32),
    PixelStorei(u32, i32),
    LinkProgram(WebGLProgramId, WebGLSender<ProgramLinkInfo>),
    Uniform1f(i32, f32),
    Uniform1fv(i32, Vec<f32>),
    Uniform1i(i32, i32),
    Uniform1ui(i32, u32),
    Uniform1iv(i32, Vec<i32>),
    Uniform1uiv(i32, Vec<u32>),
    Uniform2f(i32, f32, f32),
    Uniform2fv(i32, Vec<f32>),
    Uniform2i(i32, i32, i32),
    Uniform2ui(i32, u32, u32),
    Uniform2iv(i32, Vec<i32>),
    Uniform2uiv(i32, Vec<u32>),
    Uniform3f(i32, f32, f32, f32),
    Uniform3fv(i32, Vec<f32>),
    Uniform3i(i32, i32, i32, i32),
    Uniform3ui(i32, u32, u32, u32),
    Uniform3iv(i32, Vec<i32>),
    Uniform3uiv(i32, Vec<u32>),
    Uniform4f(i32, f32, f32, f32, f32),
    Uniform4fv(i32, Vec<f32>),
    Uniform4i(i32, i32, i32, i32, i32),
    Uniform4ui(i32, u32, u32, u32, u32),
    Uniform4iv(i32, Vec<i32>),
    Uniform4uiv(i32, Vec<u32>),
    UniformMatrix2fv(i32, Vec<f32>),
    UniformMatrix3fv(i32, Vec<f32>),
    UniformMatrix4fv(i32, Vec<f32>),
    UniformMatrix3x2fv(i32, Vec<f32>),
    UniformMatrix4x2fv(i32, Vec<f32>),
    UniformMatrix2x3fv(i32, Vec<f32>),
    UniformMatrix4x3fv(i32, Vec<f32>),
    UniformMatrix2x4fv(i32, Vec<f32>),
    UniformMatrix3x4fv(i32, Vec<f32>),
    UseProgram(Option<WebGLProgramId>),
    ValidateProgram(WebGLProgramId),
    VertexAttrib(u32, f32, f32, f32, f32),
    VertexAttribI(u32, i32, i32, i32, i32),
    VertexAttribU(u32, u32, u32, u32, u32),
    VertexAttribPointer(u32, i32, u32, bool, i32, u32),
    VertexAttribPointer2f(u32, i32, bool, i32, u32),
    SetViewport(i32, i32, i32, i32),
    TexImage2D {
        target: u32,
        level: u32,
        internal_format: TexFormat,
        size: Size2D<u32>,
        format: TexFormat,
        data_type: TexDataType,
        // FIXME(nox): This should be computed on the WebGL thread.
        effective_data_type: u32,
        unpacking_alignment: u32,
        alpha_treatment: Option<AlphaTreatment>,
        y_axis_treatment: YAxisTreatment,
        pixel_format: Option<PixelFormat>,
        data: TruncatedDebug<IpcSharedMemory>,
    },
    TexImage2DPBO {
        target: u32,
        level: u32,
        internal_format: TexFormat,
        size: Size2D<u32>,
        format: TexFormat,
        effective_data_type: u32,
        unpacking_alignment: u32,
        offset: i64,
    },
    TexSubImage2D {
        target: u32,
        level: u32,
        xoffset: i32,
        yoffset: i32,
        size: Size2D<u32>,
        format: TexFormat,
        data_type: TexDataType,
        // FIXME(nox): This should be computed on the WebGL thread.
        effective_data_type: u32,
        unpacking_alignment: u32,
        alpha_treatment: Option<AlphaTreatment>,
        y_axis_treatment: YAxisTreatment,
        pixel_format: Option<PixelFormat>,
        data: TruncatedDebug<IpcSharedMemory>,
    },
    CompressedTexImage2D {
        target: u32,
        level: u32,
        internal_format: u32,
        size: Size2D<u32>,
        data: TruncatedDebug<IpcSharedMemory>,
    },
    CompressedTexSubImage2D {
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        size: Size2D<u32>,
        format: u32,
        data: TruncatedDebug<IpcSharedMemory>,
    },
    DrawingBufferWidth(WebGLSender<i32>),
    DrawingBufferHeight(WebGLSender<i32>),
    Finish(WebGLSender<()>),
    Flush,
    GenerateMipmap(u32),
    CreateVertexArray(WebGLSender<Option<WebGLVertexArrayId>>),
    DeleteVertexArray(WebGLVertexArrayId),
    BindVertexArray(Option<WebGLVertexArrayId>),
    GetParameterBool(ParameterBool, WebGLSender<bool>),
    GetParameterBool4(ParameterBool4, WebGLSender<[bool; 4]>),
    GetParameterInt(ParameterInt, WebGLSender<i32>),
    GetParameterInt2(ParameterInt2, WebGLSender<[i32; 2]>),
    GetParameterInt4(ParameterInt4, WebGLSender<[i32; 4]>),
    GetParameterFloat(ParameterFloat, WebGLSender<f32>),
    GetParameterFloat2(ParameterFloat2, WebGLSender<[f32; 2]>),
    GetParameterFloat4(ParameterFloat4, WebGLSender<[f32; 4]>),
    GetProgramValidateStatus(WebGLProgramId, WebGLSender<bool>),
    GetProgramActiveUniforms(WebGLProgramId, WebGLSender<i32>),
    GetCurrentVertexAttrib(u32, WebGLSender<[f32; 4]>),
    GetTexParameterFloat(u32, TexParameterFloat, WebGLSender<f32>),
    GetTexParameterInt(u32, TexParameterInt, WebGLSender<i32>),
    GetTexParameterBool(u32, TexParameterBool, WebGLSender<bool>),
    GetInternalFormatIntVec(u32, u32, InternalFormatIntVec, WebGLSender<Vec<i32>>),
    TexParameteri(u32, u32, i32),
    TexParameterf(u32, u32, f32),
    TexStorage2D(u32, u32, TexFormat, u32, u32),
    TexStorage3D(u32, u32, TexFormat, u32, u32, u32),
    DrawArrays {
        mode: u32,
        first: i32,
        count: i32,
    },
    DrawArraysInstanced {
        mode: u32,
        first: i32,
        count: i32,
        primcount: i32,
    },
    DrawElements {
        mode: u32,
        count: i32,
        type_: u32,
        offset: u32,
    },
    DrawElementsInstanced {
        mode: u32,
        count: i32,
        type_: u32,
        offset: u32,
        primcount: i32,
    },
    VertexAttribDivisor {
        index: u32,
        divisor: u32,
    },
    GetUniformBool(WebGLProgramId, i32, WebGLSender<bool>),
    GetUniformBool2(WebGLProgramId, i32, WebGLSender<[bool; 2]>),
    GetUniformBool3(WebGLProgramId, i32, WebGLSender<[bool; 3]>),
    GetUniformBool4(WebGLProgramId, i32, WebGLSender<[bool; 4]>),
    GetUniformInt(WebGLProgramId, i32, WebGLSender<i32>),
    GetUniformInt2(WebGLProgramId, i32, WebGLSender<[i32; 2]>),
    GetUniformInt3(WebGLProgramId, i32, WebGLSender<[i32; 3]>),
    GetUniformInt4(WebGLProgramId, i32, WebGLSender<[i32; 4]>),
    GetUniformUint(WebGLProgramId, i32, WebGLSender<u32>),
    GetUniformUint2(WebGLProgramId, i32, WebGLSender<[u32; 2]>),
    GetUniformUint3(WebGLProgramId, i32, WebGLSender<[u32; 3]>),
    GetUniformUint4(WebGLProgramId, i32, WebGLSender<[u32; 4]>),
    GetUniformFloat(WebGLProgramId, i32, WebGLSender<f32>),
    GetUniformFloat2(WebGLProgramId, i32, WebGLSender<[f32; 2]>),
    GetUniformFloat3(WebGLProgramId, i32, WebGLSender<[f32; 3]>),
    GetUniformFloat4(WebGLProgramId, i32, WebGLSender<[f32; 4]>),
    GetUniformFloat9(WebGLProgramId, i32, WebGLSender<[f32; 9]>),
    GetUniformFloat16(WebGLProgramId, i32, WebGLSender<[f32; 16]>),
    GetUniformFloat2x3(WebGLProgramId, i32, WebGLSender<[f32; 2 * 3]>),
    GetUniformFloat2x4(WebGLProgramId, i32, WebGLSender<[f32; 2 * 4]>),
    GetUniformFloat3x2(WebGLProgramId, i32, WebGLSender<[f32; 3 * 2]>),
    GetUniformFloat3x4(WebGLProgramId, i32, WebGLSender<[f32; 3 * 4]>),
    GetUniformFloat4x2(WebGLProgramId, i32, WebGLSender<[f32; 4 * 2]>),
    GetUniformFloat4x3(WebGLProgramId, i32, WebGLSender<[f32; 4 * 3]>),
    GetUniformBlockIndex(WebGLProgramId, String, WebGLSender<u32>),
    GetUniformIndices(WebGLProgramId, Vec<String>, WebGLSender<Vec<u32>>),
    GetActiveUniforms(WebGLProgramId, Vec<u32>, u32, WebGLSender<Vec<i32>>),
    GetActiveUniformBlockName(WebGLProgramId, u32, WebGLSender<String>),
    GetActiveUniformBlockParameter(WebGLProgramId, u32, u32, WebGLSender<Vec<i32>>),
    UniformBlockBinding(WebGLProgramId, u32, u32),
    InitializeFramebuffer {
        color: bool,
        depth: bool,
        stencil: bool,
    },
    BeginQuery(u32, WebGLQueryId),
    DeleteQuery(WebGLQueryId),
    EndQuery(u32),
    GenerateQuery(WebGLSender<WebGLQueryId>),
    GetQueryState(WebGLSender<u32>, WebGLQueryId, u32),
    GenerateSampler(WebGLSender<WebGLSamplerId>),
    DeleteSampler(WebGLSamplerId),
    BindSampler(u32, WebGLSamplerId),
    SetSamplerParameterFloat(WebGLSamplerId, u32, f32),
    SetSamplerParameterInt(WebGLSamplerId, u32, i32),
    GetSamplerParameterFloat(WebGLSamplerId, u32, WebGLSender<f32>),
    GetSamplerParameterInt(WebGLSamplerId, u32, WebGLSender<i32>),
    BindBufferBase(u32, u32, Option<WebGLBufferId>),
    BindBufferRange(u32, u32, Option<WebGLBufferId>, i64, i64),
    ClearBufferfv(u32, i32, Vec<f32>),
    ClearBufferiv(u32, i32, Vec<i32>),
    ClearBufferuiv(u32, i32, Vec<u32>),
    ClearBufferfi(u32, i32, f32, i32),
    InvalidateFramebuffer(u32, Vec<u32>),
    InvalidateSubFramebuffer(u32, Vec<u32>, i32, i32, i32, i32),
    FramebufferTextureLayer(u32, u32, Option<WebGLTextureId>, i32, i32),
    ReadBuffer(u32),
    DrawBuffers(Vec<u32>),
}

/// WebXR layer management
#[derive(Debug, Deserialize, Serialize)]
pub enum WebXRCommand {
    CreateLayerManager(WebGLSender<Result<WebXRLayerManagerId, WebXRError>>),
    DestroyLayerManager(WebXRLayerManagerId),
    CreateLayer(
        WebXRLayerManagerId,
        WebXRContextId,
        WebXRLayerInit,
        WebGLSender<Result<WebXRLayerId, WebXRError>>,
    ),
    DestroyLayer(WebXRLayerManagerId, WebXRContextId, WebXRLayerId),
    BeginFrame(
        WebXRLayerManagerId,
        Vec<(WebXRContextId, WebXRLayerId)>,
        WebGLSender<Result<Vec<WebXRSubImages>, WebXRError>>,
    ),
    EndFrame(
        WebXRLayerManagerId,
        Vec<(WebXRContextId, WebXRLayerId)>,
        WebGLSender<Result<(), WebXRError>>,
    ),
}

macro_rules! nonzero_type {
    (u32) => {
        NonZeroU32
    };
    (u64) => {
        NonZeroU64
    };
}

macro_rules! define_resource_id {
    ($name:ident, $type:tt) => {
        #[derive(Clone, Copy, Eq, Hash, PartialEq)]
        pub struct $name(nonzero_type!($type));

        impl $name {
            #[allow(unsafe_code)]
            #[inline]
            /// Create a new $name.
            ///
            /// # Safety
            ///
            /// Using an invalid OpenGL id may result in undefined behavior.
            pub unsafe fn new(id: $type) -> Self {
                $name(<nonzero_type!($type)>::new_unchecked(id))
            }

            #[inline]
            pub fn maybe_new(id: $type) -> Option<Self> {
                <nonzero_type!($type)>::new(id).map($name)
            }

            #[inline]
            pub fn get(self) -> $type {
                self.0.get()
            }
        }

        #[allow(unsafe_code)]
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let id = <$type>::deserialize(deserializer)?;
                if id == 0 {
                    Err(::serde::de::Error::custom("expected a non-zero value"))
                } else {
                    Ok(unsafe { $name::new(id) })
                }
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                self.get().serialize(serializer)
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                fmt.debug_tuple(stringify!($name))
                    .field(&self.get())
                    .finish()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(fmt, "{}", self.get())
            }
        }

        impl ::malloc_size_of::MallocSizeOf for $name {
            fn size_of(&self, _ops: &mut ::malloc_size_of::MallocSizeOfOps) -> usize {
                0
            }
        }
    };
}

define_resource_id!(WebGLBufferId, u32);
define_resource_id!(WebGLFramebufferId, u32);
define_resource_id!(WebGLRenderbufferId, u32);
define_resource_id!(WebGLTextureId, u32);
define_resource_id!(WebGLProgramId, u32);
define_resource_id!(WebGLQueryId, u32);
define_resource_id!(WebGLSamplerId, u32);
define_resource_id!(WebGLShaderId, u32);
define_resource_id!(WebGLSyncId, u64);
define_resource_id!(WebGLVertexArrayId, u32);
define_resource_id!(WebXRLayerManagerId, u32);

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct WebGLContextId(pub u64);

impl From<WebXRContextId> for WebGLContextId {
    fn from(id: WebXRContextId) -> Self {
        Self(id.0)
    }
}

impl From<WebGLContextId> for WebXRContextId {
    fn from(id: WebGLContextId) -> Self {
        Self(id.0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum WebGLError {
    InvalidEnum,
    InvalidFramebufferOperation,
    InvalidOperation,
    InvalidValue,
    OutOfMemory,
    ContextLost,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WebGLFramebufferBindingRequest {
    Explicit(WebGLFramebufferId),
    Default,
}

pub type WebGLResult<T> = Result<T, WebGLError>;

/// Information about a WebGL program linking operation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProgramLinkInfo {
    /// Whether the program was linked successfully.
    pub linked: bool,
    /// The list of active attributes.
    pub active_attribs: Box<[ActiveAttribInfo]>,
    /// The list of active uniforms.
    pub active_uniforms: Box<[ActiveUniformInfo]>,
    /// The list of active uniform blocks.
    pub active_uniform_blocks: Box<[ActiveUniformBlockInfo]>,
    /// The number of varying variables
    pub transform_feedback_length: i32,
    /// The buffer mode used when transform feedback is active
    pub transform_feedback_mode: i32,
}

/// Description of a single active attribute.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ActiveAttribInfo {
    /// The name of the attribute.
    pub name: String,
    /// The size of the attribute.
    pub size: i32,
    /// The type of the attribute.
    pub type_: u32,
    /// The location of the attribute.
    pub location: i32,
}

/// Description of a single active uniform.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ActiveUniformInfo {
    /// The base name of the uniform.
    pub base_name: Box<str>,
    /// The size of the uniform, if it is an array.
    pub size: Option<i32>,
    /// The type of the uniform.
    pub type_: u32,
    /// The index of the indexed uniform buffer binding, if it is bound.
    pub bind_index: Option<u32>,
}

impl ActiveUniformInfo {
    pub fn name(&self) -> Cow<str> {
        if self.size.is_some() {
            let mut name = String::from(&*self.base_name);
            name.push_str("[0]");
            Cow::Owned(name)
        } else {
            Cow::Borrowed(&self.base_name)
        }
    }
}

/// Description of a single uniform block.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ActiveUniformBlockInfo {
    /// The name of the uniform block.
    pub name: String,
    /// The size of the uniform block.
    pub size: i32,
}

macro_rules! parameters {
    ($name:ident { $(
        $variant:ident($kind:ident { $(
            $param:ident = gl::$value:ident,
        )+ }),
    )+ }) => {
        #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
        pub enum $name { $(
            $variant($kind),
        )+}

        $(
            #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
            DepthWritemask = gl::DEPTH_WRITEMASK,
            SampleCoverageInvert = gl::SAMPLE_COVERAGE_INVERT,
            TransformFeedbackActive = gl::TRANSFORM_FEEDBACK_ACTIVE,
            TransformFeedbackPaused = gl::TRANSFORM_FEEDBACK_PAUSED,
            RasterizerDiscard = gl::RASTERIZER_DISCARD,
        }),
        Bool4(ParameterBool4 {
            ColorWritemask = gl::COLOR_WRITEMASK,
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
            FragmentShaderDerivativeHint = gl::FRAGMENT_SHADER_DERIVATIVE_HINT,
            FrontFace = gl::FRONT_FACE,
            GenerateMipmapHint = gl::GENERATE_MIPMAP_HINT,
            GreenBits = gl::GREEN_BITS,
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
            TransformFeedbackBinding = gl::TRANSFORM_FEEDBACK_BINDING,
            MaxTransformFeedbackInterleavedComponents = gl::MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS,
            MaxTransformFeedbackSeparateAttribs = gl::MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS,
            MaxTransformFeedbackSeparateComponents = gl::MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS,
            TransformFeedbackBufferSize = gl::TRANSFORM_FEEDBACK_BUFFER_SIZE,
            TransformFeedbackBufferStart = gl::TRANSFORM_FEEDBACK_BUFFER_START,
            PackRowLength = gl::PACK_ROW_LENGTH,
            PackSkipPixels = gl::PACK_SKIP_PIXELS,
            PackSkipRows = gl::PACK_SKIP_ROWS,
            UnpackImageHeight = gl::UNPACK_IMAGE_HEIGHT,
            UnpackRowLength = gl::UNPACK_ROW_LENGTH,
            UnpackSkipImages = gl::UNPACK_SKIP_IMAGES,
            UnpackSkipPixels = gl::UNPACK_SKIP_PIXELS,
            UnpackSkipRows = gl::UNPACK_SKIP_ROWS,
        }),
        Int2(ParameterInt2 {
            MaxViewportDims = gl::MAX_VIEWPORT_DIMS,
        }),
        Int4(ParameterInt4 {
            ScissorBox = gl::SCISSOR_BOX,
            Viewport = gl::VIEWPORT,
        }),
        Float(ParameterFloat {
            DepthClearValue = gl::DEPTH_CLEAR_VALUE,
            LineWidth = gl::LINE_WIDTH,
            MaxTextureMaxAnisotropyExt = gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT,
            PolygonOffsetFactor = gl::POLYGON_OFFSET_FACTOR,
            PolygonOffsetUnits = gl::POLYGON_OFFSET_UNITS,
            SampleCoverageValue = gl::SAMPLE_COVERAGE_VALUE,
        }),
        Float2(ParameterFloat2 {
            AliasedPointSizeRange = gl::ALIASED_POINT_SIZE_RANGE,
            AliasedLineWidthRange = gl::ALIASED_LINE_WIDTH_RANGE,
            DepthRange = gl::DEPTH_RANGE,
        }),
        Float4(ParameterFloat4 {
            BlendColor = gl::BLEND_COLOR,
            ColorClearValue = gl::COLOR_CLEAR_VALUE,
        }),
    }
}

parameters! {
    TexParameter {
        Float(TexParameterFloat {
            TextureMaxAnisotropyExt = gl::TEXTURE_MAX_ANISOTROPY_EXT,
            TextureMaxLod = gl::TEXTURE_MAX_LOD,
            TextureMinLod = gl::TEXTURE_MIN_LOD,
        }),
        Int(TexParameterInt {
            TextureWrapS = gl::TEXTURE_WRAP_S,
            TextureWrapT = gl::TEXTURE_WRAP_T,
            TextureWrapR = gl::TEXTURE_WRAP_R,
            TextureBaseLevel = gl::TEXTURE_BASE_LEVEL,
            TextureMinFilter = gl::TEXTURE_MIN_FILTER,
            TextureMagFilter = gl::TEXTURE_MAG_FILTER,
            TextureMaxLevel = gl::TEXTURE_MAX_LEVEL,
            TextureCompareFunc = gl::TEXTURE_COMPARE_FUNC,
            TextureCompareMode = gl::TEXTURE_COMPARE_MODE,
            TextureImmutableLevels = gl::TEXTURE_IMMUTABLE_LEVELS,
        }),
        Bool(TexParameterBool {
            TextureImmutableFormat = gl::TEXTURE_IMMUTABLE_FORMAT,
        }),
    }
}

impl TexParameter {
    pub fn required_webgl_version(self) -> WebGLVersion {
        match self {
            Self::Float(TexParameterFloat::TextureMaxAnisotropyExt) |
            Self::Int(TexParameterInt::TextureWrapS) |
            Self::Int(TexParameterInt::TextureWrapT) => WebGLVersion::WebGL1,
            _ => WebGLVersion::WebGL2,
        }
    }
}

parameters! {
    InternalFormatParameter {
        IntVec(InternalFormatIntVec {
            Samples = gl::SAMPLES,
        }),
    }
}

#[macro_export]
macro_rules! gl_enums {
    ($(pub enum $name:ident { $($variant:ident = $mod:ident::$constant:ident,)+ })*) => {
        $(
            #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, malloc_size_of_derive::MallocSizeOf)]
            #[derive(PartialEq, Serialize)]
            #[repr(u32)]
            pub enum $name { $($variant = $mod::$constant,)+ }

            impl $name {
                pub fn from_gl_constant(constant: u32) -> Option<Self> {
                    Some(match constant {
                        $($mod::$constant => $name::$variant, )+
                        _ => return None,
                    })
                }

                #[inline]
                pub fn as_gl_constant(&self) -> u32 {
                    *self as u32
                }
            }
        )*
    }
}

// FIXME: These should come from sparkle
mod gl_ext_constants {
    use sparkle::gl::types::GLenum;

    pub const COMPRESSED_RGB_S3TC_DXT1_EXT: GLenum = 0x83F0;
    pub const COMPRESSED_RGBA_S3TC_DXT1_EXT: GLenum = 0x83F1;
    pub const COMPRESSED_RGBA_S3TC_DXT3_EXT: GLenum = 0x83F2;
    pub const COMPRESSED_RGBA_S3TC_DXT5_EXT: GLenum = 0x83F3;
    pub const COMPRESSED_RGB_ETC1_WEBGL: GLenum = 0x8D64;

    pub static COMPRESSIONS: &[GLenum] = &[
        COMPRESSED_RGB_S3TC_DXT1_EXT,
        COMPRESSED_RGBA_S3TC_DXT1_EXT,
        COMPRESSED_RGBA_S3TC_DXT3_EXT,
        COMPRESSED_RGBA_S3TC_DXT5_EXT,
        COMPRESSED_RGB_ETC1_WEBGL,
    ];

    pub const ALPHA16F_ARB: u32 = 0x881C;
    pub const ALPHA32F_ARB: u32 = 0x8816;
    pub const LUMINANCE16F_ARB: u32 = 0x881E;
    pub const LUMINANCE32F_ARB: u32 = 0x8818;
    pub const LUMINANCE_ALPHA16F_ARB: u32 = 0x881F;
    pub const LUMINANCE_ALPHA32F_ARB: u32 = 0x8819;
}

gl_enums! {
    pub enum TexFormat {
        DepthComponent = gl::DEPTH_COMPONENT,
        DepthStencil = gl::DEPTH_STENCIL,
        Alpha = gl::ALPHA,
        Alpha32f = gl_ext_constants::ALPHA32F_ARB,
        Alpha16f = gl_ext_constants::ALPHA16F_ARB,
        Red = gl::RED,
        RedInteger = gl::RED_INTEGER,
        RG = gl::RG,
        RGInteger = gl::RG_INTEGER,
        RGB = gl::RGB,
        RGBInteger = gl::RGB_INTEGER,
        RGBA = gl::RGBA,
        RGBAInteger = gl::RGBA_INTEGER,
        Luminance = gl::LUMINANCE,
        LuminanceAlpha = gl::LUMINANCE_ALPHA,
        Luminance32f = gl_ext_constants::LUMINANCE32F_ARB,
        Luminance16f = gl_ext_constants::LUMINANCE16F_ARB,
        LuminanceAlpha32f = gl_ext_constants::LUMINANCE_ALPHA32F_ARB,
        LuminanceAlpha16f = gl_ext_constants::LUMINANCE_ALPHA16F_ARB,
        CompressedRgbS3tcDxt1 = gl_ext_constants::COMPRESSED_RGB_S3TC_DXT1_EXT,
        CompressedRgbaS3tcDxt1 = gl_ext_constants::COMPRESSED_RGBA_S3TC_DXT1_EXT,
        CompressedRgbaS3tcDxt3 = gl_ext_constants::COMPRESSED_RGBA_S3TC_DXT3_EXT,
        CompressedRgbaS3tcDxt5 = gl_ext_constants::COMPRESSED_RGBA_S3TC_DXT5_EXT,
        CompressedRgbEtc1 = gl_ext_constants::COMPRESSED_RGB_ETC1_WEBGL,
        R8 = gl::R8,
        R8SNorm = gl::R8_SNORM,
        R16f = gl::R16F,
        R32f = gl::R32F,
        R8ui = gl::R8UI,
        R8i = gl::R8I,
        R16ui = gl::R16UI,
        R16i = gl::R16I,
        R32ui = gl::R32UI,
        R32i = gl::R32I,
        RG8 = gl::RG8,
        RG8SNorm = gl::RG8_SNORM,
        RG16f = gl::RG16F,
        RG32f = gl::RG32F,
        RG8ui = gl::RG8UI,
        RG8i = gl::RG8I,
        RG16ui = gl::RG16UI,
        RG16i = gl::RG16I,
        RG32ui = gl::RG32UI,
        RG32i = gl::RG32I,
        RGB8 = gl::RGB8,
        SRGB8 = gl::SRGB8,
        RGB565 = gl::RGB565,
        RGB8SNorm = gl::RGB8_SNORM,
        R11fG11fB10f = gl::R11F_G11F_B10F,
        RGB9E5 = gl::RGB9_E5,
        RGB16f = gl::RGB16F,
        RGB32f = gl::RGB32F,
        RGB8ui = gl::RGB8UI,
        RGB8i = gl::RGB8I,
        RGB16ui = gl::RGB16UI,
        RGB16i = gl::RGB16I,
        RGB32ui = gl::RGB32UI,
        RGB32i = gl::RGB32I,
        RGBA8 = gl::RGBA8,
        SRGB8Alpha8 = gl::SRGB8_ALPHA8,
        RGBA8SNorm = gl::RGBA8_SNORM,
        RGB5A1 = gl::RGB5_A1,
        RGBA4 = gl::RGBA4,
        RGB10A2 = gl::RGB10_A2,
        RGBA16f = gl::RGBA16F,
        RGBA32f = gl::RGBA32F,
        RGBA8ui = gl::RGBA8UI,
        RGBA8i = gl::RGBA8I,
        RGB10A2ui = gl::RGB10_A2UI,
        RGBA16ui = gl::RGBA16UI,
        RGBA16i = gl::RGBA16I,
        RGBA32i = gl::RGBA32I,
        RGBA32ui = gl::RGBA32UI,
        DepthComponent16 = gl::DEPTH_COMPONENT16,
        DepthComponent24 = gl::DEPTH_COMPONENT24,
        DepthComponent32f = gl::DEPTH_COMPONENT32F,
        Depth24Stencil8 = gl::DEPTH24_STENCIL8,
        Depth32fStencil8 = gl::DEPTH32F_STENCIL8,
    }

    pub enum TexDataType {
        Byte = gl::BYTE,
        Int = gl::INT,
        Short = gl::SHORT,
        UnsignedByte = gl::UNSIGNED_BYTE,
        UnsignedInt = gl::UNSIGNED_INT,
        UnsignedInt10f11f11fRev = gl::UNSIGNED_INT_10F_11F_11F_REV,
        UnsignedInt2101010Rev = gl::UNSIGNED_INT_2_10_10_10_REV,
        UnsignedInt5999Rev = gl::UNSIGNED_INT_5_9_9_9_REV,
        UnsignedInt248 = gl::UNSIGNED_INT_24_8,
        UnsignedShort = gl::UNSIGNED_SHORT,
        UnsignedShort4444 = gl::UNSIGNED_SHORT_4_4_4_4,
        UnsignedShort5551 = gl::UNSIGNED_SHORT_5_5_5_1,
        UnsignedShort565 = gl::UNSIGNED_SHORT_5_6_5,
        Float = gl::FLOAT,
        HalfFloat = gl::HALF_FLOAT_OES,
        Float32UnsignedInt248Rev = gl::FLOAT_32_UNSIGNED_INT_24_8_REV,
    }
}

impl TexFormat {
    /// Returns how many components does this format need. For example, RGBA
    /// needs 4 components, while RGB requires 3.
    pub fn components(&self) -> u32 {
        match self.to_unsized() {
            TexFormat::DepthStencil => 2,
            TexFormat::LuminanceAlpha => 2,
            TexFormat::RG | TexFormat::RGInteger => 2,
            TexFormat::RGB | TexFormat::RGBInteger => 3,
            TexFormat::RGBA | TexFormat::RGBAInteger => 4,
            _ => 1,
        }
    }

    /// Returns whether this format is a known texture compression format.
    pub fn is_compressed(&self) -> bool {
        gl_ext_constants::COMPRESSIONS.contains(&self.as_gl_constant())
    }

    /// Returns whether this format is a known sized or unsized format.
    pub fn is_sized(&self) -> bool {
        !matches!(
            self,
            TexFormat::DepthComponent |
                TexFormat::DepthStencil |
                TexFormat::Alpha |
                TexFormat::Red |
                TexFormat::RG |
                TexFormat::RGB |
                TexFormat::RGBA |
                TexFormat::Luminance |
                TexFormat::LuminanceAlpha
        )
    }

    pub fn to_unsized(self) -> TexFormat {
        match self {
            TexFormat::R8 => TexFormat::Red,
            TexFormat::R8SNorm => TexFormat::Red,
            TexFormat::R16f => TexFormat::Red,
            TexFormat::R32f => TexFormat::Red,
            TexFormat::R8ui => TexFormat::RedInteger,
            TexFormat::R8i => TexFormat::RedInteger,
            TexFormat::R16ui => TexFormat::RedInteger,
            TexFormat::R16i => TexFormat::RedInteger,
            TexFormat::R32ui => TexFormat::RedInteger,
            TexFormat::R32i => TexFormat::RedInteger,
            TexFormat::RG8 => TexFormat::RG,
            TexFormat::RG8SNorm => TexFormat::RG,
            TexFormat::RG16f => TexFormat::RG,
            TexFormat::RG32f => TexFormat::RG,
            TexFormat::RG8ui => TexFormat::RGInteger,
            TexFormat::RG8i => TexFormat::RGInteger,
            TexFormat::RG16ui => TexFormat::RGInteger,
            TexFormat::RG16i => TexFormat::RGInteger,
            TexFormat::RG32ui => TexFormat::RGInteger,
            TexFormat::RG32i => TexFormat::RGInteger,
            TexFormat::RGB8 => TexFormat::RGB,
            TexFormat::SRGB8 => TexFormat::RGB,
            TexFormat::RGB565 => TexFormat::RGB,
            TexFormat::RGB8SNorm => TexFormat::RGB,
            TexFormat::R11fG11fB10f => TexFormat::RGB,
            TexFormat::RGB9E5 => TexFormat::RGB,
            TexFormat::RGB16f => TexFormat::RGB,
            TexFormat::RGB32f => TexFormat::RGB,
            TexFormat::RGB8ui => TexFormat::RGBInteger,
            TexFormat::RGB8i => TexFormat::RGBInteger,
            TexFormat::RGB16ui => TexFormat::RGBInteger,
            TexFormat::RGB16i => TexFormat::RGBInteger,
            TexFormat::RGB32ui => TexFormat::RGBInteger,
            TexFormat::RGB32i => TexFormat::RGBInteger,
            TexFormat::RGBA8 => TexFormat::RGBA,
            TexFormat::SRGB8Alpha8 => TexFormat::RGBA,
            TexFormat::RGBA8SNorm => TexFormat::RGBA,
            TexFormat::RGB5A1 => TexFormat::RGBA,
            TexFormat::RGBA4 => TexFormat::RGBA,
            TexFormat::RGB10A2 => TexFormat::RGBA,
            TexFormat::RGBA16f => TexFormat::RGBA,
            TexFormat::RGBA32f => TexFormat::RGBA,
            TexFormat::RGBA8ui => TexFormat::RGBAInteger,
            TexFormat::RGBA8i => TexFormat::RGBAInteger,
            TexFormat::RGB10A2ui => TexFormat::RGBAInteger,
            TexFormat::RGBA16ui => TexFormat::RGBAInteger,
            TexFormat::RGBA16i => TexFormat::RGBAInteger,
            TexFormat::RGBA32i => TexFormat::RGBAInteger,
            TexFormat::RGBA32ui => TexFormat::RGBAInteger,
            TexFormat::DepthComponent16 => TexFormat::DepthComponent,
            TexFormat::DepthComponent24 => TexFormat::DepthComponent,
            TexFormat::DepthComponent32f => TexFormat::DepthComponent,
            TexFormat::Depth24Stencil8 => TexFormat::DepthStencil,
            TexFormat::Depth32fStencil8 => TexFormat::DepthStencil,
            TexFormat::Alpha32f => TexFormat::Alpha,
            TexFormat::Alpha16f => TexFormat::Alpha,
            TexFormat::Luminance32f => TexFormat::Luminance,
            TexFormat::Luminance16f => TexFormat::Luminance,
            TexFormat::LuminanceAlpha32f => TexFormat::LuminanceAlpha,
            TexFormat::LuminanceAlpha16f => TexFormat::LuminanceAlpha,
            _ => self,
        }
    }

    pub fn compatible_data_types(self) -> &'static [TexDataType] {
        match self {
            TexFormat::RGB => &[
                TexDataType::UnsignedByte,
                TexDataType::UnsignedShort565,
                TexDataType::Float,
                TexDataType::HalfFloat,
            ][..],
            TexFormat::RGBA => &[
                TexDataType::UnsignedByte,
                TexDataType::UnsignedShort4444,
                TexDataType::UnsignedShort5551,
                TexDataType::Float,
                TexDataType::HalfFloat,
            ][..],
            TexFormat::LuminanceAlpha => &[
                TexDataType::UnsignedByte,
                TexDataType::Float,
                TexDataType::HalfFloat,
            ][..],
            TexFormat::Luminance => &[
                TexDataType::UnsignedByte,
                TexDataType::Float,
                TexDataType::HalfFloat,
            ][..],
            TexFormat::Alpha => &[
                TexDataType::UnsignedByte,
                TexDataType::Float,
                TexDataType::HalfFloat,
            ][..],
            TexFormat::LuminanceAlpha32f => &[TexDataType::Float][..],
            TexFormat::LuminanceAlpha16f => &[TexDataType::HalfFloat][..],
            TexFormat::Luminance32f => &[TexDataType::Float][..],
            TexFormat::Luminance16f => &[TexDataType::HalfFloat][..],
            TexFormat::Alpha32f => &[TexDataType::Float][..],
            TexFormat::Alpha16f => &[TexDataType::HalfFloat][..],
            TexFormat::R8 => &[TexDataType::UnsignedByte][..],
            TexFormat::R8SNorm => &[TexDataType::Byte][..],
            TexFormat::R16f => &[TexDataType::HalfFloat, TexDataType::Float][..],
            TexFormat::R32f => &[TexDataType::Float][..],
            TexFormat::R8ui => &[TexDataType::UnsignedByte][..],
            TexFormat::R8i => &[TexDataType::Byte][..],
            TexFormat::R16ui => &[TexDataType::UnsignedShort][..],
            TexFormat::R16i => &[TexDataType::Short][..],
            TexFormat::R32ui => &[TexDataType::UnsignedInt][..],
            TexFormat::R32i => &[TexDataType::Int][..],
            TexFormat::RG8 => &[TexDataType::UnsignedByte][..],
            TexFormat::RG8SNorm => &[TexDataType::Byte][..],
            TexFormat::RG16f => &[TexDataType::HalfFloat, TexDataType::Float][..],
            TexFormat::RG32f => &[TexDataType::Float][..],
            TexFormat::RG8ui => &[TexDataType::UnsignedByte][..],
            TexFormat::RG8i => &[TexDataType::Byte][..],
            TexFormat::RG16ui => &[TexDataType::UnsignedShort][..],
            TexFormat::RG16i => &[TexDataType::Short][..],
            TexFormat::RG32ui => &[TexDataType::UnsignedInt][..],
            TexFormat::RG32i => &[TexDataType::Int][..],
            TexFormat::RGB8 => &[TexDataType::UnsignedByte][..],
            TexFormat::SRGB8 => &[TexDataType::UnsignedByte][..],
            TexFormat::RGB565 => &[TexDataType::UnsignedByte, TexDataType::UnsignedShort565][..],
            TexFormat::RGB8SNorm => &[TexDataType::Byte][..],
            TexFormat::R11fG11fB10f => &[
                TexDataType::UnsignedInt10f11f11fRev,
                TexDataType::HalfFloat,
                TexDataType::Float,
            ][..],
            TexFormat::RGB9E5 => &[
                TexDataType::UnsignedInt5999Rev,
                TexDataType::HalfFloat,
                TexDataType::Float,
            ][..],
            TexFormat::RGB16f => &[TexDataType::HalfFloat, TexDataType::Float][..],
            TexFormat::RGB32f => &[TexDataType::Float][..],
            TexFormat::RGB8ui => &[TexDataType::UnsignedByte][..],
            TexFormat::RGB8i => &[TexDataType::Byte][..],
            TexFormat::RGB16ui => &[TexDataType::UnsignedShort][..],
            TexFormat::RGB16i => &[TexDataType::Short][..],
            TexFormat::RGB32ui => &[TexDataType::UnsignedInt][..],
            TexFormat::RGB32i => &[TexDataType::Int][..],
            TexFormat::RGBA8 => &[TexDataType::UnsignedByte][..],
            TexFormat::SRGB8Alpha8 => &[TexDataType::UnsignedByte][..],
            TexFormat::RGBA8SNorm => &[TexDataType::Byte][..],
            TexFormat::RGB5A1 => &[
                TexDataType::UnsignedByte,
                TexDataType::UnsignedShort5551,
                TexDataType::UnsignedInt2101010Rev,
            ][..],
            TexFormat::RGBA4 => &[TexDataType::UnsignedByte, TexDataType::UnsignedShort4444][..],
            TexFormat::RGB10A2 => &[TexDataType::UnsignedInt2101010Rev][..],
            TexFormat::RGBA16f => &[TexDataType::HalfFloat, TexDataType::Float][..],
            TexFormat::RGBA32f => &[TexDataType::Float][..],
            TexFormat::RGBA8ui => &[TexDataType::UnsignedByte][..],
            TexFormat::RGBA8i => &[TexDataType::Byte][..],
            TexFormat::RGB10A2ui => &[TexDataType::UnsignedInt2101010Rev][..],
            TexFormat::RGBA16ui => &[TexDataType::UnsignedShort][..],
            TexFormat::RGBA16i => &[TexDataType::Short][..],
            TexFormat::RGBA32i => &[TexDataType::Int][..],
            TexFormat::RGBA32ui => &[TexDataType::UnsignedInt][..],
            TexFormat::DepthComponent16 => {
                &[TexDataType::UnsignedShort, TexDataType::UnsignedInt][..]
            },
            TexFormat::DepthComponent24 => &[TexDataType::UnsignedInt][..],
            TexFormat::DepthComponent32f => &[TexDataType::Float][..],
            TexFormat::Depth24Stencil8 => &[TexDataType::UnsignedInt248][..],
            TexFormat::Depth32fStencil8 => &[TexDataType::Float32UnsignedInt248Rev][..],
            TexFormat::CompressedRgbS3tcDxt1 |
            TexFormat::CompressedRgbaS3tcDxt1 |
            TexFormat::CompressedRgbaS3tcDxt3 |
            TexFormat::CompressedRgbaS3tcDxt5 => &[TexDataType::UnsignedByte][..],
            _ => &[][..],
        }
    }

    pub fn required_webgl_version(self) -> WebGLVersion {
        match self {
            TexFormat::DepthComponent |
            TexFormat::Alpha |
            TexFormat::RGB |
            TexFormat::RGBA |
            TexFormat::Luminance |
            TexFormat::LuminanceAlpha |
            TexFormat::CompressedRgbS3tcDxt1 |
            TexFormat::CompressedRgbaS3tcDxt1 |
            TexFormat::CompressedRgbaS3tcDxt3 |
            TexFormat::CompressedRgbaS3tcDxt5 => WebGLVersion::WebGL1,
            _ => WebGLVersion::WebGL2,
        }
    }

    pub fn usable_as_internal(self) -> bool {
        !self.compatible_data_types().is_empty()
    }
}

#[derive(PartialEq)]
pub enum SizedDataType {
    Int8,
    Int16,
    Int32,
    Uint8,
    Uint16,
    Uint32,
    Float32,
}

impl TexDataType {
    /// Returns the compatible sized data type for this texture data type.
    pub fn sized_data_type(&self) -> SizedDataType {
        match self {
            TexDataType::Byte => SizedDataType::Int8,
            TexDataType::UnsignedByte => SizedDataType::Uint8,
            TexDataType::Short => SizedDataType::Int16,
            TexDataType::UnsignedShort |
            TexDataType::UnsignedShort4444 |
            TexDataType::UnsignedShort5551 |
            TexDataType::UnsignedShort565 => SizedDataType::Uint16,
            TexDataType::Int => SizedDataType::Int32,
            TexDataType::UnsignedInt |
            TexDataType::UnsignedInt10f11f11fRev |
            TexDataType::UnsignedInt2101010Rev |
            TexDataType::UnsignedInt5999Rev |
            TexDataType::UnsignedInt248 => SizedDataType::Uint32,
            TexDataType::HalfFloat => SizedDataType::Uint16,
            TexDataType::Float | TexDataType::Float32UnsignedInt248Rev => SizedDataType::Float32,
        }
    }

    /// Returns the size in bytes of each element of data.
    pub fn element_size(&self) -> u32 {
        use self::*;
        match *self {
            TexDataType::Byte | TexDataType::UnsignedByte => 1,
            TexDataType::Short |
            TexDataType::UnsignedShort |
            TexDataType::UnsignedShort4444 |
            TexDataType::UnsignedShort5551 |
            TexDataType::UnsignedShort565 => 2,
            TexDataType::Int |
            TexDataType::UnsignedInt |
            TexDataType::UnsignedInt10f11f11fRev |
            TexDataType::UnsignedInt2101010Rev |
            TexDataType::UnsignedInt5999Rev => 4,
            TexDataType::UnsignedInt248 => 4,
            TexDataType::Float => 4,
            TexDataType::HalfFloat => 2,
            TexDataType::Float32UnsignedInt248Rev => 4,
        }
    }

    /// Returns how many components a single element may hold. For example, a
    /// UnsignedShort4444 holds four components, each with 4 bits of data.
    pub fn components_per_element(&self) -> u32 {
        match *self {
            TexDataType::Byte => 1,
            TexDataType::UnsignedByte => 1,
            TexDataType::Short => 1,
            TexDataType::UnsignedShort => 1,
            TexDataType::UnsignedShort565 => 3,
            TexDataType::UnsignedShort5551 => 4,
            TexDataType::UnsignedShort4444 => 4,
            TexDataType::Int => 1,
            TexDataType::UnsignedInt => 1,
            TexDataType::UnsignedInt10f11f11fRev => 3,
            TexDataType::UnsignedInt2101010Rev => 4,
            TexDataType::UnsignedInt5999Rev => 4,
            TexDataType::UnsignedInt248 => 2,
            TexDataType::Float => 1,
            TexDataType::HalfFloat => 1,
            TexDataType::Float32UnsignedInt248Rev => 2,
        }
    }

    pub fn required_webgl_version(self) -> WebGLVersion {
        match self {
            TexDataType::UnsignedByte |
            TexDataType::UnsignedShort4444 |
            TexDataType::UnsignedShort5551 |
            TexDataType::UnsignedShort565 |
            TexDataType::Float |
            TexDataType::HalfFloat => WebGLVersion::WebGL1,
            _ => WebGLVersion::WebGL2,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum AlphaTreatment {
    Premultiply,
    Unmultiply,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum YAxisTreatment {
    AsIs,
    Flipped,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GLContextAttributes {
    pub alpha: bool,
    pub depth: bool,
    pub stencil: bool,
    pub antialias: bool,
    pub premultiplied_alpha: bool,
    pub preserve_drawing_buffer: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GLLimits {
    pub max_vertex_attribs: u32,
    pub max_tex_size: u32,
    pub max_cube_map_tex_size: u32,
    pub max_combined_texture_image_units: u32,
    pub max_fragment_uniform_vectors: u32,
    pub max_renderbuffer_size: u32,
    pub max_texture_image_units: u32,
    pub max_varying_vectors: u32,
    pub max_vertex_texture_image_units: u32,
    pub max_vertex_uniform_vectors: u32,
    pub max_client_wait_timeout_webgl: std::time::Duration,
    pub max_transform_feedback_separate_attribs: u32,
    pub max_vertex_output_vectors: u32,
    pub max_fragment_input_vectors: u32,
    pub max_draw_buffers: u32,
    pub max_color_attachments: u32,
    pub max_uniform_buffer_bindings: u32,
    pub min_program_texel_offset: i32,
    pub max_program_texel_offset: u32,
    pub max_uniform_block_size: u64,
    pub max_combined_uniform_blocks: u32,
    pub max_combined_vertex_uniform_components: u64,
    pub max_combined_fragment_uniform_components: u64,
    pub max_vertex_uniform_blocks: u32,
    pub max_vertex_uniform_components: u32,
    pub max_fragment_uniform_blocks: u32,
    pub max_fragment_uniform_components: u32,
    pub max_3d_texture_size: u32,
    pub max_array_texture_layers: u32,
    pub uniform_buffer_offset_alignment: u32,
    pub max_element_index: u64,
    pub max_elements_indices: u32,
    pub max_elements_vertices: u32,
    pub max_fragment_input_components: u32,
    pub max_samples: u32,
    pub max_server_wait_timeout: std::time::Duration,
    pub max_texture_lod_bias: f32,
    pub max_varying_components: u32,
    pub max_vertex_output_components: u32,
}
