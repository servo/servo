/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { isCompatibilityDevice } from '../common/framework/test_config.js';import { keysOf } from '../common/util/data_tables.js';import { assert, unreachable } from '../common/util/util.js';

import { align, roundDown } from './util/math.js';
import { getTextureDimensionFromView } from './util/texture/base.js';


//
// Texture format tables
//

/**
 * Defaults applied to all texture format tables automatically. Used only inside
 * `formatTableWithDefaults`. This ensures keys are never missing, always explicitly `undefined`.
 *
 * All top-level keys must be defined here, or they won't be exposed at all.
 * Documentation is also written here; this makes it propagate through to the end types.
 */
const kFormatUniversalDefaults = {
  /** Texel block width. */
  blockWidth: undefined,
  /** Texel block height. */
  blockHeight: undefined,
  color: undefined,
  depth: undefined,
  stencil: undefined,
  /**
   * Info when this format can be used as a color render target. The format may require a feature
   * to actually be used as a render target. Eg: rg11b10ufloat which requires rg11b10ufloat-renderable
   * Call {@link isTextureFormatPossiblyUsableAsColorRenderAttachment} before having a device
   * Call {@link isTextureFormatColorRenderable}(device, format) to find out for a particular device.
   * Use {@link kPossibleColorRenderableTextureFormats} for params.
   */
  colorRender: undefined,
  /**
   * Whether the format can possibly be used as a multisample texture. The format may require a
   * feature to actually multisampled. Eg: rg11b10ufloat which requires rg11b10ufloat-renderable
   * Call {@link isTextureFormatPossiblyMultisampled} before having a device
   * Call {@link isTextureFormatMultisampled}(device, format) to find out for a particular device.
   * Use {@link kPossibleMultisampledTextureFormats} for params.
   */
  multisample: undefined,
  /** Optional feature required to use this format, or `undefined` if none. */
  feature: undefined,
  /** The base format for srgb formats. Specified on both srgb and equivalent non-srgb formats. */
  baseFormat: undefined,

  /** @deprecated Use `.color.bytes`, `.depth.bytes`, or `.stencil.bytes`. */
  bytesPerBlock: undefined

  // IMPORTANT:
  // Add new top-level keys both here and in TextureFormatInfo_TypeCheck.
};
/**
 * Takes `table` and applies `defaults` to every row, i.e. for each row,
 * `{ ... kUniversalDefaults, ...defaults, ...row }`.
 * This only operates at the first level; it doesn't support defaults in nested objects.
 */
function formatTableWithDefaults({
  defaults,
  table



})







{
  return Object.fromEntries(
    Object.entries(table).map(([k, row]) => [
    k,
    { ...kFormatUniversalDefaults, ...defaults, ...row }]
    )

  );
}

/** "plain color formats", plus rgb9e5ufloat. */
const kRegularTextureFormatInfo = formatTableWithDefaults({
  defaults: { blockWidth: 1, blockHeight: 1 },
  table: {
    // plain, 8 bits per component

    r8unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 1
      },
      colorRender: { blend: true, resolve: true, byteCost: 1, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    r8snorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 1
      },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    r8uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 1
      },
      colorRender: { blend: false, resolve: false, byteCost: 1, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    r8sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 1
      },
      colorRender: { blend: false, resolve: false, byteCost: 1, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },

    rg8unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: true, resolve: true, byteCost: 2, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg8snorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg8uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg8sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },

    rgba8unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      multisample: true,
      baseFormat: 'rgba8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'rgba8unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      multisample: true,
      baseFormat: 'rgba8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba8snorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 4
      },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba8uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba8sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 1 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    bgra8unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      multisample: true,
      baseFormat: 'bgra8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bgra8unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 1 },
      multisample: true,
      baseFormat: 'bgra8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    // plain, 16 bits per component

    r16unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: true, resolve: false, byteCost: 2, alignment: 2 },
      multisample: true,
      feature: 'texture-formats-tier1',
      get bytesPerBlock() {return this.color.bytes;}
    },
    r16snorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: true, resolve: false, byteCost: 2, alignment: 2 },
      multisample: true,
      feature: 'texture-formats-tier1',
      get bytesPerBlock() {return this.color.bytes;}
    },
    r16uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    r16sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: false, resolve: false, byteCost: 2, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    r16float: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      colorRender: { blend: true, resolve: true, byteCost: 2, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },

    rg16unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: false, byteCost: 4, alignment: 2 },
      multisample: true,
      feature: 'texture-formats-tier1',
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg16snorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: false, byteCost: 4, alignment: 2 },
      multisample: true,
      feature: 'texture-formats-tier1',
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg16uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg16sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg16float: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: true, byteCost: 4, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },

    rgba16unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: true, resolve: false, byteCost: 8, alignment: 4 },
      multisample: true,
      feature: 'texture-formats-tier1',
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba16snorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: true, resolve: false, byteCost: 8, alignment: 2 },
      multisample: true,
      feature: 'texture-formats-tier1',
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba16uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba16sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba16float: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 2 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },

    // plain, 32 bits per component

    r32uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: true,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    r32sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: true,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    r32float: {
      color: {
        type: 'unfilterable-float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: true,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 4, alignment: 4 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },

    rg32uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg32sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg32float: {
      color: {
        type: 'unfilterable-float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 8
      },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },

    rgba32uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 16
      },
      colorRender: { blend: false, resolve: false, byteCost: 16, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba32sint: {
      color: {
        type: 'sint',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 16
      },
      colorRender: { blend: false, resolve: false, byteCost: 16, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgba32float: {
      color: {
        type: 'unfilterable-float',
        copySrc: true,
        copyDst: true,
        storage: true,
        readWriteStorage: false,
        bytes: 16
      },
      colorRender: { blend: false, resolve: false, byteCost: 16, alignment: 4 },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    },

    // plain, mixed component width, 32 bits per texel

    rgb10a2uint: {
      color: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: false, resolve: false, byteCost: 8, alignment: 4 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rgb10a2unorm: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 4 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },
    rg11b10ufloat: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      colorRender: { blend: true, resolve: true, byteCost: 8, alignment: 4 },
      multisample: true,
      get bytesPerBlock() {return this.color.bytes;}
    },

    // packed

    rgb9e5ufloat: {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      multisample: false,
      get bytesPerBlock() {return this.color.bytes;}
    }
  }
});

