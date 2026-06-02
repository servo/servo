/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureSampleGrad';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureSampleGrad coords parameter must be correct type
* test textureSampleGrad array_index parameter must be correct type
* test textureSampleGrad ddX parameter must be correct type
* test textureSampleGrad ddY parameter must be correct type
* test textureSampleGrad coords parameter must be correct type
* test textureSampleGrad offset parameter must be correct type
* test textureSampleGrad offset parameter must be a const-expression
* test textureSampleGrad offset parameter must be between -8 and +7 inclusive
* test textureSampleGrad returns the correct type
* test textureSampleGrad doesn't work with texture types it's not supposed to
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

// Note: ddX and ddy parameter types match coords so we'll use coordsArgType for ddX and ddY.






const kValidTextureSampleGradParameterTypes = {
  'texture_2d<f32>': {
    coordsArgType: Type.vec2f,
    offsetArgType: Type.vec2i
  },
  'texture_2d_array<f32>': {
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  'texture_3d<f32>': { coordsArgType: Type.vec3f, offsetArgType: Type.vec3i },
  'texture_cube<f32>': { coordsArgType: Type.vec3f },
  'texture_cube_array<f32>': { coordsArgType: Type.vec3f, hasArrayIndexArg: true }
};

const kTextureTypes = keysOf(kValidTextureSampleGradParameterTypes);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', keysOf(kValidTextureSampleGradParameterTypes)).
beginSubcases().
expand('offset', (t) =>
kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { returnType, textureType, offset } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const { offsetArgType, coordsArgType, hasArrayIndexArg } =
  kValidTextureSampleGradParameterTypes[textureType];

  const varWGSL = returnVarType.toString();
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const ddWGSL = coordsArgType.create(0).wgsl();
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureSampleGrad(t, s, ${coordWGSL}${arrayWGSL}, ${ddWGSL}, ${ddWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(Type.vec4f, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureSampleGradParameterTypes)).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, coordType, offset, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const {
    offsetArgType,
    coordsArgType: coordsRequiredType,
    hasArrayIndexArg
  } = kValidTextureSampleGradParameterTypes[textureType];

  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const ddWGSL = coordsRequiredType.create(0).wgsl();
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleGrad(t, s, ${coordWGSL}${arrayWGSL}, ${ddWGSL}, ${ddWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(coordArgType, coordsRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no array_index
.filter((t) => !!kValidTextureSampleGradParameterTypes[t.textureType].hasArrayIndexArg).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, arrayIndexType, value, offset } = t.params;
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgType, offsetArgType } = kValidTextureSampleGradParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const ddWGSL = coordsArgType.create(0).wgsl();
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleGrad(t, s, ${coordWGSL}, ${arrayWGSL}, ${ddWGSL}, ${ddWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('ddX_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
desc(
  `
Validates that only incorrect ddX arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('ddxType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.ddxType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, ddxType, value, offset } = t.params;
  const ddxArgType = kValuesTypes[ddxType];
  const args = [ddxArgType.create(value)];
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleGradParameterTypes[textureType];

  const ddxRequiredType = coordsArgType;
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const ddXWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const ddYWGSL = coordsArgType.create(0).wgsl();
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleGrad(t, s, ${coordWGSL}${arrayWGSL}, ${ddXWGSL}, ${ddYWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(ddxArgType, ddxRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('ddY_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
desc(
  `
Validates that only incorrect ddY arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('ddyType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.ddyType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { textureType, ddyType, value, offset } = t.params;
  const ddyArgType = kValuesTypes[ddyType];
  const args = [ddyArgType.create(value)];
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleGradParameterTypes[textureType];

  const ddyRequiredType = coordsArgType;
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const ddXWGSL = coordsArgType.create(0).wgsl();
  const ddYWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleGrad(t, s, ${coordWGSL}${arrayWGSL}, ${ddXWGSL}, ${ddYWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(ddyArgType, ddyRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
desc(
  `
Validates that only incorrect offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no offset
.filter((t) => !!kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType).
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
  } = kValidTextureSampleGradParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const ddWGSL = coordsArgType.create(0).wgsl();
  const offsetWGSL = args.map((arg) => arg.wgsl()).join(', ');

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleGrad(t, s, ${coordWGSL}${arrayWGSL}, ${ddWGSL}, ${ddWGSL}, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(offsetArgType, offsetRequiredType) && value >= -8 && value <= 7;
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument,non_const').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
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
.filter((t) => !!kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType)
).
fn((t) => {
  const { textureType, varType } = t.params;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleGradParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const ddWGSL = coordsArgType.create(0).wgsl();
  const offsetWGSL = `${offsetArgType}(${varType})`;

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@group(0) @binding(2) var<uniform> u: ${offsetArgType};
@fragment fn fs() -> @location(0) vec4f {
  const c = 1;
  let l = ${offsetArgType.create(0).wgsl()};
  let v = textureSampleGrad(t, s, ${coordWGSL}${arrayWGSL}, ${ddWGSL}, ${ddWGSL}, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = varType === 'c';
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplegrad').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
combine('textureType', keysOf(kValidTextureSampleGradParameterTypes)).
beginSubcases().
expand('offset', (t) =>
kValidTextureSampleGradParameterTypes[t.textureType].offsetArgType ? [false, true] : [false]
)
).
fn((t) => {
  const { testTextureType, textureType, offset } = t.params;
  const { coordsArgType, offsetArgType, hasArrayIndexArg } =
  kValidTextureSampleGradParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const ddWGSL = coordsArgType.create(0).wgsl();
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleGrad(t, s, ${coordWGSL}${arrayWGSL}, ${ddWGSL}, ${ddWGSL}${offsetWGSL});
  return vec4f(0);
}
`;

  const types = kValidTextureSampleGradParameterTypes[testTextureType];
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
      ${t.params.use ? '_ =' : ''} textureSampleGrad(t,s,vec2(0,0),vec2(0,0),vec2(0,0));
    }`;
  t.expectCompileResult(t.params.use, code);
});