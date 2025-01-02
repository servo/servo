/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureSampleCompareLevel';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureSampleCompareLevel coords parameter must be correct type
* test textureSampleCompareLevel array_index parameter must be correct type
* test textureSampleCompareLevel depth_ref parameter must be correct type
* test textureSampleCompareLevel offset parameter must be correct type
* test textureSampleCompareLevel offset parameter must be a const-expression
* test textureSampleCompareLevel offset parameter must be between -8 and +7 inclusive
* test textureSampleCompareLevel returns the correct type
* test textureSampleCompareLevel doesn't work with texture types it's not supposed to
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

import { kTestTextureTypes } from './shader_builtin_utils.js';







const kValidTextureSampleCompareLevelParameterTypes =

{
  texture_depth_2d: { coordsArgType: Type.vec2f, offsetArgType: Type.vec2i },
  texture_depth_2d_array: {
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  texture_depth_cube: { coordsArgType: Type.vec3f },
  texture_depth_cube_array: { coordsArgType: Type.vec3f, hasArrayIndexArg: true }
};

const kTextureTypes = keysOf(kValidTextureSampleCompareLevelParameterTypes);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecomparelevel').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', keysOf(kValidTextureSampleCompareLevelParameterTypes)).
beginSubcases().
expand('offset', (t) =>
kValidTextureSampleCompareLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { returnType, textureType, offset } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const { offsetArgType, coordsArgType, hasArrayIndexArg } =
  kValidTextureSampleCompareLevelParameterTypes[textureType];

  const varWGSL = returnVarType.toString();
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler_comparison;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureSampleCompareLevel(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(Type.f32, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecomparelevel').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureSampleCompareLevelParameterTypes)).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleCompareLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { textureType, coordType, offset, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const {
    offsetArgType,
    coordsArgType: coordsRequiredType,
    hasArrayIndexArg
  } = kValidTextureSampleCompareLevelParameterTypes[textureType];

  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler_comparison;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleCompareLevel(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(coordArgType, coordsRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecomparelevel').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no array_index
.filter((t) => !!kValidTextureSampleCompareLevelParameterTypes[t.textureType].hasArrayIndexArg).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleCompareLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { textureType, arrayIndexType, value, offset } = t.params;
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgType, offsetArgType } =
  kValidTextureSampleCompareLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler_comparison;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleCompareLevel(t, s, ${coordWGSL}, ${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('depth_ref_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecomparelevel').
desc(
  `
Validates that only incorrect depth_ref arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('depthRefType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.depthRefType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleCompareLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { textureType, depthRefType, value, offset } = t.params;
  const depthRefArgType = kValuesTypes[depthRefType];
  const args = [depthRefArgType.create(value)];
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleCompareLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const depthRefWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler_comparison;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleCompareLevel(t, s, ${coordWGSL}${arrayWGSL}, ${depthRefWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(depthRefArgType, Type.f32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecomparelevel').
desc(
  `
Validates that only incorrect offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no offset
.filter((t) => !!kValidTextureSampleCompareLevelParameterTypes[t.textureType].offsetArgType).
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
  } = kValidTextureSampleCompareLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = args.map((arg) => arg.wgsl()).join(', ');

  const code = `
@group(0) @binding(0) var s: sampler_comparison;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleCompareLevel(t, s, ${coordWGSL}${arrayWGSL}, 0, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(offsetArgType, offsetRequiredType) && value >= -8 && value <= 7;
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument,non_const').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecomparelevel').
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
.filter((t) => !!kValidTextureSampleCompareLevelParameterTypes[t.textureType].offsetArgType)
).
fn((t) => {
  const { textureType, varType } = t.params;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleCompareLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = `${offsetArgType}(${varType})`;

  const code = `
@group(0) @binding(0) var s: sampler_comparison;
@group(0) @binding(1) var t: ${textureType};
@group(0) @binding(2) var<uniform> u: ${offsetArgType};
@fragment fn fs() -> @location(0) vec4f {
  const c = 1;
  let l = ${offsetArgType?.create(0).wgsl()};
  let v = textureSampleCompareLevel(t, s, ${coordWGSL}${arrayWGSL}, 0, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = varType === 'c';
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecomparelevel').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
beginSubcases().
combine('textureType', keysOf(kValidTextureSampleCompareLevelParameterTypes)).
expand('offset', (t) =>
kValidTextureSampleCompareLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { testTextureType, textureType, offset } = t.params;
  const { coordsArgType, offsetArgType, hasArrayIndexArg } =
  kValidTextureSampleCompareLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler_comparison;
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleCompareLevel(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;

  const types = kValidTextureSampleCompareLevelParameterTypes[testTextureType];
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
    @group(0) @binding(0) var t : texture_depth_2d;
    @group(0) @binding(1) var s : sampler_comparison;
    fn foo() {
      ${t.params.use ? '_ =' : ''} textureSampleCompareLevel(t, s, vec2(0,0), 0);
    }`;
  t.expectCompileResult(t.params.use, code);
});