/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { keysOf } from '../../common/util/data_tables.js';import { assert } from '../../common/util/util.js';import { align } from '../util/math.js';

const kDefaultArrayLength = 3;

// never is the same as "must not"



export const HostSharableTypes = ['i32', 'u32', 'f16', 'f32'];

// The alignment and size of a host shareable type.
// See  "Alignment and Size" in the WGSL spec.  https://w3.org/TR/WGSL/#alignment-and-size
// Note this is independent of the address space that values of this type might appear in.
// See RequiredAlignOf(...) for the 16-byte granularity requirement when
// values of a type are placed in the uniform address space.







/** Info for each plain scalar type. */
export const kScalarTypeInfo =
{
  'i32': { layout: { alignment: 4, size: 4 }, supportsAtomics: true, arrayLength: 1, innerLength: 0 },
  'u32': { layout: { alignment: 4, size: 4 }, supportsAtomics: true, arrayLength: 1, innerLength: 0 },
  'f16': { layout: { alignment: 2, size: 2 }, supportsAtomics: false, arrayLength: 1, innerLength: 0, feature: 'shader-f16' },
  'f32': { layout: { alignment: 4, size: 4 }, supportsAtomics: false, arrayLength: 1, innerLength: 0 },
  'bool': { layout: undefined, supportsAtomics: false, arrayLength: 1, innerLength: 0 }
};
/** List of all plain scalar types. */
export const kScalarTypes = keysOf(kScalarTypeInfo);

/** Info for each vecN<> container type. */
export const kVectorContainerTypeInfo =
{
  'vec2': { arrayLength: 2, innerLength: 0 },
  'vec3': { arrayLength: 3, innerLength: 0 },
  'vec4': { arrayLength: 4, innerLength: 0 }
};
/** List of all vecN<> container types. */
export const kVectorContainerTypes = keysOf(kVectorContainerTypeInfo);

/** Returns the vector layout for a given vector container and base type, or undefined if that base type has no layout */
function vectorLayout(
vectorContainer,
baseType)
{
  const n = kVectorContainerTypeInfo[vectorContainer].arrayLength;
  const scalarLayout = kScalarTypeInfo[baseType].layout;
  if (scalarLayout === undefined) {
    return undefined;
  }
  if (n === 3) {
    return { alignment: scalarLayout.alignment * 4, size: scalarLayout.size * 3 };
  }
  return { alignment: scalarLayout.alignment * n, size: scalarLayout.size * n };
}

/** Info for each matNxN<> container type. */
export const kMatrixContainerTypeInfo =
{
  'mat2x2': { arrayLength: 2, innerLength: 2 },
  'mat3x2': { arrayLength: 3, innerLength: 2 },
  'mat4x2': { arrayLength: 4, innerLength: 2 },
  'mat2x3': { arrayLength: 2, innerLength: 3 },
  'mat3x3': { arrayLength: 3, innerLength: 3 },
  'mat4x3': { arrayLength: 4, innerLength: 3 },
  'mat2x4': { arrayLength: 2, innerLength: 4 },
  'mat3x4': { arrayLength: 3, innerLength: 4 },
  'mat4x4': { arrayLength: 4, innerLength: 4 }
};
/** List of all matNxN<> container types. */
export const kMatrixContainerTypes = keysOf(kMatrixContainerTypeInfo);

export const kMatrixContainerTypeLayoutInfo =
{
  'f16': {
    'mat2x2': { layout: { alignment: 4, size: 8 } },
    'mat3x2': { layout: { alignment: 4, size: 12 } },
    'mat4x2': { layout: { alignment: 4, size: 16 } },
    'mat2x3': { layout: { alignment: 8, size: 16 } },
    'mat3x3': { layout: { alignment: 8, size: 24 } },
    'mat4x3': { layout: { alignment: 8, size: 32 } },
    'mat2x4': { layout: { alignment: 8, size: 16 } },
    'mat3x4': { layout: { alignment: 8, size: 24 } },
    'mat4x4': { layout: { alignment: 8, size: 32 } }
  },
  'f32': {
    'mat2x2': { layout: { alignment: 8, size: 16 } },
    'mat3x2': { layout: { alignment: 8, size: 24 } },
    'mat4x2': { layout: { alignment: 8, size: 32 } },
    'mat2x3': { layout: { alignment: 16, size: 32 } },
    'mat3x3': { layout: { alignment: 16, size: 48 } },
    'mat4x3': { layout: { alignment: 16, size: 64 } },
    'mat2x4': { layout: { alignment: 16, size: 32 } },
    'mat3x4': { layout: { alignment: 16, size: 48 } },
    'mat4x4': { layout: { alignment: 16, size: 64 } }
  }
};





