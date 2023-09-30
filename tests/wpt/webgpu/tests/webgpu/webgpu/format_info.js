/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { keysOf } from '../common/util/data_tables.js';
import { assert } from '../common/util/util.js';
import { align } from './util/math.js';

//
// Texture format tables
//

/**
 * Defaults applied to all texture format tables automatically. Used only inside
 * `formatTableWithDefaults`. This ensures keys are never missing, always explicitly `undefined`.
 *
 * All top-level keys must be defined here, or they won't be exposed at all.
 */
const kFormatUniversalDefaults = {
  blockWidth: undefined,
  blockHeight: undefined,
  color: undefined,
  depth: undefined,
  stencil: undefined,
  colorRender: undefined,
  multisample: undefined,
  feature: undefined,
  baseFormat: undefined,

  sampleType: undefined,
  copySrc: undefined,
  copyDst: undefined,
  bytesPerBlock: undefined,
  renderable: false,
  renderTargetPixelByteCost: undefined,
  renderTargetComponentAlignment: undefined,

  // IMPORTANT:
  // Add new top-level keys both here and in TextureFormatInfo_TypeCheck.
};
/**
 * Takes `table` and applies `defaults` to every row, i.e. for each row,
 * `{ ... kUniversalDefaults, ...defaults, ...row }`.
 * This only operates at the first level; it doesn't support defaults in nested objects.
 */
function formatTableWithDefaults({ defaults, table }) {
  return Object.fromEntries(
    Object.entries(table).map(([k, row]) => [
      k,
      { ...kFormatUniversalDefaults, ...defaults, ...row },
    ])
  );
}

/** "plain color formats", plus rgb9e5ufloat. */
const kRegularTextureFormatInfo = formatTableWithDefaults({
  defaults: { blockWidth: 1, blockHeight: 1, copySrc: true, copyDst: true },
  table: {
    // plain, 8 bits per component

    r8unorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 1 },
      colorRender: { blend: true, resolve: true, byteCost: 1, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    r8snorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 1 },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    r8uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 1 },
      colorRender: { blend: false, resolve: false, byteCost: 1, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    r8sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: false, bytes: 1 },
      colorRender: { blend: false, resolve: false, byteCost: 1, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    rg8unorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      colorRender: { blend: true, resolve: true, byteCost: 2, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg8snorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg8uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg8sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    rgba8unorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: true, bytes: 4 },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      baseFormat: 'rgba8unorm',
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'rgba8unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      baseFormat: 'rgba8unorm',
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgba8snorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: true, bytes: 4 },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgba8uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: true, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgba8sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: true, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    bgra8unorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      baseFormat: 'bgra8unorm',
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bgra8unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      baseFormat: 'bgra8unorm',
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    // plain, 16 bits per component

    r16uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    r16sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    r16float: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      colorRender: { blend: true, resolve: true, byteCost: 2, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    rg16uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg16sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg16float: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: true, resolve: true, byteCost: 4, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    rgba16uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: true, bytes: 8 },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgba16sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: true, bytes: 8 },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgba16float: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: true, bytes: 8 },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 2 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    // plain, 32 bits per component

    r32uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: true, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    r32sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: true, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    r32float: {
      color: { type: 'unfilterable-float', copySrc: true, copyDst: true, storage: true, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    rg32uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: true, bytes: 8 },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg32sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: true, bytes: 8 },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg32float: {
      color: { type: 'unfilterable-float', copySrc: true, copyDst: true, storage: true, bytes: 8 },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    rgba32uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: true, bytes: 16 },
      colorRender: { blend: false, resolve: false, byteCost: 16, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgba32sint: {
      color: { type: 'sint', copySrc: true, copyDst: true, storage: true, bytes: 16 },
      colorRender: { blend: false, resolve: false, byteCost: 16, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgba32float: {
      color: { type: 'unfilterable-float', copySrc: true, copyDst: true, storage: true, bytes: 16 },
      colorRender: { blend: false, resolve: false, byteCost: 16, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    // plain, mixed component width, 32 bits per texel

    rgb10a2uint: {
      color: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rgb10a2unorm: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 4 },
      renderable: true,
      get renderTargetComponentAlignment() {
        return this.colorRender.alignment;
      },
      get renderTargetPixelByteCost() {
        return this.colorRender.byteCost;
      },
      multisample: true,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    rg11b10ufloat: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
      renderTargetPixelByteCost: 8,
      renderTargetComponentAlignment: 4,
    },

    // packed

    rgb9e5ufloat: {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 4 },
      multisample: false,
      get sampleType() {
        return this.color.type;
      },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
  },
});

// MAINTENANCE_TODO: Distinguishing "sized" and "unsized" depth stencil formats doesn't make sense
// because one aspect can be sized and one can be unsized. This should be cleaned up, but is kept
// this way during a migration phase.
const kSizedDepthStencilFormatInfo = formatTableWithDefaults({
  defaults: { blockWidth: 1, blockHeight: 1, multisample: true, copySrc: true, renderable: true },
  table: {
    stencil8: {
      stencil: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 1 },
      sampleType: 'uint',
      copyDst: true,
      bytesPerBlock: 1,
    },
    depth16unorm: {
      depth: { type: 'depth', copySrc: true, copyDst: true, storage: false, bytes: 2 },
      sampleType: 'depth',
      copyDst: true,
      bytesPerBlock: 2,
    },
    depth32float: {
      depth: { type: 'depth', copySrc: true, copyDst: false, storage: false, bytes: 4 },
      sampleType: 'depth',
      copyDst: false,
      bytesPerBlock: 4,
    },
  },
});
const kUnsizedDepthStencilFormatInfo = formatTableWithDefaults({
  defaults: { blockWidth: 1, blockHeight: 1, multisample: true },
  table: {
    depth24plus: {
      depth: { type: 'depth', copySrc: false, copyDst: false, storage: false, bytes: undefined },
      copySrc: false,
      copyDst: false,
      sampleType: 'depth',
      renderable: true,
    },
    'depth24plus-stencil8': {
      depth: { type: 'depth', copySrc: false, copyDst: false, storage: false, bytes: undefined },
      stencil: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 1 },
      copySrc: false,
      copyDst: false,
      sampleType: 'depth',
      renderable: true,
    },
    'depth32float-stencil8': {
      depth: { type: 'depth', copySrc: true, copyDst: false, storage: false, bytes: 4 },
      stencil: { type: 'uint', copySrc: true, copyDst: true, storage: false, bytes: 1 },
      feature: 'depth32float-stencil8',
      copySrc: false,
      copyDst: false,
      sampleType: 'depth',
      renderable: true,
    },
  },
});

