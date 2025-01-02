/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureSample';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureSample coords parameter must be correct type
* test textureSample array_index parameter must be correct type
* test textureSample coords parameter must be correct type
* test textureSample offset parameter must be correct type
* test textureSample offset parameter must be a const-expression
* test textureSample offset parameter must be between -8 and +7 inclusive
* test textureSample not usable in a compute or vertex shader
* test textureSample returns the correct type
* test textureSample doesn't work with texture types it's not supposed to

note: uniformity validation is covered in src/webgpu/shader/validation/uniformity/uniformity.spec.ts
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kAllScalarsAndVectors,
  isConvertible,


  isUnsignedType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  kEntryPointsToValidateFragmentOnlyBuiltins,
  kTestTextureTypes } from
'./shader_builtin_utils.js';








const kValidTextureSampleParameterTypes = {
  'texture_1d<f32>': { returnType: Type.vec4f, coordsArgType: Type.f32 },
  'texture_2d<f32>': {
    returnType: Type.vec4f,
    coordsArgType: Type.vec2f,
    offsetArgType: Type.vec2i
  },
  'texture_2d_array<f32>': {
    returnType: Type.vec4f,
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  'texture_3d<f32>': {
    returnType: Type.vec4f,
    coordsArgType: Type.vec3f,
    offsetArgType: Type.vec3i
  },
  'texture_cube<f32>': { returnType: Type.vec4f, coordsArgType: Type.vec3f },
  'texture_cube_array<f32>': {
    returnType: Type.vec4f,
    coordsArgType: Type.vec3f,
    hasArrayIndexArg: true
  },
  texture_depth_2d: { returnType: Type.f32, coordsArgType: Type.vec2f, offsetArgType: Type.vec2i },
  texture_depth_2d_array: {
    returnType: Type.f32,
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  texture_depth_cube: { returnType: Type.f32, coordsArgType: Type.vec3f },
  texture_depth_cube_array: {
    returnType: Type.f32,
    coordsArgType: Type.vec3f,
    hasArrayIndexArg: true
  }
};

const kTextureTypes = keysOf(kValidTextureSampleParameterTypes);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesample').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', keysOf(kValidTextureSampleParameterTypes)).
beginSubcases().
expand('offset', (t) =>
kValidTextureSampleParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { returnType, textureType, offset } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const {
    returnType: returnExpectedType,
    offsetArgType,
    coordsArgType,
    hasArrayIndexArg
  } = kValidTextureSampleParameterTypes[textureType];

  const varWGSL = returnVarType.toString();
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureSample(t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(returnExpectedType, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesample').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureSampleParameterTypes)).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, coordType, offset, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const {
    offsetArgType,
    coordsArgType: coordsRequiredType,
    hasArrayIndexArg
  } = kValidTextureSampleParameterTypes[textureType];

  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSample(t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(coordArgType, coordsRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesample').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no array_index
.filter((t) => !!kValidTextureSampleParameterTypes[t.textureType].hasArrayIndexArg).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, arrayIndexType, value, offset } = t.params;
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgType, offsetArgType } = kValidTextureSampleParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSample(t, s, ${coordWGSL}, ${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesample').
desc(
  `
Validates that only incorrect offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no offset
.filter((t) => !!kValidTextureSampleParameterTypes[t.textureType].offsetArgType).
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
  } = kValidTextureSampleParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = args.map((arg) => arg.wgsl()).join(', ');

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSample(t, s, ${coordWGSL}${arrayWGSL}, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(offsetArgType, offsetRequiredType) && value >= -8 && value <= 7;
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument,non_const').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesample').
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
.filter((t) => !!kValidTextureSampleParameterTypes[t.textureType].offsetArgType)
).
fn((t) => {
  const { textureType, varType } = t.params;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleParameterTypes[textureType];

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
  let v = textureSample(t, s, ${coordWGSL}${arrayWGSL}, ${offsetWGSL});
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
kValidTextureSampleParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, entryPoint, offset } = t.params;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const config = kEntryPointsToValidateFragmentOnlyBuiltins[entryPoint];
  const code = `
${config.code}
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};

fn foo() {
  _ = textureSample(t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
}`;
  t.expectCompileResult(config.expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesample').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
combine('textureType', keysOf(kValidTextureSampleParameterTypes)).
expand('offset', (t) =>
kValidTextureSampleParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { testTextureType, textureType, offset } = t.params;
  const { coordsArgType, offsetArgType, hasArrayIndexArg } =
  kValidTextureSampleParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSample(t, s, ${coordWGSL}${arrayWGSL}${offsetWGSL});
  return vec4f(0);
}
`;

  const types = kValidTextureSampleParameterTypes[testTextureType];
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
      ${t.params.use ? '_ =' : ''} textureSample(t, s, vec2(0,0));
    }`;
  t.expectCompileResult(t.params.use, code);
});