export const kAccessModeInfo = {
  read: { read: true, write: false },
  write: { read: false, write: true },
  read_write: { read: true, write: true }
};
























export const kAddressSpaceInfo = {
  storage: {
    scope: 'module',
    binding: true,
    spell: 'must',
    accessModes: ['read', 'read_write'],
    spellAccessMode: 'may'
  },
  uniform: {
    scope: 'module',
    binding: true,
    spell: 'must',
    accessModes: ['read'],
    spellAccessMode: 'never'
  },
  private: {
    scope: 'module',
    binding: false,
    spell: 'must',
    accessModes: ['read_write'],
    spellAccessMode: 'never'
  },
  workgroup: {
    scope: 'module',
    binding: false,
    spell: 'must',
    accessModes: ['read_write'],
    spellAccessMode: 'never'
  },
  function: {
    scope: 'function',
    binding: false,
    spell: 'may',
    accessModes: ['read_write'],
    spellAccessMode: 'never'
  },
  handle: {
    scope: 'module',
    binding: true,
    spell: 'never',
    accessModes: [],
    spellAccessMode: 'never'
  }
};

/** List of texel formats and their shader representation */
export const TexelFormats = [
{ format: 'rgba8unorm', _shaderType: 'f32' },
{ format: 'rgba8snorm', _shaderType: 'f32' },
{ format: 'rgba8uint', _shaderType: 'u32' },
{ format: 'rgba8sint', _shaderType: 'i32' },
{ format: 'rgba16uint', _shaderType: 'u32' },
{ format: 'rgba16sint', _shaderType: 'i32' },
{ format: 'rgba16float', _shaderType: 'f32' },
{ format: 'r32uint', _shaderType: 'u32' },
{ format: 'r32sint', _shaderType: 'i32' },
{ format: 'r32float', _shaderType: 'f32' },
{ format: 'rg32uint', _shaderType: 'u32' },
{ format: 'rg32sint', _shaderType: 'i32' },
{ format: 'rg32float', _shaderType: 'f32' },
{ format: 'rgba32uint', _shaderType: 'u32' },
{ format: 'rgba32sint', _shaderType: 'i32' },
{ format: 'rgba32float', _shaderType: 'f32' }];


/**
 * Generate a bunch types (vec, mat, sized/unsized array) for testing.
 */
export function* generateTypes({
  addressSpace,
  baseType,
  containerType,
  isAtomic = false








})
























