/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpurenderpipeline
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPURenderPipeline {
};
GPURenderPipeline includes GPUObjectBase;
GPURenderPipeline includes GPUPipelineBase;

dictionary GPURenderPipelineDescriptor : GPUPipelineDescriptorBase {
    required GPUVertexState vertex;
    GPUPrimitiveState primitive = {};
    GPUDepthStencilState depthStencil;
    GPUMultisampleState multisample = {};
    GPUFragmentState fragment;
};

dictionary GPUPrimitiveState {
    GPUPrimitiveTopology topology = "triangle-list";
    GPUIndexFormat stripIndexFormat;
    GPUFrontFace frontFace = "ccw";
    GPUCullMode cullMode = "none";
    // Enable depth clamping (requires "depth-clamping" feature)
    boolean clampDepth = false;
};

enum GPUPrimitiveTopology {
    "point-list",
    "line-list",
    "line-strip",
    "triangle-list",
    "triangle-strip"
};

enum GPUFrontFace {
    "ccw",
    "cw"
};

enum GPUCullMode {
    "none",
    "front",
    "back"
};

dictionary GPUMultisampleState {
    GPUSize32 count = 1;
    GPUSampleMask mask = 0xFFFFFFFF;
    boolean alphaToCoverageEnabled = false;
};

dictionary GPUFragmentState: GPUProgrammableStage {
    required sequence<GPUColorTargetState> targets;
};

dictionary GPUColorTargetState {
    required GPUTextureFormat format;
    GPUBlendState blend;
    GPUColorWriteFlags writeMask = 0xF;  // GPUColorWrite.ALL
};

dictionary GPUBlendState {
    required GPUBlendComponent color;
    required GPUBlendComponent alpha;
};

typedef [EnforceRange] unsigned long GPUSampleMask;
typedef [EnforceRange] long GPUDepthBias;

dictionary GPUBlendComponent {
    GPUBlendFactor srcFactor = "one";
    GPUBlendFactor dstFactor = "zero";
    GPUBlendOperation operation = "add";
};

enum GPUBlendFactor {
    "zero",
    "one",
    "src",
    "one-minus-src",
    "src-alpha",
    "one-minus-src-alpha",
    "dst",
    "one-minus-dst",
    "dst-alpha",
    "one-minus-dst-alpha",
    "src-alpha-saturated",
    "constant",
    "one-minus-constant",
};

enum GPUBlendOperation {
    "add",
    "subtract",
    "reverse-subtract",
    "min",
    "max"
};

dictionary GPUDepthStencilState {
    required GPUTextureFormat format;

    boolean depthWriteEnabled = false;
    GPUCompareFunction depthCompare = "always";

    GPUStencilFaceState stencilFront = {};
    GPUStencilFaceState stencilBack = {};

    GPUStencilValue stencilReadMask = 0xFFFFFFFF;
    GPUStencilValue stencilWriteMask = 0xFFFFFFFF;

    GPUDepthBias depthBias = 0;
    float depthBiasSlopeScale = 0;
    float depthBiasClamp = 0;
};

dictionary GPUStencilFaceState {
    GPUCompareFunction compare = "always";
    GPUStencilOperation failOp = "keep";
    GPUStencilOperation depthFailOp = "keep";
    GPUStencilOperation passOp = "keep";
};

enum GPUStencilOperation {
    "keep",
    "zero",
    "replace",
    "invert",
    "increment-clamp",
    "decrement-clamp",
    "increment-wrap",
    "decrement-wrap"
};

enum GPUIndexFormat {
    "uint16",
    "uint32",
};

typedef [EnforceRange] unsigned long GPUStencilValue;

enum GPUVertexFormat {
    "uint8x2",
    "uint8x4",
    "sint8x2",
    "sint8x4",
    "unorm8x2",
    "unorm8x4",
    "snorm8x2",
    "snorm8x4",
    "uint16x2",
    "uint16x4",
    "sint16x2",
    "sint16x4",
    "unorm16x2",
    "unorm16x4",
    "snorm16x2",
    "snorm16x4",
    "float16x2",
    "float16x4",
    "float32",
    "float32x2",
    "float32x3",
    "float32x4",
    "uint32",
    "uint32x2",
    "uint32x3",
    "uint32x4",
    "sint32",
    "sint32x2",
    "sint32x3",
    "sint32x4",
};

enum GPUVertexStepMode {
    "vertex",
    "instance",
};

dictionary GPUVertexState: GPUProgrammableStage {
    sequence<GPUVertexBufferLayout?> buffers = [];
};

dictionary GPUVertexBufferLayout {
    required GPUSize64 arrayStride;
    GPUVertexStepMode stepMode = "vertex";
    required sequence<GPUVertexAttribute> attributes;
};

dictionary GPUVertexAttribute {
    required GPUVertexFormat format;
    required GPUSize64 offset;
    required GPUIndex32 shaderLocation;
};