const kBCTextureFormatInfo = formatTableWithDefaults({
  defaults: {
    blockWidth: 4,
    blockHeight: 4,
    multisample: false,
    feature: 'texture-compression-bc',
    sampleType: 'float',
    copySrc: true,
    copyDst: true,
  },
  table: {
    'bc1-rgba-unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      baseFormat: 'bc1-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bc1-rgba-unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      baseFormat: 'bc1-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'bc2-rgba-unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'bc2-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bc2-rgba-unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'bc2-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'bc3-rgba-unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'bc3-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bc3-rgba-unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'bc3-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'bc4-r-unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bc4-r-snorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'bc5-rg-unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bc5-rg-snorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'bc6h-rgb-ufloat': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bc6h-rgb-float': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'bc7-rgba-unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'bc7-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'bc7-rgba-unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'bc7-rgba-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
  },
});

const kETC2TextureFormatInfo = formatTableWithDefaults({
  defaults: {
    blockWidth: 4,
    blockHeight: 4,
    multisample: false,
    feature: 'texture-compression-etc2',
    sampleType: 'float',
    copySrc: true,
    copyDst: true,
  },
  table: {
    'etc2-rgb8unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      baseFormat: 'etc2-rgb8unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'etc2-rgb8unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      baseFormat: 'etc2-rgb8unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'etc2-rgb8a1unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      baseFormat: 'etc2-rgb8a1unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'etc2-rgb8a1unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      baseFormat: 'etc2-rgb8a1unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'etc2-rgba8unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'etc2-rgba8unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'etc2-rgba8unorm-srgb': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'etc2-rgba8unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'eac-r11unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'eac-r11snorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 8 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'eac-rg11unorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'eac-rg11snorm': {
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
  },
});

const kASTCTextureFormatInfo = formatTableWithDefaults({
  defaults: {
    multisample: false,
    feature: 'texture-compression-astc',
    sampleType: 'float',
    copySrc: true,
    copyDst: true,
  },
  table: {
    'astc-4x4-unorm': {
      blockWidth: 4,
      blockHeight: 4,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-4x4-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-4x4-unorm-srgb': {
      blockWidth: 4,
      blockHeight: 4,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-4x4-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-5x4-unorm': {
      blockWidth: 5,
      blockHeight: 4,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-5x4-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-5x4-unorm-srgb': {
      blockWidth: 5,
      blockHeight: 4,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-5x4-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-5x5-unorm': {
      blockWidth: 5,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-5x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-5x5-unorm-srgb': {
      blockWidth: 5,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-5x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-6x5-unorm': {
      blockWidth: 6,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-6x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-6x5-unorm-srgb': {
      blockWidth: 6,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-6x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-6x6-unorm': {
      blockWidth: 6,
      blockHeight: 6,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-6x6-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-6x6-unorm-srgb': {
      blockWidth: 6,
      blockHeight: 6,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-6x6-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-8x5-unorm': {
      blockWidth: 8,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-8x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-8x5-unorm-srgb': {
      blockWidth: 8,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-8x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-8x6-unorm': {
      blockWidth: 8,
      blockHeight: 6,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-8x6-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-8x6-unorm-srgb': {
      blockWidth: 8,
      blockHeight: 6,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-8x6-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-8x8-unorm': {
      blockWidth: 8,
      blockHeight: 8,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-8x8-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-8x8-unorm-srgb': {
      blockWidth: 8,
      blockHeight: 8,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-8x8-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-10x5-unorm': {
      blockWidth: 10,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-10x5-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 5,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x5-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-10x6-unorm': {
      blockWidth: 10,
      blockHeight: 6,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x6-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-10x6-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 6,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x6-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-10x8-unorm': {
      blockWidth: 10,
      blockHeight: 8,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x8-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-10x8-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 8,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x8-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-10x10-unorm': {
      blockWidth: 10,
      blockHeight: 10,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x10-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-10x10-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 10,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-10x10-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-12x10-unorm': {
      blockWidth: 12,
      blockHeight: 10,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-12x10-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-12x10-unorm-srgb': {
      blockWidth: 12,
      blockHeight: 10,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-12x10-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },

    'astc-12x12-unorm': {
      blockWidth: 12,
      blockHeight: 12,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-12x12-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
    'astc-12x12-unorm-srgb': {
      blockWidth: 12,
      blockHeight: 12,
      color: { type: 'float', copySrc: true, copyDst: true, storage: false, bytes: 16 },
      baseFormat: 'astc-12x12-unorm',
      get bytesPerBlock() {
        return this.color.bytes;
      },
    },
  },
});

// Definitions for use locally. To access the table entries, use `kTextureFormatInfo`.

// MAINTENANCE_TODO: Consider generating the exports below programmatically by filtering the big list, instead
// of using these local constants? Requires some type magic though.
const kCompressedTextureFormatInfo = {
  ...kBCTextureFormatInfo,
  ...kETC2TextureFormatInfo,
  ...kASTCTextureFormatInfo,
};
const kColorTextureFormatInfo = { ...kRegularTextureFormatInfo, ...kCompressedTextureFormatInfo };
const kEncodableTextureFormatInfo = {
  ...kRegularTextureFormatInfo,
  ...kSizedDepthStencilFormatInfo,
};
const kSizedTextureFormatInfo = {
  ...kRegularTextureFormatInfo,
  ...kSizedDepthStencilFormatInfo,
  ...kCompressedTextureFormatInfo,
};
const kDepthStencilFormatInfo = {
  ...kSizedDepthStencilFormatInfo,
  ...kUnsizedDepthStencilFormatInfo,
};
const kUncompressedTextureFormatInfo = {
  ...kRegularTextureFormatInfo,
  ...kSizedDepthStencilFormatInfo,
  ...kUnsizedDepthStencilFormatInfo,
};
const kAllTextureFormatInfo = {
  ...kUncompressedTextureFormatInfo,
  ...kCompressedTextureFormatInfo,
};

/** A "regular" texture format (uncompressed, sized, single-plane color formats). */

export const kRegularTextureFormats = keysOf(kRegularTextureFormatInfo);
export const kSizedDepthStencilFormats = keysOf(kSizedDepthStencilFormatInfo);
export const kUnsizedDepthStencilFormats = keysOf(kUnsizedDepthStencilFormatInfo);
export const kCompressedTextureFormats = keysOf(kCompressedTextureFormatInfo);

export const kColorTextureFormats = keysOf(kColorTextureFormatInfo);
export const kEncodableTextureFormats = keysOf(kEncodableTextureFormatInfo);
export const kSizedTextureFormats = keysOf(kSizedTextureFormatInfo);
export const kDepthStencilFormats = keysOf(kDepthStencilFormatInfo);
export const kUncompressedTextureFormats = keysOf(kUncompressedTextureFormatInfo);
export const kAllTextureFormats = keysOf(kAllTextureFormatInfo);

// CompressedTextureFormat are unrenderable so filter from RegularTextureFormats for color targets is enough
export const kRenderableColorTextureFormats = kRegularTextureFormats.filter(
  v => kColorTextureFormatInfo[v].colorRender
);

assert(
  kRenderableColorTextureFormats.every(
    f =>
      kAllTextureFormatInfo[f].renderTargetComponentAlignment !== undefined &&
      kAllTextureFormatInfo[f].renderTargetPixelByteCost !== undefined
  )
);

/** Per-GPUTextureFormat-per-aspect info. */

/** Per-GPUTextureFormat info. */
export const kTextureFormatInfo = {
  ...kRegularTextureFormatInfo,
  ...kSizedDepthStencilFormatInfo,
  ...kUnsizedDepthStencilFormatInfo,
  ...kBCTextureFormatInfo,
  ...kETC2TextureFormatInfo,
  ...kASTCTextureFormatInfo,
};

/** Defining this variable verifies the type of kTextureFormatInfo2. It is not used. */

const kTextureFormatInfo_TypeCheck = kTextureFormatInfo;

/** List of all GPUTextureFormat values. */
// MAINTENANCE_TODO: dedup with kAllTextureFormats
export const kTextureFormats = keysOf(kAllTextureFormatInfo);

/** Valid GPUTextureFormats for `copyExternalImageToTexture`, by spec. */
export const kValidTextureFormatsForCopyE2T = [
  'r8unorm',
  'r16float',
  'r32float',
  'rg8unorm',
  'rg16float',
  'rg32float',
  'rgba8unorm',
  'rgba8unorm-srgb',
  'bgra8unorm',
  'bgra8unorm-srgb',
  'rgb10a2unorm',
  'rgba16float',
  'rgba32float',
];

//
// Other related stuff
//

const kDepthStencilFormatCapabilityInBufferTextureCopy = {
  // kUnsizedDepthStencilFormats
  depth24plus: {
    CopyB2T: [],
    CopyT2B: [],
    texelAspectSize: { 'depth-only': -1, 'stencil-only': -1 },
  },
  'depth24plus-stencil8': {
    CopyB2T: ['stencil-only'],
    CopyT2B: ['stencil-only'],
    texelAspectSize: { 'depth-only': -1, 'stencil-only': 1 },
  },

  // kSizedDepthStencilFormats
  depth16unorm: {
    CopyB2T: ['all', 'depth-only'],
    CopyT2B: ['all', 'depth-only'],
    texelAspectSize: { 'depth-only': 2, 'stencil-only': -1 },
  },
  depth32float: {
    CopyB2T: [],
    CopyT2B: ['all', 'depth-only'],
    texelAspectSize: { 'depth-only': 4, 'stencil-only': -1 },
  },
  'depth32float-stencil8': {
    CopyB2T: ['stencil-only'],
    CopyT2B: ['depth-only', 'stencil-only'],
    texelAspectSize: { 'depth-only': 4, 'stencil-only': 1 },
  },
  stencil8: {
    CopyB2T: ['all', 'stencil-only'],
    CopyT2B: ['all', 'stencil-only'],
    texelAspectSize: { 'depth-only': -1, 'stencil-only': 1 },
  },
};

/** `kDepthStencilFormatResolvedAspect[format][aspect]` returns the aspect-specific format for a
 *  depth-stencil format, or `undefined` if the format doesn't have the aspect.
 */
export const kDepthStencilFormatResolvedAspect = {
  // kUnsizedDepthStencilFormats
  depth24plus: {
    all: 'depth24plus',
    'depth-only': 'depth24plus',
    'stencil-only': undefined,
  },
  'depth24plus-stencil8': {
    all: 'depth24plus-stencil8',
    'depth-only': 'depth24plus',
    'stencil-only': 'stencil8',
  },

  // kSizedDepthStencilFormats
  depth16unorm: {
    all: 'depth16unorm',
    'depth-only': 'depth16unorm',
    'stencil-only': undefined,
  },
  depth32float: {
    all: 'depth32float',
    'depth-only': 'depth32float',
    'stencil-only': undefined,
  },
  'depth32float-stencil8': {
    all: 'depth32float-stencil8',
    'depth-only': 'depth32float',
    'stencil-only': 'stencil8',
  },
  stencil8: {
    all: 'stencil8',
    'depth-only': undefined,
    'stencil-only': 'stencil8',
  },
};

/**
 * @returns the GPUTextureFormat corresponding to the @param aspect of @param format.
 * This allows choosing the correct format for depth-stencil aspects when creating pipelines that
 * will have to match the resolved format of views, or to get per-aspect information like the
 * `blockByteSize`.
 *
 * Many helpers use an `undefined` `aspect` to means `'all'` so this is also the default for this
 * function.
 */
export function resolvePerAspectFormat(format, aspect) {
  if (aspect === 'all' || aspect === undefined) {
    return format;
  }
  assert(!!kTextureFormatInfo[format].depth || !!kTextureFormatInfo[format].stencil);
  const resolved = kDepthStencilFormatResolvedAspect[format][aspect ?? 'all'];
  assert(resolved !== undefined);
  return resolved;
}

/**
 * Gets all copyable aspects for copies between texture and buffer for specified depth/stencil format and copy type, by spec.
 */
export function depthStencilFormatCopyableAspects(type, format) {
  const appliedType = type === 'WriteTexture' ? 'CopyB2T' : type;
  return kDepthStencilFormatCapabilityInBufferTextureCopy[format][appliedType];
}

/**
 * Computes whether a copy between a depth/stencil texture aspect and a buffer is supported, by spec.
 */
export function depthStencilBufferTextureCopySupported(type, format, aspect) {
  const supportedAspects = depthStencilFormatCopyableAspects(type, format);

  return supportedAspects.includes(aspect);
}

/**
 * Returns the byte size of the depth or stencil aspect of the specified depth/stencil format,
 * or -1 if none.
 */
export function depthStencilFormatAspectSize(format, aspect) {
  const texelAspectSize =
    kDepthStencilFormatCapabilityInBufferTextureCopy[format].texelAspectSize[aspect];
  assert(texelAspectSize > 0);
  return texelAspectSize;
}

/**
 * Returns true iff a texture can be created with the provided GPUTextureDimension
 * (defaulting to 2d) and GPUTextureFormat, by spec.
 */
export function textureDimensionAndFormatCompatible(dimension, format) {
  const info = kAllTextureFormatInfo[format];
  return !(
    (dimension === '1d' || dimension === '3d') &&
    (info.blockWidth > 1 || info.depth || info.stencil)
  );
}

/**
 * Check if two formats are view format compatible.
 *
 * This function may need to be generalized to use `baseFormat` from `kTextureFormatInfo`.
 */
export function viewCompatible(a, b) {
  return a === b || a + '-srgb' === b || b + '-srgb' === a;
}

export function getFeaturesForFormats(formats) {
  return Array.from(new Set(formats.map(f => (f ? kTextureFormatInfo[f].feature : undefined))));
}

export function filterFormatsByFeature(feature, formats) {
  return formats.filter(f => f === undefined || kTextureFormatInfo[f].feature === feature);
}

export function isCompressedTextureFormat(format) {
  return format in kCompressedTextureFormatInfo;
}

export const kFeaturesForFormats = getFeaturesForFormats(kTextureFormats);

/**
 * Given an array of texture formats return the number of bytes per sample.
 */
export function computeBytesPerSampleFromFormats(formats) {
  let bytesPerSample = 0;
  for (const format of formats) {
    const info = kTextureFormatInfo[format];
    const alignedBytesPerSample = align(bytesPerSample, info.colorRender.alignment);
    bytesPerSample = alignedBytesPerSample + info.colorRender.byteCost;
  }
  return bytesPerSample;
}

/**
 * Given an array of GPUColorTargetState return the number of bytes per sample
 */
export function computeBytesPerSample(targets) {
  return computeBytesPerSampleFromFormats(targets.map(({ format }) => format));
}