// MAINTENANCE_TODO: Distinguishing "sized" and "unsized" depth stencil formats doesn't make sense
// because one aspect can be sized and one can be unsized. This should be cleaned up, but is kept
// this way during a migration phase.
const kSizedDepthStencilFormatInfo = formatTableWithDefaults({
  defaults: { blockWidth: 1, blockHeight: 1, multisample: true },
  table: {
    stencil8: {
      stencil: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 1
      },
      bytesPerBlock: 1
    },
    depth16unorm: {
      depth: {
        type: 'depth',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 2
      },
      bytesPerBlock: 2
    },
    depth32float: {
      depth: {
        type: 'depth',
        copySrc: true,
        copyDst: false,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      bytesPerBlock: 4
    }
  }
});
const kUnsizedDepthStencilFormatInfo = formatTableWithDefaults({
  defaults: { blockWidth: 1, blockHeight: 1, multisample: true },
  table: {
    depth24plus: {
      depth: {
        type: 'depth',
        copySrc: false,
        copyDst: false,
        storage: false,
        readWriteStorage: false,
        bytes: undefined
      }
    },
    'depth24plus-stencil8': {
      depth: {
        type: 'depth',
        copySrc: false,
        copyDst: false,
        storage: false,
        readWriteStorage: false,
        bytes: undefined
      },
      stencil: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 1
      }
    },
    'depth32float-stencil8': {
      depth: {
        type: 'depth',
        copySrc: true,
        copyDst: false,
        storage: false,
        readWriteStorage: false,
        bytes: 4
      },
      stencil: {
        type: 'uint',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 1
      },
      feature: 'depth32float-stencil8'
    }
  }
});

const kBCTextureFormatInfo = formatTableWithDefaults({
  defaults: {
    blockWidth: 4,
    blockHeight: 4,
    multisample: false,
    feature: 'texture-compression-bc'
  },
  table: {
    'bc1-rgba-unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      baseFormat: 'bc1-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bc1-rgba-unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      baseFormat: 'bc1-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'bc2-rgba-unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'bc2-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bc2-rgba-unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'bc2-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'bc3-rgba-unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'bc3-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bc3-rgba-unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'bc3-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'bc4-r-unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bc4-r-snorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      get bytesPerBlock() {return this.color.bytes;}
    },

    'bc5-rg-unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bc5-rg-snorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      get bytesPerBlock() {return this.color.bytes;}
    },

    'bc6h-rgb-ufloat': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bc6h-rgb-float': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      get bytesPerBlock() {return this.color.bytes;}
    },

    'bc7-rgba-unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'bc7-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'bc7-rgba-unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'bc7-rgba-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    }
  }
});

const kETC2TextureFormatInfo = formatTableWithDefaults({
  defaults: {
    blockWidth: 4,
    blockHeight: 4,
    multisample: false,
    feature: 'texture-compression-etc2'
  },
  table: {
    'etc2-rgb8unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      baseFormat: 'etc2-rgb8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'etc2-rgb8unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      baseFormat: 'etc2-rgb8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'etc2-rgb8a1unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      baseFormat: 'etc2-rgb8a1unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'etc2-rgb8a1unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      baseFormat: 'etc2-rgb8a1unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'etc2-rgba8unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'etc2-rgba8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'etc2-rgba8unorm-srgb': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'etc2-rgba8unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'eac-r11unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      get bytesPerBlock() {return this.color.bytes;}
    },
    'eac-r11snorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 8
      },
      get bytesPerBlock() {return this.color.bytes;}
    },

    'eac-rg11unorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      get bytesPerBlock() {return this.color.bytes;}
    },
    'eac-rg11snorm': {
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      get bytesPerBlock() {return this.color.bytes;}
    }
  }
});

const kASTCTextureFormatInfo = formatTableWithDefaults({
  defaults: {
    multisample: false,
    feature: 'texture-compression-astc'
  },
  table: {
    'astc-4x4-unorm': {
      blockWidth: 4,
      blockHeight: 4,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-4x4-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-4x4-unorm-srgb': {
      blockWidth: 4,
      blockHeight: 4,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-4x4-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-5x4-unorm': {
      blockWidth: 5,
      blockHeight: 4,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-5x4-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-5x4-unorm-srgb': {
      blockWidth: 5,
      blockHeight: 4,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-5x4-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-5x5-unorm': {
      blockWidth: 5,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-5x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-5x5-unorm-srgb': {
      blockWidth: 5,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-5x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-6x5-unorm': {
      blockWidth: 6,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-6x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-6x5-unorm-srgb': {
      blockWidth: 6,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-6x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-6x6-unorm': {
      blockWidth: 6,
      blockHeight: 6,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-6x6-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-6x6-unorm-srgb': {
      blockWidth: 6,
      blockHeight: 6,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-6x6-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-8x5-unorm': {
      blockWidth: 8,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-8x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-8x5-unorm-srgb': {
      blockWidth: 8,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-8x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-8x6-unorm': {
      blockWidth: 8,
      blockHeight: 6,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-8x6-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-8x6-unorm-srgb': {
      blockWidth: 8,
      blockHeight: 6,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-8x6-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-8x8-unorm': {
      blockWidth: 8,
      blockHeight: 8,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-8x8-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-8x8-unorm-srgb': {
      blockWidth: 8,
      blockHeight: 8,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-8x8-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-10x5-unorm': {
      blockWidth: 10,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-10x5-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 5,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x5-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-10x6-unorm': {
      blockWidth: 10,
      blockHeight: 6,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x6-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-10x6-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 6,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x6-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-10x8-unorm': {
      blockWidth: 10,
      blockHeight: 8,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x8-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-10x8-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 8,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x8-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-10x10-unorm': {
      blockWidth: 10,
      blockHeight: 10,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x10-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-10x10-unorm-srgb': {
      blockWidth: 10,
      blockHeight: 10,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-10x10-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-12x10-unorm': {
      blockWidth: 12,
      blockHeight: 10,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-12x10-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-12x10-unorm-srgb': {
      blockWidth: 12,
      blockHeight: 10,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-12x10-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },

    'astc-12x12-unorm': {
      blockWidth: 12,
      blockHeight: 12,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-12x12-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    },
    'astc-12x12-unorm-srgb': {
      blockWidth: 12,
      blockHeight: 12,
      color: {
        type: 'float',
        copySrc: true,
        copyDst: true,
        storage: false,
        readWriteStorage: false,
        bytes: 16
      },
      baseFormat: 'astc-12x12-unorm',
      get bytesPerBlock() {return this.color.bytes;}
    }
  }
});

// Definitions for use locally.

