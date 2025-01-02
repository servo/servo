/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureSampleBaseClampToEdge';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureSampleBaseClampToEdge coords parameter must be correct type
* test textureSampleBaseClampToEdge returns the correct type
* test textureSampleBaseClampToEdge doesn't work with texture types it's not supposed to
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

const kTextureSampleBaseClampToEdgeTextureTypes = ['texture_2d<f32>', 'texture_external'];
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebaseclamptoedge').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', kTextureSampleBaseClampToEdgeTextureTypes)
).
fn((t) => {
  const { returnType, textureType } = t.params;
  const returnVarType = kValuesTypes[returnType];

  const varWGSL = returnVarType.toString();

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureSampleBaseClampToEdge(t, s, vec2f(0));
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(Type.vec4f, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebaseclamptoedge').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureSampleBaseClampToEdgeTextureTypes).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0)
).
fn((t) => {
  const { textureType, coordType, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const coordWGSL = coordArgType.create(value).wgsl();

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleBaseClampToEdge(t, s, ${coordWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(coordArgType, Type.vec2f);
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplebaseclamptoedge').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) => u.combine('testTextureType', kTestTextureTypes)).
fn((t) => {
  const { testTextureType } = t.params;

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleBaseClampToEdge(t, s, vec2f(0));
  return vec4f(0);
}
`;
  const expectSuccess = kTextureSampleBaseClampToEdgeTextureTypes.includes(testTextureType);
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
      ${t.params.use ? '_ =' : ''} textureSampleBaseClampToEdge(t,s, vec2(0,0));
    }`;
  t.expectCompileResult(t.params.use, code);
});