{
  const scalarInfo = kScalarTypeInfo[baseType];
  if (isAtomic) {
    assert(scalarInfo.supportsAtomics, 'type does not support atomics');
    assert(
      containerType === 'scalar' || containerType === 'array',
      "can only generate atomic inner types with containerType 'scalar' or 'array'"
    );
  }
  const scalarType = isAtomic ? `atomic<${baseType}>` : baseType;

  // Storage and uniform require host-sharable types.
  if (addressSpace === 'storage' || addressSpace === 'uniform') {
    assert(isHostSharable(baseType), 'type ' + baseType.toString() + ' is not host sharable');
  }

  // Scalar types
  if (containerType === 'scalar') {
    yield {
      type: `${scalarType}`,
      _kTypeInfo: {
        elementBaseType: `${scalarType}`,
        ...scalarInfo
      }
    };
  }

  // Vector types
  if (containerType === 'vector') {
    for (const vectorType of kVectorContainerTypes) {
      yield {
        type: `${vectorType}<${scalarType}>`,
        _kTypeInfo: {
          elementBaseType: baseType,
          ...kVectorContainerTypeInfo[vectorType],
          layout: vectorLayout(vectorType, scalarType),
          supportsAtomics: false
        }
      };
    }
  }

  if (containerType === 'matrix') {
    // Matrices can only be f16 or f32.
    if (baseType === 'f16' || baseType === 'f32') {
      for (const matrixType of kMatrixContainerTypes) {
        const matrixDimInfo = kMatrixContainerTypeInfo[matrixType];
        const matrixLayoutInfo = kMatrixContainerTypeLayoutInfo[baseType][matrixType];
        yield {
          type: `${matrixType}<${scalarType}>`,
          _kTypeInfo: {
            elementBaseType: `vec${matrixDimInfo.innerLength}<${scalarType}>`,
            ...matrixDimInfo,
            ...matrixLayoutInfo,
            supportsAtomics: false
          }
        };
      }
    }
  }

  // Array types
  if (containerType === 'array') {
    let arrayElemType = scalarType;
    let arrayElementCount = kDefaultArrayLength;
    let supportsAtomics = scalarInfo.supportsAtomics;
    let layout = undefined;
    if (scalarInfo.layout) {
      // Compute the layout of the array type.
      // Adjust the array element count or element type as needed.
      if (addressSpace === 'uniform') {
        // Use a vec4 of the scalar type, to achieve a 16 byte alignment without internal padding.
        // This works for 4-byte scalar types, and does not work for f16.
        // It is the caller's responsibility to filter out the f16 case.
        assert(!isAtomic, 'the uniform case is making vec4 of scalar, which cannot handle atomics');
        arrayElemType = `vec4<${baseType}>`;
        supportsAtomics = false;
        const arrayElemLayout = vectorLayout('vec4', baseType);
        // assert(arrayElemLayout.alignment % 16 === 0); // Callers responsibility to avoid
        arrayElementCount = align(arrayElementCount, 4) / 4;
        const arrayByteSize = arrayElementCount * arrayElemLayout.size;
        layout = { alignment: arrayElemLayout.alignment, size: arrayByteSize };
      } else {
        // The ordinary case.  Use scalarType as the element type.
        const stride = arrayStride(scalarInfo.layout);
        let arrayByteSize = arrayElementCount * stride;
        if (addressSpace === 'storage') {
          // The buffer effective binding size must be a multiple of 4.
          // Adjust the array element count as needed.
          while (arrayByteSize % 4 > 0) {
            arrayElementCount++;
            arrayByteSize = arrayElementCount * stride;
          }
        }
        layout = { alignment: scalarInfo.layout.alignment, size: arrayByteSize };
      }
    }

    const arrayTypeInfo = {
      elementBaseType: `${baseType}`,
      arrayLength: arrayElementCount,
      layout,
      supportsAtomics
    };

    // Sized
    yield { type: `array<${arrayElemType},${arrayElementCount}>`, _kTypeInfo: arrayTypeInfo };
    // Unsized
    if (addressSpace === 'storage') {
      yield { type: `array<${arrayElemType}>`, _kTypeInfo: arrayTypeInfo };
    }
  }

  function arrayStride(elementLayout) {
    return align(elementLayout.size, elementLayout.alignment);
  }

  function isHostSharable(baseType) {
    for (const sharableType of HostSharableTypes) {
      if (sharableType === baseType) return true;
    }
    return false;
  }
}

/** Atomic access requires scalar/array container type and storage/workgroup memory. */
export function supportsAtomics(p)




{
  return (
    (p.addressSpace === 'storage' && p.storageMode === 'read_write' ||
    p.addressSpace === 'workgroup') && (
    p.containerType === 'scalar' || p.containerType === 'array'));

}

/** Generates an iterator of supported base types (i32/u32/f16/f32/bool) */
export function* supportedScalarTypes(p) {
  for (const scalarType of kScalarTypes) {
    const info = kScalarTypeInfo[scalarType];

    // Test atomics only on supported scalar types.
    if (p.isAtomic && !info.supportsAtomics) continue;

    // Storage and uniform require host-sharable types.
    const isHostShared = p.addressSpace === 'storage' || p.addressSpace === 'uniform';
    if (isHostShared && info.layout === undefined) continue;

    yield scalarType;
  }
}