// MAINTENANCE_TODO: Consider generating the exports below programmatically by filtering the big list, instead
// of using these local constants? Requires some type magic though.
const kCompressedTextureFormatInfo = { ...kBCTextureFormatInfo, ...kETC2TextureFormatInfo, ...kASTCTextureFormatInfo };
const kColorTextureFormatInfo = { ...kRegularTextureFormatInfo, ...kCompressedTextureFormatInfo };
const kEncodableTextureFormatInfo = { ...kRegularTextureFormatInfo, ...kSizedDepthStencilFormatInfo };
const kSizedTextureFormatInfo = { ...kRegularTextureFormatInfo, ...kSizedDepthStencilFormatInfo, ...kCompressedTextureFormatInfo };
const kDepthStencilFormatInfo = { ...kSizedDepthStencilFormatInfo, ...kUnsizedDepthStencilFormatInfo };
const kUncompressedTextureFormatInfo = { ...kRegularTextureFormatInfo, ...kSizedDepthStencilFormatInfo, ...kUnsizedDepthStencilFormatInfo };
const kAllTextureFormatInfo = { ...kUncompressedTextureFormatInfo, ...kCompressedTextureFormatInfo };

/** A "regular" texture format (uncompressed, sized, single-plane color formats). */

/** A sized depth/stencil texture format. */

/** An unsized depth/stencil texture format. */

/** A compressed (block) texture format. */


/** A color texture format (regular | compressed). */

/** An encodable texture format (regular | sized depth/stencil). */

/** A sized texture format (regular | sized depth/stencil | compressed). */

/** A depth/stencil format (sized | unsized). */

/** An uncompressed (block size 1x1) format (regular | depth/stencil). */


export const kRegularTextureFormats = keysOf(kRegularTextureFormatInfo);
export const kSizedDepthStencilFormats = keysOf(kSizedDepthStencilFormatInfo);
export const kUnsizedDepthStencilFormats = keysOf(kUnsizedDepthStencilFormatInfo);
export const kCompressedTextureFormats = keysOf(kCompressedTextureFormatInfo);
export const kBCCompressedTextureFormats = keysOf(kBCTextureFormatInfo);
export const kASTCCompressedTextureFormats = keysOf(kASTCTextureFormatInfo);

export const kColorTextureFormats = keysOf(kColorTextureFormatInfo);
export const kEncodableTextureFormats = keysOf(kEncodableTextureFormatInfo);
export const kSizedTextureFormats = keysOf(kSizedTextureFormatInfo);
export const kDepthStencilFormats = keysOf(kDepthStencilFormatInfo);
export const kUncompressedTextureFormats = keysOf(kUncompressedTextureFormatInfo);
export const kAllTextureFormats = keysOf(kAllTextureFormatInfo);

/** Per-GPUTextureFormat-per-aspect info. */












/** Per GPUTextureFormat-per-aspect info for color aspects. */





/** Per GPUTextureFormat-per-aspect info for depth aspects. */




/** Per GPUTextureFormat-per-aspect info for stencil aspects. */





/**
 * Per-GPUTextureFormat info.
 * This is not actually the type of values in kTextureFormatInfo; that type is fully const
 * so that it can be narrowed very precisely at usage sites by the compiler.
 * This type exists only as a type check on the inferred type of kTextureFormatInfo.
 */













































/**
 * DO NOT EXPORT THIS - functions that need info from this table should use the appropriate
 * method for their needs.
 *
 * For a list of textures formats for test parameters there are:
 *
 * Lists of formats that might require features to be enabled
 * * kPossibleColorRenderableTextureFormats
 * * kPossibleStorageTextureFormats
 * * kPossibleReadWriteStorageTextureFormats
 * * kPossibleMultisampledTextureFormats
 *
 * Lists of formats that end in -srgb
 * * kDifferentBaseFormatTextureFormats  (includes compressed textures)
 * * kDifferentBaseFormatRegularTextureFormats (does not include compressed textures)
 *
 * Formats that require a feature to use at all (mostly compressed formats)
 * * kOptionalTextureFormats
 *
 * Misc
 * * kRegularTextureFormats
 * * kSizedDepthStencilFormats
 * * kUnsizedDepthStencilFormats
 * * kCompressedTextureFormats
 * * kUncompressedTextureFormats
 * * kColorTextureFormats - color formats including compressed and sint/uint
 * * kEncodableTextureFormats - formats that TexelView supports.
 * * kSizedTextureFormats - formats that have a known size (so not depth24plus ...)
 * * kDepthStencilFormats - depth, stencil, depth-stencil
 * * kDepthTextureFormats - depth and depth-stencil
 * * kStencilTextureFormats - stencil and depth-stencil
 * * kAllTextureFormats
 *
 * If one of the list above does not work, add a new one or to filter in beforeAllSubcases you generally want to use
 * You will not know if you can actually use a texture for the given use case until the test runs and has a device.
 *
 * * isTextureFormatPossiblyUsableAsRenderAttachment
 * * isTextureFormatPossiblyUsableAsColorRenderAttachment
 * * isTextureFormatPossiblyMultisampled
 * * isTextureFormatPossiblyStorageReadable
 * * isTextureFormatPossiblyStorageReadWritable
 * * isTextureFormatPossiblyFilterableAsTextureF32
 * * isTextureFormatPossiblyUsableWithCopyExternalImageToTexture
 *
 * These are also usable before or during a test
 *
 * * isColorTextureFormat
 * * isDepthTextureFormat
 * * isStencilTextureFormat
 * * isDepthOrStencilTextureFormat
 * * isEncodableTextureFormat
 * * isRegularTextureFormat
 * * isCompressedFloatTextureFormat
 * * isSintOrUintFormat
 *
 * To skip a test use the `skipIfXXX` tests in `GPUTest` if possible. Otherwise these functions
 * require a device to give a correct answer.
 *
 * * isTextureFormatUsableAsRenderAttachment
 * * isTextureFormatColorRenderable
 * * isTextureFormatResolvable
 * * isTextureFormatBlendable
 * * isTextureFormatMultisampled
 * * isTextureFormatUsableAsStorageTexture
 * * isTextureFormatUsableAsReadWriteStorageTexture
 * * isTextureFormatUsableAsStorageFormatInCreateShaderModule
 * * isTextureFormatUsableWithCopyExternalImageToTexture
 *
 * Per-GPUTextureFormat info.
 */
const kTextureFormatInfo = {
  ...kRegularTextureFormatInfo,
  ...kSizedDepthStencilFormatInfo,
  ...kUnsizedDepthStencilFormatInfo,
  ...kBCTextureFormatInfo,
  ...kETC2TextureFormatInfo,
  ...kASTCTextureFormatInfo
};

