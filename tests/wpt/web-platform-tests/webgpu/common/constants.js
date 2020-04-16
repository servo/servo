/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

// https://github.com/gpuweb/gpuweb/blob/0a48816412b5d08a5fb8b89005e019165a1a2c63/spec/index.bs
// String enums
export let ExtensionName;

(function (ExtensionName) {
  ExtensionName["TextureCompressionBC"] = "texture-compression-bc";
})(ExtensionName || (ExtensionName = {}));

export let AddressMode;

(function (AddressMode) {
  AddressMode["ClampToEdge"] = "clamp-to-edge";
  AddressMode["Repeat"] = "repeat";
  AddressMode["MirrorRepeat"] = "mirror-repeat";
})(AddressMode || (AddressMode = {}));

export let BindingType;

(function (BindingType) {
  BindingType["UniformBuffer"] = "uniform-buffer";
  BindingType["StorageBuffer"] = "storage-buffer";
  BindingType["ReadonlyStorageBuffer"] = "readonly-storage-buffer";
  BindingType["Sampler"] = "sampler";
  BindingType["ComparisonSampler"] = "comparison-sampler";
  BindingType["SampledTexture"] = "sampled-texture";
  BindingType["ReadonlyStorageTexture"] = "readonly-storage-texture";
  BindingType["WriteonlyStorageTexture"] = "writeonly-storage-texture";
})(BindingType || (BindingType = {}));

export let BlendFactor;

(function (BlendFactor) {
  BlendFactor["Zero"] = "zero";
  BlendFactor["One"] = "one";
  BlendFactor["SrcColor"] = "src-color";
  BlendFactor["OneMinusSrcColor"] = "one-minus-src-color";
  BlendFactor["SrcAlpha"] = "src-alpha";
  BlendFactor["OneMinusSrcAlpha"] = "one-minus-src-alpha";
  BlendFactor["DstColor"] = "dst-color";
  BlendFactor["OneMinusDstColor"] = "one-minus-dst-color";
  BlendFactor["DstAlpha"] = "dst-alpha";
  BlendFactor["OneMinusDstAlpha"] = "one-minus-dst-alpha";
  BlendFactor["SrcAlphaSaturated"] = "src-alpha-saturated";
  BlendFactor["BlendColor"] = "blend-color";
  BlendFactor["OneMinusBlendColor"] = "one-minus-blend-color";
})(BlendFactor || (BlendFactor = {}));

export let BlendOperation;

(function (BlendOperation) {
  BlendOperation["Add"] = "add";
  BlendOperation["Subtract"] = "subtract";
  BlendOperation["ReverseSubtract"] = "reverse-subtract";
  BlendOperation["Min"] = "min";
  BlendOperation["Max"] = "max";
})(BlendOperation || (BlendOperation = {}));

export let CompareFunction;

(function (CompareFunction) {
  CompareFunction["Never"] = "never";
  CompareFunction["Less"] = "less";
  CompareFunction["Equal"] = "equal";
  CompareFunction["LessEqual"] = "less-equal";
  CompareFunction["Greater"] = "greater";
  CompareFunction["NotEqual"] = "not-equal";
  CompareFunction["GreaterEqual"] = "greater-equal";
  CompareFunction["Always"] = "always";
})(CompareFunction || (CompareFunction = {}));

export let CullMode;

(function (CullMode) {
  CullMode["None"] = "none";
  CullMode["Front"] = "front";
  CullMode["Back"] = "back";
})(CullMode || (CullMode = {}));

export let FilterMode;

(function (FilterMode) {
  FilterMode["Nearest"] = "nearest";
  FilterMode["Linear"] = "linear";
})(FilterMode || (FilterMode = {}));

export let FrontFace;

(function (FrontFace) {
  FrontFace["CCW"] = "ccw";
  FrontFace["CW"] = "cw";
})(FrontFace || (FrontFace = {}));

export let IndexFormat;

(function (IndexFormat) {
  IndexFormat["Uint16"] = "uint16";
  IndexFormat["Uint32"] = "uint32";
})(IndexFormat || (IndexFormat = {}));

export let InputStepMode;

(function (InputStepMode) {
  InputStepMode["Vertex"] = "vertex";
  InputStepMode["Instance"] = "instance";
})(InputStepMode || (InputStepMode = {}));

export let LoadOp;

(function (LoadOp) {
  LoadOp["Load"] = "load";
})(LoadOp || (LoadOp = {}));

export let PrimitiveTopology;

