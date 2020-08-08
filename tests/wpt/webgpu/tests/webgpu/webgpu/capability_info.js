/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/

function keysOf(obj) {
  return Object.keys(obj);
}

function numericKeysOf(obj) {
  return Object.keys(obj).map(n => Number(n));
}

// Buffers

export const kBufferUsageInfo = {
  [GPUBufferUsage.MAP_READ]: {},
  [GPUBufferUsage.MAP_WRITE]: {},
  [GPUBufferUsage.COPY_SRC]: {},
  [GPUBufferUsage.COPY_DST]: {},
  [GPUBufferUsage.INDEX]: {},
  [GPUBufferUsage.VERTEX]: {},
  [GPUBufferUsage.UNIFORM]: {},
  [GPUBufferUsage.STORAGE]: {},
  [GPUBufferUsage.INDIRECT]: {},
  [GPUBufferUsage.QUERY_RESOLVE]: {},
};

export const kBufferUsages = numericKeysOf(kBufferUsageInfo);

// Textures

export const kTextureFormatInfo = {
  // Try to keep these manually-formatted in a readable grid.
  // (Note: this list should always match the one in the spec.)

  // 8-bit formats
  r8unorm: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 1,
    blockWidth: 1,
    blockHeight: 1,
  },
  r8snorm: {
    renderable: false,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 1,
    blockWidth: 1,
    blockHeight: 1,
  },
  r8uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 1,
    blockWidth: 1,
    blockHeight: 1,
  },
  r8sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 1,
    blockWidth: 1,
    blockHeight: 1,
  },
  // 16-bit formats
  r16uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 2,
    blockWidth: 1,
    blockHeight: 1,
  },
  r16sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 2,
    blockWidth: 1,
    blockHeight: 1,
  },
  r16float: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 2,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg8unorm: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 2,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg8snorm: {
    renderable: false,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 2,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg8uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 2,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg8sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 2,
    blockWidth: 1,
    blockHeight: 1,
  },
  // 32-bit formats
  r32uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  r32sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  r32float: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg16uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg16sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg16float: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba8unorm: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  'rgba8unorm-srgb': {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba8snorm: {
    renderable: false,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba8uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba8sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  bgra8unorm: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  'bgra8unorm-srgb': {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  // Packed 32-bit formats
  rgb10a2unorm: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg11b10float: {
    renderable: false,
    color: true,
    depth: false,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  // 64-bit formats
  rg32uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 8,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg32sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 8,
    blockWidth: 1,
    blockHeight: 1,
  },
  rg32float: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 8,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba16uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 8,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba16sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 8,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba16float: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 8,
    blockWidth: 1,
    blockHeight: 1,
  },
  // 128-bit formats
  rgba32uint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 16,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba32sint: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 16,
    blockWidth: 1,
    blockHeight: 1,
  },
  rgba32float: {
    renderable: true,
    color: true,
    depth: false,
    stencil: false,
    storage: true,
    copyable: true,
    bytesPerBlock: 16,
    blockWidth: 1,
    blockHeight: 1,
  },
  // Depth/stencil formats
  depth32float: {
    renderable: true,
    color: false,
    depth: true,
    stencil: false,
    storage: false,
    copyable: true,
    bytesPerBlock: 4,
    blockWidth: 1,
    blockHeight: 1,
  },
  depth24plus: {
    renderable: true,
    color: false,
    depth: true,
    stencil: false,
    storage: false,
    copyable: false,
  },
  'depth24plus-stencil8': {
    renderable: true,
    color: false,
    depth: true,
    stencil: true,
    storage: false,
    copyable: false,
  },
};

export const kTextureFormats = keysOf(kTextureFormatInfo);

export const kTextureDimensionInfo = {
  '1d': {},
  '2d': {},
  '3d': {},
};

export const kTextureDimensions = keysOf(kTextureDimensionInfo);

export const kTextureAspectInfo = {
  all: {},
  'depth-only': {},
  'stencil-only': {},
};

export const kTextureAspects = keysOf(kTextureAspectInfo);

export const kTextureUsageInfo = {
  [GPUTextureUsage.COPY_SRC]: {},
  [GPUTextureUsage.COPY_DST]: {},
  [GPUTextureUsage.SAMPLED]: {},
  [GPUTextureUsage.STORAGE]: {},
  [GPUTextureUsage.OUTPUT_ATTACHMENT]: {},
};

export const kTextureUsages = numericKeysOf(kTextureUsageInfo);

export const kTextureComponentTypeInfo = {
  float: {},
  sint: {},
  uint: {},
};

export const kTextureComponentTypes = keysOf(kTextureComponentTypeInfo);

// Texture View

export const kTextureViewDimensionInfo = {
  '1d': { storage: true },
  '2d': { storage: true },
  '2d-array': { storage: true },
  cube: { storage: false },
  'cube-array': { storage: false },
  '3d': { storage: true },
};