/** Defining this variable verifies the type of kTextureFormatInfo2. It is not used. */

const kTextureFormatInfo_TypeCheck =

kTextureFormatInfo;

// Depth texture formats including formats that also support stencil
export const kDepthTextureFormats = [
...kDepthStencilFormats.filter((v) => kTextureFormatInfo[v].depth)];

// Stencil texture formats including formats that also support depth
export const kStencilTextureFormats = kDepthStencilFormats.filter(
  (v) => kTextureFormatInfo[v].stencil
);

export const kTextureFormatTier1AllowsResolve = [
'r8snorm',
'rg8snorm',
'rgba8snorm',
'rg11b10ufloat'];


export const kTextureFormatTier1ThrowsWhenNotEnabled = [
'r16unorm',
'r16snorm',
'rg16unorm',
'rg16snorm',
'rgba16unorm',
'rgba16snorm'];


export const kTextureFormatTier1AllowsRenderAttachmentBlendableMultisample =
[
'r16unorm',
'r16snorm',
'rg16unorm',
'rg16snorm',
'rgba16unorm',
'rgba16snorm',
'r8snorm',
'rg8snorm',
'rgba8snorm',
'rg11b10ufloat'];


export const kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly = [
'r8unorm',
'r8snorm',
'r8uint',
'r8sint',
'rg8unorm',
'rg8snorm',
'rg8uint',
'rg8sint',
'r16uint',
'r16sint',
'r16float',
'rg16uint',
'rg16sint',
'rg16float',
'rgb10a2uint',
'rgb10a2unorm',
'rg11b10ufloat'];


export const kTextureFormatsTier2EnablesStorageReadWrite = [
'r8unorm',
'r8uint',
'r8sint',
'rgba8unorm',
'rgba8uint',
'rgba8sint',
'r16uint',
'r16sint',
'r16float',
'rgba16uint',
'rgba16sint',
'rgba16float',
'rgba32uint',
'rgba32sint',
'rgba32float'];


// Texture formats that may possibly be used as a storage texture.
// Some may require certain features to be enabled.
export const kPossibleStorageTextureFormats = [
...kRegularTextureFormats.filter((f) => kTextureFormatInfo[f].color?.storage),
'bgra8unorm',
// these can be used as storage when texture-formats-tier1 is enabled
...kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly];


// Texture formats that may possibly be used as a storage texture.
// Some may require certain features to be enabled.
export const kPossibleReadWriteStorageTextureFormats = [
...kPossibleStorageTextureFormats.filter((f) => kTextureFormatInfo[f].color?.readWriteStorage),
// these can be used as storage when texture-formats-tier2 is enabled
...kTextureFormatsTier2EnablesStorageReadWrite];


// Texture formats that may possibly be multisampled.
// Some may require certain features to be enabled.
export const kPossibleMultisampledTextureFormats = [
...kRegularTextureFormats.filter((f) => kTextureFormatInfo[f].multisample),
...kDepthStencilFormats.filter((f) => kTextureFormatInfo[f].multisample)];


// Texture formats that may possibly be color renderable.
// Some may require certain features to be enabled.
export const kPossibleColorRenderableTextureFormats = [
...kRegularTextureFormats.filter((f) => kTextureFormatInfo[f].colorRender)];




// Texture formats that have a different base format. This is effectively all -srgb formats
// including compressed formats.
export const kDifferentBaseFormatTextureFormats = kColorTextureFormats.filter(
  (f) => kTextureFormatInfo[f].baseFormat && kTextureFormatInfo[f].baseFormat !== f
);

// "Regular" texture formats that have a different base format. This is effectively all -srgb formats
// except compressed formats.
export const kDifferentBaseFormatRegularTextureFormats = kRegularTextureFormats.filter(
  (f) => kTextureFormatInfo[f].baseFormat && kTextureFormatInfo[f].baseFormat !== f
);

// Textures formats that are optional
export const kOptionalTextureFormats = kAllTextureFormats.filter(
  (t) => kTextureFormatInfo[t].feature !== undefined
);

/** Formats added from 'texture-formats-tier1' to be usable with `copyExternalImageToTexture`.
 * DO NOT EXPORT. Use kPossibleValidTextureFormatsForCopyE2T and
 * filter with `isTextureFormatUsableWithCopyExternalImageToTexture`
 * or `GPUTest.skipIfTextureFormatNotUsableWithCopyExternalImageToTexture`
 */
const kValidTextureFormatsForCopyE2TTier1 = [
'r16unorm',
'r16snorm',
'rg16unorm',
'rg16snorm',
'rgba16unorm',
'rgba16snorm',
'r8snorm',
'rg8snorm',
'rgba8snorm',
'rg11b10ufloat'];


/** Possibly Valid GPUTextureFormats for `copyExternalImageToTexture`, by spec. */
export const kPossibleValidTextureFormatsForCopyE2T = [
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
...kValidTextureFormatsForCopyE2TTier1];


/**
 * Valid GPUTextureFormats for `copyExternalImageToTexture` for core and compat.
 * DO NOT EXPORT. Use kPossibleValidTextureFormatsForCopyE2T and
 * filter with `isTextureFormatUsableWithCopyExternalImageToTexture`
 * or `GPUTest.skipIfTextureFormatNotUsableWithCopyExternalImageToTexture`
 */
const kValidTextureFormatsForCopyE2T = [
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
'rgba32float'];


/**
 * Returns true if a texture can be used with copyExternalImageToTexture.
 */
export function isTextureFormatUsableWithCopyExternalImageToTexture(
device,
format)
{
  if (device.features.has('texture-formats-tier1')) {
    if (kValidTextureFormatsForCopyE2TTier1.includes(format)) {
      return true;
    }
  }
  return kValidTextureFormatsForCopyE2T.includes(format);
}

//
// Other related stuff
//