(function (PrimitiveTopology) {
  PrimitiveTopology["PointList"] = "point-list";
  PrimitiveTopology["LineList"] = "line-list";
  PrimitiveTopology["LineStrip"] = "line-strip";
  PrimitiveTopology["TriangleList"] = "triangle-list";
  PrimitiveTopology["TriangleStrip"] = "triangle-strip";
})(PrimitiveTopology || (PrimitiveTopology = {}));

export let StencilOperation;

(function (StencilOperation) {
  StencilOperation["Keep"] = "keep";
  StencilOperation["Zero"] = "zero";
  StencilOperation["Replace"] = "replace";
  StencilOperation["Invert"] = "invert";
  StencilOperation["IncrementClamp"] = "increment-clamp";
  StencilOperation["DecrementClamp"] = "decrement-clamp";
  StencilOperation["IncrementWrap"] = "increment-wrap";
  StencilOperation["DecrementWrap"] = "decrement-wrap";
})(StencilOperation || (StencilOperation = {}));

export let StoreOp;

(function (StoreOp) {
  StoreOp["Store"] = "store";
  StoreOp["Clear"] = "clear";
})(StoreOp || (StoreOp = {}));

export let TextureDimension;

(function (TextureDimension) {
  TextureDimension["E1d"] = "1d";
  TextureDimension["E2d"] = "2d";
  TextureDimension["E3d"] = "3d";
})(TextureDimension || (TextureDimension = {}));

export let TextureFormat;

(function (TextureFormat) {
  TextureFormat["R8Unorm"] = "r8unorm";
  TextureFormat["R8Snorm"] = "r8snorm";
  TextureFormat["R8Uint"] = "r8uint";
  TextureFormat["R8Sint"] = "r8sint";
  TextureFormat["R16Uint"] = "r16uint";
  TextureFormat["R16Sint"] = "r16sint";
  TextureFormat["R16Float"] = "r16float";
  TextureFormat["RG8Unorm"] = "rg8unorm";
  TextureFormat["RG8Snorm"] = "rg8snorm";
  TextureFormat["RG8Uint"] = "rg8uint";
  TextureFormat["RG8Sint"] = "rg8sint";
  TextureFormat["R32Uint"] = "r32uint";
  TextureFormat["R32Sint"] = "r32sint";
  TextureFormat["R32Float"] = "r32float";
  TextureFormat["RG16Uint"] = "rg16uint";
  TextureFormat["RG16Sint"] = "rg16sint";
  TextureFormat["RG16Float"] = "rg16float";
  TextureFormat["RGBA8Unorm"] = "rgba8unorm";
  TextureFormat["RGBA8UnormSRGB"] = "rgba8unorm-srgb";
  TextureFormat["RGBA8Snorm"] = "rgba8snorm";
  TextureFormat["RGBA8Uint"] = "rgba8uint";
  TextureFormat["RGBA8Sint"] = "rgba8sint";
  TextureFormat["BGRA8Unorm"] = "bgra8unorm";
  TextureFormat["BGRA8UnormSRGB"] = "bgra8unorm-srgb";
  TextureFormat["RGB10A2Unorm"] = "rgb10a2unorm";
  TextureFormat["RGB11B10Float"] = "rg11b10float";
  TextureFormat["RG32Uint"] = "rg32uint";
  TextureFormat["RG32Sint"] = "rg32sint";
  TextureFormat["RG32Float"] = "rg32float";
  TextureFormat["RGBA16Uint"] = "rgba16uint";
  TextureFormat["RGBA16Sint"] = "rgba16sint";
  TextureFormat["RGBA16Float"] = "rgba16float";
  TextureFormat["RGBA32Uint"] = "rgba32uint";
  TextureFormat["RGBA32Sint"] = "rgba32sint";
  TextureFormat["RGBA32Float"] = "rgba32float";
  TextureFormat["Depth32Float"] = "depth32float";
  TextureFormat["Depth24Plus"] = "depth24plus";
  TextureFormat["Depth24PlusStencil8"] = "depth24plus-stencil8";
})(TextureFormat || (TextureFormat = {}));

export let TextureComponentType;

(function (TextureComponentType) {
  TextureComponentType["Float"] = "float";
  TextureComponentType["Sint"] = "sint";
  TextureComponentType["Uint"] = "uint";
})(TextureComponentType || (TextureComponentType = {}));

export let TextureViewDimension;