export const kTextureViewDimensions = keysOf(kTextureViewDimensionInfo);

// Typedefs for bindings

// Bindings

export const kMaxBindingsPerBindGroup = 16;

export const kPerStageBindingLimits = {
  uniformBuf: { class: 'uniformBuf', max: 12 },
  storageBuf: { class: 'storageBuf', max: 4 },
  sampler: { class: 'sampler', max: 16 },
  sampledTex: { class: 'sampledTex', max: 16 },
  storageTex: { class: 'storageTex', max: 4 },
};

export const kPerPipelineBindingLimits = {
  uniformBuf: { class: 'uniformBuf', maxDynamic: 8 },
  storageBuf: { class: 'storageBuf', maxDynamic: 4 },
  sampler: { class: 'sampler', maxDynamic: 0 },
  sampledTex: { class: 'sampledTex', maxDynamic: 0 },
  storageTex: { class: 'storageTex', maxDynamic: 0 },
};

const kBindableResource = {
  uniformBuf: {},
  storageBuf: {},
  plainSamp: {},
  compareSamp: {},
  sampledTex: {},
  storageTex: {},
  errorBuf: {},
  errorSamp: {},
  errorTex: {},
};

export const kBindableResources = keysOf(kBindableResource);

const kBindingKind = {
  uniformBuf: {
    resource: 'uniformBuf',
    perStageLimitClass: kPerStageBindingLimits.uniformBuf,
    perPipelineLimitClass: kPerPipelineBindingLimits.uniformBuf,
  },
  storageBuf: {
    resource: 'storageBuf',
    perStageLimitClass: kPerStageBindingLimits.storageBuf,
    perPipelineLimitClass: kPerPipelineBindingLimits.storageBuf,
  },
  plainSamp: {
    resource: 'plainSamp',
    perStageLimitClass: kPerStageBindingLimits.sampler,
    perPipelineLimitClass: kPerPipelineBindingLimits.sampler,
  },
  compareSamp: {
    resource: 'compareSamp',
    perStageLimitClass: kPerStageBindingLimits.sampler,
    perPipelineLimitClass: kPerPipelineBindingLimits.sampler,
  },
  sampledTex: {
    resource: 'sampledTex',
    perStageLimitClass: kPerStageBindingLimits.sampledTex,
    perPipelineLimitClass: kPerPipelineBindingLimits.sampledTex,
  },
  storageTex: {
    resource: 'storageTex',
    perStageLimitClass: kPerStageBindingLimits.storageTex,
    perPipelineLimitClass: kPerPipelineBindingLimits.storageTex,
  },
};

// Binding type info

const kValidStagesAll = {
  validStages: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT | GPUShaderStage.COMPUTE,
};

const kValidStagesStorageWrite = { validStages: GPUShaderStage.FRAGMENT | GPUShaderStage.COMPUTE };

export const kBufferBindingTypeInfo = {
  'uniform-buffer': {
    usage: GPUBufferUsage.UNIFORM,
    ...kBindingKind.uniformBuf,
    ...kValidStagesAll,
  },
  'storage-buffer': {
    usage: GPUBufferUsage.STORAGE,
    ...kBindingKind.storageBuf,
    ...kValidStagesStorageWrite,
  },
  'readonly-storage-buffer': {
    usage: GPUBufferUsage.STORAGE,
    ...kBindingKind.storageBuf,
    ...kValidStagesAll,
  },
};

export const kBufferBindingTypes = keysOf(kBufferBindingTypeInfo);

export const kSamplerBindingTypeInfo = {
  sampler: { ...kBindingKind.plainSamp, ...kValidStagesAll },
  'comparison-sampler': { ...kBindingKind.compareSamp, ...kValidStagesAll },
};

export const kSamplerBindingTypes = keysOf(kSamplerBindingTypeInfo);

export const kTextureBindingTypeInfo = {
  'sampled-texture': {
    usage: GPUTextureUsage.SAMPLED,
    ...kBindingKind.sampledTex,
    ...kValidStagesAll,
  },
  'writeonly-storage-texture': {
    usage: GPUTextureUsage.STORAGE,
    ...kBindingKind.storageTex,
    ...kValidStagesStorageWrite,
  },
  'readonly-storage-texture': {
    usage: GPUTextureUsage.STORAGE,
    ...kBindingKind.storageTex,
    ...kValidStagesAll,
  },
};

export const kTextureBindingTypes = keysOf(kTextureBindingTypeInfo);

// All binding types (merged from above)

export const kBindingTypeInfo = {
  ...kBufferBindingTypeInfo,
  ...kSamplerBindingTypeInfo,
  ...kTextureBindingTypeInfo,
};

export const kBindingTypes = keysOf(kBindingTypeInfo);

export const kShaderStages = [
  GPUShaderStage.VERTEX,
  GPUShaderStage.FRAGMENT,
  GPUShaderStage.COMPUTE,
];

export const kShaderStageCombinations = [0, 1, 2, 3, 4, 5, 6, 7];