const kDepthStencilFormatCapabilityInBufferTextureCopy = {
  // kUnsizedDepthStencilFormats
  depth24plus: {
    CopyB2T: [],
    CopyT2B: [],
    texelAspectSize: { 'depth-only': -1, 'stencil-only': -1 }
  },
  'depth24plus-stencil8': {
    CopyB2T: ['stencil-only'],
    CopyT2B: ['stencil-only'],
    texelAspectSize: { 'depth-only': -1, 'stencil-only': 1 }
  },

  // kSizedDepthStencilFormats
  depth16unorm: {
    CopyB2T: ['all', 'depth-only'],
    CopyT2B: ['all', 'depth-only'],
    texelAspectSize: { 'depth-only': 2, 'stencil-only': -1 }
  },
  depth32float: {
    CopyB2T: [],
    CopyT2B: ['all', 'depth-only'],
    texelAspectSize: { 'depth-only': 4, 'stencil-only': -1 }
  },
  'depth32float-stencil8': {
    CopyB2T: ['stencil-only'],
    CopyT2B: ['depth-only', 'stencil-only'],
    texelAspectSize: { 'depth-only': 4, 'stencil-only': 1 }
  },
  stencil8: {
    CopyB2T: ['all', 'stencil-only'],
    CopyT2B: ['all', 'stencil-only'],
    texelAspectSize: { 'depth-only': -1, 'stencil-only': 1 }
  }
};

/** `kDepthStencilFormatResolvedAspect[format][aspect]` returns the aspect-specific format for a
 *  depth-stencil format, or `undefined` if the format doesn't have the aspect.
 */
export const kDepthStencilFormatResolvedAspect =



