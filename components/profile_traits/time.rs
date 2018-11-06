/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::energy::read_energy_uj;
use ipc_channel::ipc::IpcSender;
use servo_config::opts;
use time::precise_time_ns;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TimerMetadata {
    pub url: String,
    pub iframe: TimerMetadataFrameType,
    pub incremental: TimerMetadataReflowType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfilerChan(pub IpcSender<ProfilerMsg>);

impl ProfilerChan {
    pub fn send(&self, msg: ProfilerMsg) {
        if let Err(e) = self.0.send(msg) {
            warn!("Error communicating with the time profiler thread: {}", e);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProfilerData {
    NoRecords,
    Record(Vec<f64>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProfilerMsg {
    /// Normal message used for reporting time
    Time(
        (ProfilerCategory, Option<TimerMetadata>),
        (u64, u64),
        (u64, u64),
    ),
    /// Message used to get time spend entries for a particular ProfilerBuckets (in nanoseconds)
    Get(
        (ProfilerCategory, Option<TimerMetadata>),
        IpcSender<ProfilerData>,
    ),
    /// Message used to force print the profiling metrics
    Print,
    /// Tells the profiler to shut down.
    Exit(IpcSender<()>),
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ProfilerCategory {
    Compositing = 0x000,
    LayoutPerform = 0x100,
    LayoutStyleRecalc = 0x101,
    LayoutTextShaping = 0x102,
    LayoutRestyleDamagePropagation = 0x103,
    LayoutNonIncrementalReset = 0x104,
    LayoutSelectorMatch = 0x105,
    LayoutTreeBuilder = 0x106,
    LayoutDamagePropagate = 0x107,
    LayoutGeneratedContent = 0x108,
    LayoutDisplayListSorting = 0x109,
    LayoutFloatPlacementSpeculation = 0x10a,
    LayoutMain = 0x10b,
    LayoutStoreOverflow = 0x10c,
    LayoutParallelWarmup = 0x10d,
    LayoutDispListBuild = 0x10e,
    NetHTTPRequestResponse = 0x300,
    PaintingPerTile = 0x401,
    PaintingPrepBuff = 0x402,
    Painting = 0x403,
    ImageDecoding = 0x500,
    ImageSaving = 0x501,
    ScriptAttachLayout = 0x600,
    ScriptConstellationMsg = 0x601,
    ScriptDevtoolsMsg = 0x602,
    ScriptDocumentEvent = 0x603,
    ScriptDomEvent = 0x604,
    ScriptEvaluate = 0x605,
    ScriptEvent = 0x606,
    ScriptFileRead = 0x607,
    ScriptImageCacheMsg = 0x608,
    ScriptInputEvent = 0x609,
    ScriptNetworkEvent = 0x60a,
    ScriptParseHTML = 0x60b,
    ScriptPlannedNavigation = 0x60c,
    ScriptResize = 0x60d,
    ScriptSetScrollState = 0x60e,
    ScriptSetViewport = 0x60f,
    ScriptTimerEvent = 0x610,
    ScriptStylesheetLoad = 0x611,
    ScriptUpdateReplacedElement = 0x612,
    ScriptWebSocketEvent = 0x613,
    ScriptWorkerEvent = 0x614,
    ScriptServiceWorkerEvent = 0x615,
    ScriptParseXML = 0x616,
    ScriptEnterFullscreen = 0x617,
    ScriptExitFullscreen = 0x618,
    ScriptWebVREvent = 0x619,
    ScriptWorkletEvent = 0x61a,
    ScriptPerformanceEvent = 0x61b,
    ScriptHistoryEvent = 0x61c,
    TimeToFirstPaint = 0x800,
    TimeToFirstContentfulPaint = 0x801,
    TimeToInteractive = 0x802,
    IpcReceiver = 0x803,
    IpcBytesReceiver = 0x804,
    ApplicationHeartbeat = 0x901,
    WebGlGetContextAttributes = 0xa01,
    WebGlActiveTexture = 0xa02,
    WebGlBlendColor = 0xa03,
    WebGlBlendEquation = 0xa04,
    WebGlBlendEquationSeparate = 0xa05,
    WebGlBlendFunc = 0xa06,
    WebGlBlendFuncSeparate = 0xa07,
    WebGlAttachShader = 0xa08,
    WebGlDetachShader = 0xa09,
    WebGlBindAttribLocation = 0xa0a,
    WebGlBufferData = 0xa0b,
    WebGlBufferSubData = 0xa0c,
    WebGlClear = 0xa0d,
    WebGlClearColor = 0xa0e,
    WebGlClearDepth = 0xa0f,
    WebGlClearStencil = 0xa10,
    WebGlColorMask = 0xa11,
    WebGlCullFace = 0xa12,
    WebGlFrontFace = 0xa13,
    WebGlDepthFunc = 0xa14,
    WebGlDepthMask = 0xa15,
    WebGlDepthRange = 0xa16,
    WebGlEnable = 0xa17,
    WebGlDisable = 0xa18,
    WebGlCompileShader = 0xa19,
    WebGlCopyTexImage2D = 0xa1a,
    WebGlCopyTexSubImage2D = 0xa1b,
    WebGlCreateBuffer = 0xa1c,
    WebGlCreateFramebuffer = 0xa1d,
    WebGlCreateRenderbuffer = 0xa1e,
    WebGlCreateTexture = 0xa1f,
    WebGlCreateProgram = 0xa20,
    WebGlCreateShader = 0xa21,
    WebGlDeleteBuffer = 0xa22,
    WebGlDeleteFramebuffer = 0xa23,
    WebGlDeleteRenderbuffer = 0xa24,
    WebGlDeleteTexture = 0xa25,
    WebGlDeleteProgram = 0xa26,
    WebGlDeleteShader = 0xa27,
    WebGlBindBuffer = 0xa28,
    WebGlBindFramebuffer = 0xa29,
    WebGlBindRenderbuffer = 0xa2a,
    WebGlBindTexture = 0xa2b,
    WebGlDisableVertexAttribArray = 0xa2c,
    WebGlEnableVertexAttribArray = 0xa2d,
    WebGlFramebufferRenderbuffer = 0xa2e,
    WebGlFramebufferTexture2D = 0xa2f,
    WebGlGetExtensions = 0xa30,
    WebGlGetShaderPrecisionFormat = 0xa31,
    WebGlGetUniformLocation = 0xa32,
    WebGlGetShaderInfoLog = 0xa33,
    WebGlGetProgramInfoLog = 0xa34,
    WebGlGetFramebufferAttachmentParameter = 0xa35,
    WebGlGetRenderbufferParameter = 0xa36,
    WebGlPolygonOffset = 0xa37,
    WebGlRenderbufferStorage = 0xa38,
    WebGlReadPixels = 0xa39,
    WebGlSampleCoverage = 0xa3a,
    WebGlScissor = 0xa3b,
    WebGlStencilFunc = 0xa3c,
    WebGlStencilFuncSeparate = 0xa3d,
    WebGlStencilMask = 0xa3e,
    WebGlStencilMaskSeparate = 0xa3f,
    WebGlStencilOp = 0xa40,
    WebGlStencilOpSeparate = 0xa41,
    WebGlHint = 0xa42,
    WebGlLineWidth = 0xa43,
    WebGlPixelStorei = 0xa44,
    WebGlLinkProgram = 0xa45,
    WebGlUniform1f = 0xa46,
    WebGlUniform1fv = 0xa47,
    WebGlUniform1i = 0xa48,
    WebGlUniform1iv = 0xa49,
    WebGlUniform2f = 0xa4a,
    WebGlUniform2fv = 0xa4b,
    WebGlUniform2i = 0xa4c,
    WebGlUniform2iv = 0xa4d,
    WebGlUniform3f = 0xa4e,
    WebGlUniform3fv = 0xa4f,
    WebGlUniform3i = 0xa50,
    WebGlUniform3iv = 0xa51,
    WebGlUniform4f = 0xa52,
    WebGlUniform4fv = 0xa53,
    WebGlUniform4i = 0xa54,
    WebGlUniform4iv = 0xa55,
    WebGlUniformMatrix2fv = 0xa56,
    WebGlUniformMatrix3fv = 0xa57,
    WebGlUniformMatrix4fv = 0xa58,
    WebGlUseProgram = 0xa59,
    WebGlValidateProgram = 0xa5a,
    WebGlVertexAttrib = 0xa5b,
    WebGlVertexAttribPointer = 0xa5c,
    WebGlVertexAttribPointer2f = 0xa5d,
    WebGlSetViewport = 0xa5e,
    WebGlTexImage2D = 0xa5f,
    WebGlTexSubImage2D = 0xa60,
    WebGlDrawingBufferWidth = 0xa61,
    WebGlDrawingBufferHeight = 0xa62,
    WebGlFinish = 0xa63,
    WebGlFlush = 0xa64,
    WebGlGenerateMipmap = 0xa65,
    WebGlCreateVertexArray = 0xa66,
    WebGlDeleteVertexArray = 0xa67,
    WebGlBindVertexArray = 0xa68,
    WebGlGetParameterBool = 0xa69,
    WebGlGetParameterBool4 = 0xa6a,
    WebGlGetParameterInt = 0xa6b,
    WebGlGetParameterInt2 = 0xa6c,
    WebGlGetParameterInt4 = 0xa6d,
    WebGlGetParameterFloat = 0xa6e,
    WebGlGetParameterFloat2 = 0xa6f,
    WebGlGetParameterFloat4 = 0xa70,
    WebGlGetProgramValidateStatus = 0xa71,
    WebGlGetProgramActiveUniforms = 0xa72,
    WebGlGetCurrentVertexAttrib = 0xa73,
    WebGlGetTexParameterFloat = 0xa74,
    WebGlGetTexParameterInt = 0xa75,
    WebGlTexParameteri = 0xa76,
    WebGlTexParameterf = 0xa77,
    WebGlDrawArrays = 0xa78,
    WebGlDrawArraysInstanced = 0xa79,
    WebGlDrawElements = 0xa7a,
    WebGlDrawElementsInstanced = 0xa7b,
    WebGlVertexAttribDivisor = 0xa7c,
    WebGlGetUniformBool = 0xa7d,
    WebGlGetUniformBool2 = 0xa7e,
    WebGlGetUniformBool3 = 0xa7f,
    WebGlGetUniformBool4 = 0xa80,
    WebGlGetUniformInt = 0xa81,
    WebGlGetUniformInt2 = 0xa82,
    WebGlGetUniformInt3 = 0xa83,
    WebGlGetUniformInt4 = 0xa84,
    WebGlGetUniformFloat = 0xa85,
    WebGlGetUniformFloat2 = 0xa86,
    WebGlGetUniformFloat3 = 0xa87,
    WebGlGetUniformFloat4 = 0xa88,
    WebGlGetUniformFloat9 = 0xa89,
    WebGlGetUniformFloat16 = 0xa8a,
    WebGlInitializeFramebuffer = 0xa8b,
    WebGlDOMFinish = 0xb01,
    WebGlDOMDrawingBufferWidth = 0xb02,
    WebGlDOMDrawingBufferHeight = 0xb03,
    WebGlDOMGetParameter = 0xb04,
    WebGlDOMGetTexParameter = 0xb05,
    WebGlDOMGetContextAttributes = 0xb06,
    WebGlDOMGetFramebufferAttachmentParameter = 0xb07,
    WebGlDOMGetRenderbufferParameter = 0xb08,
    WebGlDOMGetProgramParameter = 0xb09,
    WebGlDOMGetShaderPrecisionFormat = 0xb0a,
    WebGlDOMGetVertexAttrib = 0xb0b,
    WebGlDOMReadPixels = 0xb0c,
    WebGlDOMGetUniform = 0xb0d,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TimerMetadataFrameType {
    RootWindow,
    IFrame,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TimerMetadataReflowType {
    Incremental,
    FirstReflow,
}

pub fn profile<T, F>(
    category: ProfilerCategory,
    meta: Option<TimerMetadata>,
    profiler_chan: ProfilerChan,
    callback: F,
) -> T
where
    F: FnOnce() -> T,
{
    if opts::get().signpost {
        signpost::start(category as u32, &[0, 0, 0, (category as usize) >> 8]);
    }
    let start_energy = read_energy_uj();
    let start_time = precise_time_ns();

    let val = callback();

    let end_time = precise_time_ns();
    let end_energy = read_energy_uj();
    if opts::get().signpost {
        signpost::end(category as u32, &[0, 0, 0, (category as usize) >> 8]);
    }

    send_profile_data(
        category,
        meta,
        &profiler_chan,
        start_time,
        end_time,
        start_energy,
        end_energy,
    );
    val
}

pub fn send_profile_data(
    category: ProfilerCategory,
    meta: Option<TimerMetadata>,
    profiler_chan: &ProfilerChan,
    start_time: u64,
    end_time: u64,
    start_energy: u64,
    end_energy: u64,
) {
    profiler_chan.send(ProfilerMsg::Time(
        (category, meta),
        (start_time, end_time),
        (start_energy, end_energy),
    ));
}