(function (TextureViewDimension) {
  TextureViewDimension["E1d"] = "1d";
  TextureViewDimension["E2d"] = "2d";
  TextureViewDimension["E2dArray"] = "2d-array";
  TextureViewDimension["Cube"] = "cube";
  TextureViewDimension["CubeArray"] = "cube-array";
  TextureViewDimension["E3d"] = "3d";
})(TextureViewDimension || (TextureViewDimension = {}));

export let VertexFormat;

(function (VertexFormat) {
  VertexFormat["Uchar2"] = "uchar2";
  VertexFormat["Uchar4"] = "uchar4";
  VertexFormat["Char2"] = "char2";
  VertexFormat["Char4"] = "char4";
  VertexFormat["Uchar2Norm"] = "uchar2norm";
  VertexFormat["Uchar4Norm"] = "uchar4norm";
  VertexFormat["Char2Norm"] = "char2norm";
  VertexFormat["Char4Norm"] = "char4norm";
  VertexFormat["Ushort2"] = "ushort2";
  VertexFormat["Ushort4"] = "ushort4";
  VertexFormat["Short2"] = "short2";
  VertexFormat["Short4"] = "short4";
  VertexFormat["Ushort2Norm"] = "ushort2norm";
  VertexFormat["Ushort4Norm"] = "ushort4norm";
  VertexFormat["Short2Norm"] = "short2norm";
  VertexFormat["Short4Norm"] = "short4norm";
  VertexFormat["Half2"] = "half2";
  VertexFormat["Half4"] = "half4";
  VertexFormat["Float"] = "float";
  VertexFormat["Float2"] = "float2";
  VertexFormat["Float3"] = "float3";
  VertexFormat["Float4"] = "float4";
  VertexFormat["Uint"] = "uint";
  VertexFormat["Uint2"] = "uint2";
  VertexFormat["Uint3"] = "uint3";
  VertexFormat["Uint4"] = "uint4";
  VertexFormat["Int"] = "int";
  VertexFormat["Int2"] = "int2";
  VertexFormat["Int3"] = "int3";
  VertexFormat["Int4"] = "int4";
})(VertexFormat || (VertexFormat = {}));

export let TextureAspect; // Bit fields

(function (TextureAspect) {
  TextureAspect["All"] = "all";
  TextureAspect["StencilOnly"] = "stencil-only";
  TextureAspect["DepthOnly"] = "depth-only";
})(TextureAspect || (TextureAspect = {}));

export let BufferUsage;

(function (BufferUsage) {
  BufferUsage[BufferUsage["MapRead"] = 1] = "MapRead";
  BufferUsage[BufferUsage["MapWrite"] = 2] = "MapWrite";
  BufferUsage[BufferUsage["CopySrc"] = 4] = "CopySrc";
  BufferUsage[BufferUsage["CopyDst"] = 8] = "CopyDst";
  BufferUsage[BufferUsage["Index"] = 16] = "Index";
  BufferUsage[BufferUsage["Vertex"] = 32] = "Vertex";
  BufferUsage[BufferUsage["Uniform"] = 64] = "Uniform";
  BufferUsage[BufferUsage["Storage"] = 128] = "Storage";
  BufferUsage[BufferUsage["Indirect"] = 256] = "Indirect";
})(BufferUsage || (BufferUsage = {}));

export let ColorWrite;

(function (ColorWrite) {
  ColorWrite[ColorWrite["Red"] = 1] = "Red";
  ColorWrite[ColorWrite["Green"] = 2] = "Green";
  ColorWrite[ColorWrite["Blue"] = 4] = "Blue";
  ColorWrite[ColorWrite["Alpha"] = 8] = "Alpha";
  ColorWrite[ColorWrite["All"] = 15] = "All";
})(ColorWrite || (ColorWrite = {}));

export let ShaderStage;

(function (ShaderStage) {
  ShaderStage[ShaderStage["Vertex"] = 1] = "Vertex";
  ShaderStage[ShaderStage["Fragment"] = 2] = "Fragment";
  ShaderStage[ShaderStage["Compute"] = 4] = "Compute";
})(ShaderStage || (ShaderStage = {}));

export let TextureUsage;

(function (TextureUsage) {
  TextureUsage[TextureUsage["CopySrc"] = 1] = "CopySrc";
  TextureUsage[TextureUsage["CopyDst"] = 2] = "CopyDst";
  TextureUsage[TextureUsage["Sampled"] = 4] = "Sampled";
  TextureUsage[TextureUsage["Storage"] = 8] = "Storage";
  TextureUsage[TextureUsage["OutputAttachment"] = 16] = "OutputAttachment";
})(TextureUsage || (TextureUsage = {}));
//# sourceMappingURL=constants.js.map