{
  // kUnsizedDepthStencilFormats
  depth24plus: {
    all: 'depth24plus',
    'depth-only': 'depth24plus',
    'stencil-only': undefined
  },
  'depth24plus-stencil8': {
    all: 'depth24plus-stencil8',
    'depth-only': 'depth24plus',
    'stencil-only': 'stencil8'
  },

  // kSizedDepthStencilFormats
  depth16unorm: {
    all: 'depth16unorm',
    'depth-only': 'depth16unorm',
    'stencil-only': undefined
  },
  depth32float: {
    all: 'depth32float',
    'depth-only': 'depth32float',
    'stencil-only': undefined
  },
  'depth32float-stencil8': {
    all: 'depth32float-stencil8',
    'depth-only': 'depth32float',
    'stencil-only': 'stencil8'
  },
  stencil8: {
    all: 'stencil8',
    'depth-only': undefined,
    'stencil-only': 'stencil8'
  }
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
export function resolvePerAspectFormat(
format,
aspect)
{
  if (aspect === 'all' || aspect === undefined) {
    return format;
  }
  assert(!!kTextureFormatInfo[format].depth || !!kTextureFormatInfo[format].stencil);
  const resolved = kDepthStencilFormatResolvedAspect[format][aspect ?? 'all'];
  assert(resolved !== undefined);
  return resolved;
}

/**
 * @returns the sample type of the specified aspect of the specified format.
 */
export function sampleTypeForFormatAndAspect(
format,
aspect)
{
  const info = kTextureFormatInfo[format];
  if (info.color) {
    assert(aspect === 'all', `color format ${format} used with aspect ${aspect}`);
    return info.color.type;
  } else if (info.depth && info.stencil) {
    if (aspect === 'depth-only') {
      return info.depth.type;
    } else if (aspect === 'stencil-only') {
      return info.stencil.type;
    } else {
      unreachable(`depth-stencil format ${format} used with aspect ${aspect}`);
    }
  } else if (info.depth) {
    assert(aspect !== 'stencil-only', `depth-only format ${format} used with aspect ${aspect}`);
    return info.depth.type;
  } else if (info.stencil) {
    assert(aspect !== 'depth-only', `stencil-only format ${format} used with aspect ${aspect}`);
    return info.stencil.type;
  }
  unreachable();
}

/**
 * Gets all copyable aspects for copies between texture and buffer for specified depth/stencil format and copy type, by spec.
 */
export function depthStencilFormatCopyableAspects(
type,
format)
{
  const appliedType = type === 'WriteTexture' ? 'CopyB2T' : type;
  return kDepthStencilFormatCapabilityInBufferTextureCopy[format][appliedType];
}

/**
 * Computes whether a copy between a depth/stencil texture aspect and a buffer is supported, by spec.
 */
export function depthStencilBufferTextureCopySupported(
type,
format,
aspect)
{
  const supportedAspects = depthStencilFormatCopyableAspects(
    type,
    format
  );
  return supportedAspects.includes(aspect);
}

/**
 * Returns the byte size of the depth or stencil aspect of the specified depth/stencil format,
 * or -1 if none.
 */
export function depthStencilFormatAspectSize(
format,
aspect)
{
  const texelAspectSize =
  kDepthStencilFormatCapabilityInBufferTextureCopy[format].texelAspectSize[aspect];
  assert(texelAspectSize > 0);
  return texelAspectSize;
}

/**
 * Returns true iff a texture can be created with the provided GPUTextureDimension
 * (defaulting to 2d) and GPUTextureFormat, by spec.
 */
export function textureFormatAndDimensionPossiblyCompatible(
dimension,
format)
{
  if (dimension === '3d' && (isBCTextureFormat(format) || isASTCTextureFormat(format))) {
    return true;
  }
  const info = kAllTextureFormatInfo[format];
  return !(
  (dimension === '1d' || dimension === '3d') && (
  info.blockWidth > 1 || info.depth || info.stencil));

}

/**
 * Returns true iff a texture can be created with the provided GPUTextureDimension
 * (defaulting to 2d) and GPUTextureFormat for a GPU device, by spec.
 */
export function textureDimensionAndFormatCompatibleForDevice(
device,
dimension,
format)
{
  if (
  dimension === '3d' && (
  isBCTextureFormat(format) && device.features.has('texture-compression-bc-sliced-3d') ||
  isASTCTextureFormat(format) && device.features.has('texture-compression-astc-sliced-3d')))
  {
    return true;
  }
  const info = kAllTextureFormatInfo[format];
  return !(
  (dimension === '1d' || dimension === '3d') && (
  info.blockWidth > 1 || info.depth || info.stencil));

}

/**
 * Returns true iff a texture can be used with the provided GPUTextureViewDimension
 */
export function textureViewDimensionAndFormatCompatibleForDevice(
device,
dimension,
format)
{
  return textureDimensionAndFormatCompatibleForDevice(
    device,
    getTextureDimensionFromView(dimension),
    format
  );
}

/**
 * Check if two formats are view format compatible.
 */
export function textureFormatsAreViewCompatible(
device,
a,
b)
{
  return isCompatibilityDevice(device) ?
  a === b :
  a === b || a + '-srgb' === b || b + '-srgb' === a;
}

/**
 * Gets the block width, height, and bytes per block for a color texture format.
 * This is for color textures only. For all texture formats @see {@link getBlockInfoForTextureFormat}
 * The point of this function is bytesPerBlock is always defined so no need to check that it's not
 * vs getBlockInfoForTextureFormat where it may not be defined.
 */
export function getBlockInfoForColorTextureFormat(format) {
  const info = kTextureFormatInfo[format];
  return {
    blockWidth: info.blockWidth,
    blockHeight: info.blockHeight,
    bytesPerBlock: info.color?.bytes
  };
}

/**
 * Gets the block width, height, and bytes per block for a sized texture format.
 * This is for sized textures only. For all texture formats @see {@link getBlockInfoForTextureFormat}
 * The point of this function is bytesPerBlock is always defined so no need to check that it's not
 * vs getBlockInfoForTextureFormat where it may not be defined.
 */
export function getBlockInfoForSizedTextureFormat(format) {
  const info = kTextureFormatInfo[format];
  const bytesPerBlock = info.color?.bytes || info.depth?.bytes || info.stencil?.bytes;
  assert(!!bytesPerBlock);
  return {
    blockWidth: info.blockWidth,
    blockHeight: info.blockHeight,
    bytesPerBlock
  };
}

/**
 * Gets the block width, height, and bytes per block for an encodable texture format.
 * This is for encodable textures only. For all texture formats @see {@link getBlockInfoForTextureFormat}
 * The point of this function is bytesPerBlock is always defined so no need to check that it's not
 * vs getBlockInfoForTextureFormat where it may not be defined.
 */
export function getBlockInfoForEncodableTextureFormat(format) {
  const info = kTextureFormatInfo[format];
  const bytesPerBlock = info.color?.bytes || info.depth?.bytes || info.stencil?.bytes;
  assert(!!bytesPerBlock);
  return {
    blockWidth: info.blockWidth,
    blockHeight: info.blockHeight,
    bytesPerBlock
  };
}

/**
 * Gets the block width, height, and bytes per block for a color texture format.
 * Note that bytesPerBlock will be undefined if format's size is undefined.
 * If you are only using color or encodable formats, @see {@link getBlockInfoForColorTextureFormat}
 * or {@link getBlockInfoForEncodableTextureFormat}
 */
export function getBlockInfoForTextureFormat(format) {
  const info = kTextureFormatInfo[format];
  return {
    blockWidth: info.blockWidth,
    blockHeight: info.blockHeight,
    bytesPerBlock: info.color?.bytes ?? info.depth?.bytes ?? info.stencil?.bytes
  };
}

/**
 * Returns the "byteCost" of rendering to a color texture format.
 */
export function getColorRenderByteCost(format) {
  const byteCost = kTextureFormatInfo[format].colorRender?.byteCost;
  // MAINTENANCE_TODO: remove this assert. The issue is typescript thinks
  // PossibleColorRenderTextureFormat contains all texture formats and not just
  // a filtered list.
  assert(byteCost !== undefined);
  return byteCost;
}

/**
 * Returns the "alignment" of rendering to a color texture format.
 */
export function getColorRenderAlignment(format) {
  const alignment = kTextureFormatInfo[format].colorRender?.alignment;
  // MAINTENANCE_TODO: remove this assert. The issue is typescript thinks
  // PossibleColorRenderTextureFormat contains all texture formats and not just
  // a filtered list.
  assert(alignment !== undefined);
  return alignment;
}

/**
 * Gets the baseFormat for a texture format.
 */
export function getBaseFormatForTextureFormat(
format)
{
  return kTextureFormatInfo[format].baseFormat;
}

export function getBaseFormatForRegularTextureFormat(
format)
{
  return kTextureFormatInfo[format].baseFormat;
}

/**
 * Gets the feature needed for a give texture format or undefined if none.
 */
export function getRequiredFeatureForTextureFormat(format) {
  return kTextureFormatInfo[format].feature;
}

export function getFeaturesForFormats(
formats)
{
  return Array.from(new Set(formats.map((f) => f ? kTextureFormatInfo[f].feature : undefined)));
}

export function filterFormatsByFeature(
feature,
formats)
{
  return formats.filter((f) => f === undefined || kTextureFormatInfo[f].feature === feature);
}

function isTextureFormatTier1EnablesRenderAttachmentBlendableMultisample(format) {
  return kTextureFormatTier1AllowsRenderAttachmentBlendableMultisample.includes(
    format
  );
}

function isTextureFormatTier1EnablesResolve(format) {
  return kTextureFormatTier1AllowsResolve.includes(format);
}

function isTextureFormatTier1EnablesStorageReadOnlyWriteOnly(format) {
  return kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly.includes(format);
}

function isTextureFormatTier2EnablesStorageReadWrite(format) {
  return kTextureFormatsTier2EnablesStorageReadWrite.includes(format);
}

export function canCopyToAspectOfTextureFormat(format, aspect) {
  const info = kTextureFormatInfo[format];
  switch (aspect) {
    case 'depth-only':
      assert(isDepthTextureFormat(format));
      return info.depth && info.depth.copyDst;
    case 'stencil-only':
      assert(isStencilTextureFormat(format));
      return info.stencil && info.stencil.copyDst;
    case 'all':
      return (
        (!isDepthTextureFormat(format) || info.depth?.copyDst) && (
        !isStencilTextureFormat(format) || info.stencil?.copyDst) && (
        !isColorTextureFormat(format) || !info.color?.copyDst));

  }
}

export function canCopyFromAspectOfTextureFormat(
format,
aspect)
{
  const info = kTextureFormatInfo[format];
  switch (aspect) {
    case 'depth-only':
      assert(isDepthTextureFormat(format));
      return info.depth && info.depth.copySrc;
    case 'stencil-only':
      assert(isStencilTextureFormat(format));
      return info.stencil && info.stencil.copySrc;
    case 'all':
      return (
        (!isDepthTextureFormat(format) || info.depth?.copySrc) && (
        !isStencilTextureFormat(format) || info.stencil?.copySrc) && (
        !isColorTextureFormat(format) || !info.color?.copySrc));

  }
}

/**
 * Returns true if all aspects of texture can be copied to (used with COPY_DST)
 */
export function canCopyToAllAspectsOfTextureFormat(format) {
  const info = kTextureFormatInfo[format];
  return (
    (!info.color || info.color.copyDst) && (
    !info.depth || info.depth.copyDst) && (
    !info.stencil || info.stencil.copyDst));

}

/**
 * Returns true if all aspects of texture can be copied from (used with COPY_SRC)
 */
export function canCopyFromAllAspectsOfTextureFormat(format) {
  const info = kTextureFormatInfo[format];
  return (
    (!info.color || info.color.copySrc) && (
    !info.depth || info.depth.copySrc) && (
    !info.stencil || info.stencil.copySrc));

}

export function isCompressedTextureFormat(format) {
  return format in kCompressedTextureFormatInfo;
}

export function isBCTextureFormat(format) {
  return format in kBCTextureFormatInfo;
}

export function isASTCTextureFormat(format) {
  return format in kASTCTextureFormatInfo;
}

export function isColorTextureFormat(format) {
  return !!kTextureFormatInfo[format].color;
}

export function isDepthTextureFormat(format) {
  return !!kTextureFormatInfo[format].depth;
}

export function isStencilTextureFormat(format) {
  return !!kTextureFormatInfo[format].stencil;
}

export function isDepthOrStencilTextureFormat(format) {
  return isDepthTextureFormat(format) || isStencilTextureFormat(format);
}

export function isEncodableTextureFormat(format) {
  return kEncodableTextureFormats.includes(format);
}

/**
 * Returns if a texture can be used as a render attachment. some color formats and all
 * depth textures and stencil textures are usable with usage RENDER_ATTACHMENT.
 */
export function isTextureFormatUsableAsRenderAttachment(
device,
format)
{
  if (format === 'rg11b10ufloat') {
    return device.features.has('rg11b10ufloat-renderable');
  }
  if (isTextureFormatTier1EnablesRenderAttachmentBlendableMultisample(format)) {
    return device.features.has('texture-formats-tier1');
  }
  return kTextureFormatInfo[format].colorRender || isDepthOrStencilTextureFormat(format);
}

/**
 * Returns if a texture can be used as a "colorAttachment".
 */
export function isTextureFormatColorRenderable(
device,
format)
{
  if (format === 'rg11b10ufloat') {
    return device.features.has('rg11b10ufloat-renderable');
  }
  if (isTextureFormatTier1EnablesRenderAttachmentBlendableMultisample(format)) {
    return device.features.has('texture-formats-tier1');
  }
  return !!kAllTextureFormatInfo[format].colorRender;
}

/**
 * Returns if a texture can be blended.
 */
export function isTextureFormatBlendable(device, format) {
  if (!isTextureFormatColorRenderable(device, format)) {
    return false;
  }
  if (format === 'rg11b10ufloat') {
    return device.features.has('rg11b10ufloat-renderable');
  }
  if (is32Float(format)) {
    return device.features.has('float32-blendable');
  }
  return !!kAllTextureFormatInfo[format].colorRender?.blend;
}

/**
 * Returns the texture's type (float, unsigned-float, sint, uint, depth)
 */
export function getTextureFormatType(format) {
  const info = kTextureFormatInfo[format];
  const type = info.color?.type ?? info.depth?.type ?? info.stencil?.type;
  assert(!!type);
  return type;
}

/**
 * Returns the regular texture's type (float, unsigned-float, sint, uint)
 */
export function getTextureFormatColorType(format) {
  const info = kTextureFormatInfo[format];
  const type = info.color?.type;
  assert(!!type);
  return type;
}

/**
 * Returns true if a texture can possibly be used as a render attachment.
 * The texture may require certain features to be enabled.
 */
export function isTextureFormatPossiblyUsableAsRenderAttachment(format) {
  const info = kTextureFormatInfo[format];
  return (
    isDepthOrStencilTextureFormat(format) ||
    !!info.colorRender ||
    isTextureFormatTier1EnablesRenderAttachmentBlendableMultisample(format));

}

/**
 * Returns true if a texture can possibly be used as a color render attachment.
 * The texture may require certain features to be enabled.
 */
export function isTextureFormatPossiblyUsableAsColorRenderAttachment(format) {
  const info = kTextureFormatInfo[format];
  return (
    !!info.colorRender || isTextureFormatTier1EnablesRenderAttachmentBlendableMultisample(format));

}

/**
 * Returns true if a texture can possibly be used multisampled.
 * The texture may require certain features to be enabled.
 */
export function isTextureFormatPossiblyMultisampled(format) {
  const info = kTextureFormatInfo[format];
  return (
    info.multisample || isTextureFormatTier1EnablesRenderAttachmentBlendableMultisample(format));

}

/**
 * Returns true if a texture can possibly be used as a storage texture.
 * The texture may require certain features to be enabled.
 */
export function isTextureFormatPossiblyStorageReadable(format) {
  return (
    !!kTextureFormatInfo[format].color?.storage ||
    isTextureFormatTier1EnablesStorageReadOnlyWriteOnly(format));

}

/**
 * Returns true if a texture can possibly be used as a read-write storage texture.
 * The texture may require certain features to be enabled.
 */
export function isTextureFormatPossiblyStorageReadWritable(format) {
  return (
    !!kTextureFormatInfo[format].color?.readWriteStorage ||
    isTextureFormatTier2EnablesStorageReadWrite(format));

}

export function is16Float(format) {
  return format === 'r16float' || format === 'rg16float' || format === 'rgba16float';
}

export function is32Float(format) {
  return format === 'r32float' || format === 'rg32float' || format === 'rgba32float';
}

/**
 * Returns true if texture is filterable as `texture_xxx<f32>`
 *
 * examples:
 * * 'rgba8unorm' -> true
 * * 'depth16unorm' -> false
 * * 'rgba32float' -> true (you need to enable feature 'float32-filterable')
 */
export function isTextureFormatPossiblyFilterableAsTextureF32(format) {
  const info = kTextureFormatInfo[format];
  return info.color?.type === 'float' || is32Float(format);
}

export const kCompatModeUnsupportedStorageTextureFormats = [
'rg32float',
'rg32sint',
'rg32uint'];


/**
 * Return true if the format can be used as a write only storage texture.
 * Note: Some formats can be compiled in a shader but can not be used
 * in a pipeline or elsewhere. This function returns whether or not the format
 * can be used in general. If you want to know if the format can used when compiling
 * a shader @see {@link isTextureFormatUsableAsStorageFormatInCreateShaderModule}
 */
function isTextureFormatUsableAsWriteOnlyStorageTexture(
device,
format)
{
  if (isCompatibilityDevice(device)) {
    if (kCompatModeUnsupportedStorageTextureFormats.indexOf(format) >= 0) {
      return false;
    }
  }
  if (format === 'bgra8unorm' && device.features.has('bgra8unorm-storage')) {
    return true;
  }
  if (
  isTextureFormatTier1EnablesStorageReadOnlyWriteOnly(format) &&
  device.features.has('texture-formats-tier1'))
  {
    return true;
  }
  const info = kTextureFormatInfo[format];
  return !!(info.color?.storage || info.depth?.storage || info.stencil?.storage);
}

/**
 * Return true if the format can be used with the given access mode
 * access can be either GPUStorageTextureAccess or WGSL access
 * Note: Some formats can be compiled in a shader but can not be used
 * in a pipeline or elsewhere. This function returns whether or not the format
 * can be used in general. If you want to know if the format can used when compiling
 * a shader @see {@link isTextureFormatUsableAsStorageFormatInCreateShaderModule}
 */
export function isTextureFormatUsableWithStorageAccessMode(
device,
format,
access)
{
  switch (access) {
    case 'read':
    case 'read-only':
      return isTextureFormatUsableAsReadOnlyStorageTexture(device, format);
    case 'write':
    case 'write-only':
      return isTextureFormatUsableAsWriteOnlyStorageTexture(device, format);
    case 'read_write':
    case 'read-write':
      return isTextureFormatUsableAsReadWriteStorageTexture(device, format);
  }
}

/**
 * Return true if the format can be used as a read only storage texture.
 * Note: Some formats can be compiled in a shader but can not be used
 * in a pipeline or elsewhere. This function returns whether or not the format
 * can be used in general. If you want to know if the format can used when compiling
 * a shader @see {@link isTextureFormatUsableAsStorageFormatInCreateShaderModule}
 */
function isTextureFormatUsableAsReadOnlyStorageTexture(
device,
format)
{
  // This is the only storage texture format that isn't readable as a storage format.
  if (format === 'bgra8unorm') {
    return false;
  }
  // All other formats that can be used as a storage texture can be used as
  // both read-only and write-only.
  return isTextureFormatUsableAsWriteOnlyStorageTexture(device, format);
}

/**
 * Returns true if format can be used with createShaderModule on the device.
 * Some formats may require a feature to be enabled before they can be used
 * as a storage texture. Others, can't be used in a pipeline but can be compiled
 * in a shader. Examples are rg32float, rg32uint, rg32sint which are not usable
 * in compat mode but shaders can be compiled. Similarly, bgra8unorm can be
 * compiled but can't be used in a pipeline unless feature 'bgra8unorm-storage'
 * is available.
 */
export function isTextureFormatUsableAsStorageFormatInCreateShaderModule(
device,
format)
{
  return kPossibleStorageTextureFormats.includes(
    format
  );
}

function isTextureFormatUsableAsReadWriteStorageTexture(
device,
format)
{
  if (isTextureFormatTier2EnablesStorageReadWrite(format)) {
    return device.features.has('texture-formats-tier2');
  }
  return !!kTextureFormatInfo[format].color?.readWriteStorage;
}

export function isRegularTextureFormat(format) {
  return format in kRegularTextureFormatInfo;
}

/**
 * Returns true if format is both compressed and a float format, for example 'bc6h-rgb-ufloat'.
 */
export function isCompressedFloatTextureFormat(format) {
  return isCompressedTextureFormat(format) && format.includes('float');
}

/**
 * Returns true if format is sint or uint
 */
export function isSintOrUintFormat(format) {
  const info = kTextureFormatInfo[format];
  const type = info.color?.type ?? info.depth?.type ?? info.stencil?.type;
  return type === 'sint' || type === 'uint';
}

/**
 * Returns true if format can be multisampled.
 */
export const kCompatModeUnsupportedMultisampledTextureFormats = [
'r8uint',
'r8sint',
'rg8uint',
'rg8sint',
'rgba8uint',
'rgba8sint',
'r16uint',
'r16sint',
'rg16uint',
'rg16sint',
'rgba16uint',
'rgba16sint',
'rgb10a2uint',
'rgba16float',
'r32float'];


/**
 * Returns true if you can make a multisampled texture from the given format.
 */
export function isTextureFormatMultisampled(device, format) {
  if (isCompatibilityDevice(device)) {
    if (kCompatModeUnsupportedMultisampledTextureFormats.indexOf(format) >= 0) {
      return false;
    }
  }
  if (format === 'rg11b10ufloat') {
    return device.features.has('rg11b10ufloat-renderable');
  }
  if (isTextureFormatTier1EnablesRenderAttachmentBlendableMultisample(format)) {
    return device.features.has('texture-formats-tier1');
  }
  return kAllTextureFormatInfo[format].multisample;
}

/**
 * Returns true if a texture can be "resolved". uint/sint formats can be multisampled but
 * can not be resolved.
 */
export function isTextureFormatResolvable(device, format) {
  if (format === 'rg11b10ufloat') {
    return device.features.has('rg11b10ufloat-renderable');
  }
  if (isTextureFormatTier1EnablesResolve(format)) {
    return device.features.has('texture-formats-tier1');
  }
  // You can't resolve a non-multisampled format.
  if (!isTextureFormatMultisampled(device, format)) {
    return false;
  }
  const info = kAllTextureFormatInfo[format];
  return !!info.colorRender?.resolve;
}

// MAINTENANCE_TODD: See if we can remove this. This doesn't seem useful since
// formats are not on/off by feature. Some are on but a feature allows them to be
// used in more cases, like going from un-renderable to renderable, etc...
export const kFeaturesForFormats = getFeaturesForFormats(kAllTextureFormats);

/**
 * Given an array of texture formats return the number of bytes per sample.
 */
export function computeBytesPerSampleFromFormats(formats) {
  let bytesPerSample = 0;
  for (const format of formats) {
    // MAINTENANCE_TODO: Add colorRender to rg11b10ufloat format in kTextureFormatInfo
    // The issue is if we add it now lots of tests will break as they'll think they can
    // render to the format but are not enabling 'rg11b10ufloat-renderable'. Once we
    // get the CTS refactored (see issue 4181), then fix this.
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

/**
 * Returns the maximum valid size in each dimension for a given texture format.
 * This is useful because compressed formats must be a multiple of blocks in size
 * so, for example, the largest valid width of a 2d texture
 * roundDown(device.limits.maxTextureDimension2D, blockWidth)
 */
export function getMaxValidTextureSizeForFormatAndDimension(
device,
format,
dimension)
{
  const info = getBlockInfoForTextureFormat(format);
  switch (dimension) {
    case '1d':
      return [device.limits.maxTextureDimension1D, 1, 1];
    case '2d':
      return [
      roundDown(device.limits.maxTextureDimension2D, info.blockWidth),
      roundDown(device.limits.maxTextureDimension2D, info.blockHeight),
      device.limits.maxTextureArrayLayers];

    case '3d':
      return [
      roundDown(device.limits.maxTextureDimension3D, info.blockWidth),
      roundDown(device.limits.maxTextureDimension3D, info.blockHeight),
      device.limits.maxTextureDimension3D];

  }
}