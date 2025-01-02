/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureSampleBias';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureSampleBias coords parameter must be correct type
* test textureSampleBias array_index parameter must be correct type
* test textureSampleBias bias parameter must be correct type
* test textureSampleBias bias parameter must be between -16.0 and 15.99 inclusive if it's a constant
* test textureSampleBias offset parameter must be correct type
* test textureSampleBias offset parameter must be a const-expression
* test textureSampleBias offset parameter must be between -8 and +7 inclusive
* test textureSampleBias returns the correct type
* test textureSampleBias doesn't work with texture types it's not supposed to

note: uniformity validation is covered in src/webgpu/shader/validation/uniformity/uniformity.spec.ts
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kAllScalarsAndVectors,
  isConvertible,


  isUnsignedType,
  scalarTypeOf,
  isFloatType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  kEntryPointsToValidateFragmentOnlyBuiltins,
  kTestTextureTypes } from
'./shader_builtin_utils.js';







const kValidTextureSampleBiasParameterTypes = {
  'texture_2d<f32>': { coordsArgType: Type.vec2f, offsetArgType: Type.vec2i },
  'texture_2d_array<f32>': {
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  'texture_3d<f32>': { coordsArgType: Type.vec3f, offsetArgType: Type.vec3i },
  'texture_cube<f32>': { coordsArgType: Type.vec3f },
  'texture_cube_array<f32>': { coordsArgType: Type.vec3f, hasArrayIndexArg: true }
};

const kTextureTypes = keysOf(kValidTextureSampleBiasParameterTypes);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebias').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', keysOf(kValidTextureSampleBiasParameterTypes)).
beginSubcases().
expand('offset', (t) =>
kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { returnType, textureType, offset } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const { offsetArgType, coordsArgType, hasArrayIndexArg } =
  kValidTextureSampleBiasParameterTypes[textureType];

  const varWGSL = returnVarType.toString();
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureSampleBias(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(Type.vec4f, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebias').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureSampleBiasParameterTypes)).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, coordType, offset, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const {
    offsetArgType,
    coordsArgType: coordsRequiredType,
    hasArrayIndexArg
  } = kValidTextureSampleBiasParameterTypes[textureType];

  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleBias(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(coordArgType, coordsRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebias').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no array_index
.filter((t) => !!kValidTextureSampleBiasParameterTypes[t.textureType].hasArrayIndexArg).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, arrayIndexType, value, offset } = t.params;
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgType, offsetArgType } = kValidTextureSampleBiasParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleBias(t, s, ${coordWGSL}, ${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('bias_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebias').
desc(
  `
Validates that only incorrect bias arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no offset
.filter((t) => !!kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType).
combine('biasType', keysOf(kValuesTypes)).
beginSubcases()
// The spec mentions limits of > -16 and < 15.99 so pass some values around there
// No error is mentioned for out of range values so make sure no error is generated.
.combine('value', [-17, -16, -8, 0, 7, 15.99, 16])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.biasType]) || t.value >= 0)
// filter out non-integer values passed to integer types.
.filter((t) => Number.isInteger(t.value) || isFloatType(scalarTypeOf(kValuesTypes[t.biasType]))).
expand('offset', (t) =>
kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, biasType, value, offset } = t.params;
  const biasArgType = kValuesTypes[biasType];
  const args = [biasArgType.create(value)];
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleBiasParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const biasWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleBias(t, s, ${coordWGSL}${arrayWGSL}, ${biasWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(biasArgType, Type.f32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebias').
desc(
  `
Validates that only incorrect offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no offset
.filter((t) => !!kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType).
combine('offsetType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.offsetType]) || t.value >= 0)
).
fn((t) => {
  const { textureType, offsetType, value } = t.params;
  const offsetArgType = kValuesTypes[offsetType];
  const args = [offsetArgType.create(value)];
  const {
    coordsArgType,
    hasArrayIndexArg,
    offsetArgType: offsetRequiredType
  } = kValidTextureSampleBiasParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = args.map((arg) => arg.wgsl()).join(', ');

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleBias(t, s, ${coordWGSL}${arrayWGSL}, 0, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(offsetArgType, offsetRequiredType) && value >= -8 && value <= 7;
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument,non_const').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebias').
desc(
  `
Validates that only non-const offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('varType', ['c', 'u', 'l'])
// filter out types with no offset
.filter((t) => !!kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType)
).
fn((t) => {
  const { textureType, varType } = t.params;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleBiasParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = `${offsetArgType}(${varType})`;

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@group(0) @binding(2) var<uniform> u: ${offsetArgType};
@fragment fn fs() -> @location(0) vec4f {
  const c = 1;
  let l = ${offsetArgType.create(0).wgsl()};
  let v = textureSampleBias(t, s, ${coordWGSL}${arrayWGSL}, 0, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = varType === 'c';
  t.expectCompileResult(expectSuccess, code);
});

g.test('only_in_fragment').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesample').
desc(
  `
Validates that ${builtin} must not be used in a compute or vertex shader.
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('entryPoint', keysOf(kEntryPointsToValidateFragmentOnlyBuiltins)).
expand('offset', (t) =>
kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, entryPoint, offset } = t.params;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleBiasParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const config = kEntryPointsToValidateFragmentOnlyBuiltins[entryPoint];
  const code = `
${config.code}
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};

fn foo() {
  _ = textureSampleBias(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
}`;
  t.expectCompileResult(config.expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebias').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
combine('textureType', keysOf(kValidTextureSampleBiasParameterTypes)).
beginSubcases().
expand('offset', (t) =>
kValidTextureSampleBiasParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { testTextureType, textureType, offset } = t.params;
  const { coordsArgType, offsetArgType, hasArrayIndexArg } =
  kValidTextureSampleBiasParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleBias(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;

  const types = kValidTextureSampleBiasParameterTypes[testTextureType];
  const typesMatch = types ?
  types.coordsArgType === coordsArgType &&
  types.hasArrayIndexArg === hasArrayIndexArg && (
  offset ? types.offsetArgType === offsetArgType : true) :
  false;

  const expectSuccess = testTextureType === textureType || typesMatch;
  t.expectCompileResult(expectSuccess, code);
});

g.test('must_use').
desc('Tests that the result must be used').
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const code = `
    @group(0) @binding(0) var t : texture_2d<f32>;
    @group(0) @binding(1) var s : sampler;
    fn foo() {
      ${t.params.use ? '_ =' : ''} textureSampleBias(t, s, vec2(0,0), 0);
    }`;
  t.expectCompileResult(t.params.use, code);
});