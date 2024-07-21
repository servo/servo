/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { keysOf } from '../../../../../../common/util/data_tables.js';import { assert } from '../../../../../../common/util/util.js';import { Type, scalarTypeOf } from '../../../../../util/conversion.js';

/**
 * Use to test that certain WGSL builtins are only available in the fragment stage.
 * Create WGSL that defines a function "foo" and its required variables that uses
 * the builtin being tested. Append it to these code strings then compile. It should
 * succeed or fail based on the value `expectSuccess`.
 *
 * See ./textureSample.spec.ts was one example
 */
export const kEntryPointsToValidateFragmentOnlyBuiltins = {
  none: {
    expectSuccess: true,
    code: ``
  },
  fragment: {
    expectSuccess: true,
    code: `
      @fragment
      fn main() {
        foo();
      }
    `
  },
  vertex: {
    expectSuccess: false,
    code: `
      @vertex
      fn main() -> @builtin(position) vec4f {
        foo();
        return vec4f();
      }
    `
  },
  compute: {
    expectSuccess: false,
    code: `
      @compute @workgroup_size(1)
      fn main() {
        foo();
      }
    `
  },
  fragment_and_compute: {
    expectSuccess: false,
    code: `
      @fragment
      fn main1() {
        foo();
      }

      @compute @workgroup_size(1)
      fn main2() {
        foo();
      }
    `
  },
  compute_without_call: {
    expectSuccess: true,
    code: `
      @compute @workgroup_size(1)
      fn main() {
      }
    `
  }
};

const kCommonTexelTypes = [Type.vec4f, Type.vec4i, Type.vec4u];
const kDepthTexelTypes = [Type.f32];
const kExternalTexelTypes = [Type.vec4f];








const kCommonTextureTypes = {
  texture_1d: { texelTypes: kCommonTexelTypes },
  texture_2d: { texelTypes: kCommonTexelTypes },
  texture_2d_array: { texelTypes: kCommonTexelTypes },
  texture_3d: { texelTypes: kCommonTexelTypes },
  texture_cube: { texelTypes: kCommonTexelTypes },
  texture_cube_array: { texelTypes: kCommonTexelTypes },
  texture_multisampled_2d: { texelTypes: kCommonTexelTypes }
};

const kDepthTextureTypes = {
  texture_depth_2d: { texelTypes: kDepthTexelTypes, noSuffix: true },
  texture_depth_2d_array: { texelTypes: kDepthTexelTypes, noSuffix: true },
  texture_depth_cube: { texelTypes: kDepthTexelTypes, noSuffix: true },
  texture_depth_cube_array: { texelTypes: kDepthTexelTypes, noSuffix: true },
  texture_depth_multisampled_2d: { texelTypes: kDepthTexelTypes, noSuffix: true }
};

export const kNonStorageTextureTypeInfo = {
  ...kCommonTextureTypes,
  ...kDepthTextureTypes,
  texture_external: { texelTypes: kExternalTexelTypes, noSuffix: true }
};

export const kNonStorageTextureTypes = keysOf(kNonStorageTextureTypeInfo);


/**
 * @returns the WGSL needed to define a texture based on a textureType (eg: 'texture_2d')
 * and a texelType (eg: Type.vec4f) which would return `texture_2d<f32>`
 */
export function getNonStorageTextureTypeWGSL(textureType, texelType) {
  const info = kNonStorageTextureTypeInfo[textureType];
  return info.noSuffix ? textureType : `${textureType}<${scalarTypeOf(texelType)}>`;
}

export const kTestTextureTypes = [
'texture_1d<f32>',
'texture_1d<u32>',
'texture_2d<f32>',
'texture_2d<u32>',
'texture_2d_array<f32>',
'texture_2d_array<u32>',
'texture_3d<f32>',
'texture_3d<u32>',
'texture_cube<f32>',
'texture_cube<u32>',
'texture_cube_array<f32>',
'texture_cube_array<u32>',
'texture_multisampled_2d<f32>',
'texture_multisampled_2d<u32>',
'texture_depth_multisampled_2d',
'texture_external',
'texture_storage_1d<rgba8unorm, read>',
'texture_storage_1d<r32uint, read>',
'texture_storage_2d<rgba8unorm, read>',
'texture_storage_2d<r32uint, read>',
'texture_storage_2d_array<rgba8unorm, read>',
'texture_storage_2d_array<r32uint, read>',
'texture_storage_3d<rgba8unorm, read>',
'texture_storage_3d<r32uint, read>',
'texture_depth_2d',
'texture_depth_2d_array',
'texture_depth_cube',
'texture_depth_cube_array'];


const kTextureTypeSuffixToType = {
  f32: Type.vec4f,
  u32: Type.vec4i,
  'rgba8unorm, read': Type.vec4f,
  'r32uint, read': Type.vec4u
};

/** @returns the base type and sample type for kTestTextureTypes */
export function getSampleAndBaseTextureTypeForTextureType(
textureType)
{
  const match = /^(.*?)<(.*?)>/.exec(textureType);
  const sampleType = match ? kTextureTypeSuffixToType[match[2]] : Type.vec4f;
  assert(!!sampleType);
  return [match ? match[1] : textureType, sampleType];
}