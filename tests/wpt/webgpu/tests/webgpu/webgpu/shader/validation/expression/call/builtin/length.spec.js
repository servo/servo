/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'length';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {

  TypeF16,
  TypeF32,
  elementType,
  kAllFloatScalars,
  kAllFloatVector2,
  kAllFloatVector3,
  kAllFloatVector4,
  kAllIntegerScalarsAndVectors } from
'../../../../../util/conversion.js';
import { isRepresentable } from '../../../../../util/floating_point.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

/**
 * Evaluates the result and information about a call to length(), with a vector
 * formed from `vec` of the element type `type`.
 */
function calculate(
vec,
type)













{
  const squareSum = vec.reduce((prev, curr) => prev + curr * curr, 0);
  const result = Math.sqrt(squareSum);
  return {
    isIntermediateRepresentable: isRepresentable(squareSum, type),
    isResultRepresentable: isRepresentable(result, type),
    result
  };
}

const kScalarTypes = objectsToRecord(kAllFloatScalars);

g.test('scalar').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() with
the input scalar value always compiles without error
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kScalarTypes)).
filter((u) => stageSupportsType(u.stage, kScalarTypes[u.type])).
beginSubcases().
expand('value', (u) => fullRangeForType(kScalarTypes[u.type]))
).
beforeAllSubcases((t) => {
  if (elementType(kScalarTypes[t.params.type]) === TypeF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  // We only validate with numbers known to be representable by the type
  const expectedResult = true;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kScalarTypes[t.params.type].create(t.params.value)],
    t.params.stage
  );
});

const kVec2Types = objectsToRecord(kAllFloatVector2);

g.test('vec2').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() with a vec2 compiles with valid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kVec2Types)).
filter((u) => stageSupportsType(u.stage, kVec2Types[u.type])).
beginSubcases().
expand('x', (u) => fullRangeForType(kVec2Types[u.type], 5)).
expand('y', (u) => fullRangeForType(kVec2Types[u.type], 5)).
expand('_result', (u) => [calculate([u.x, u.y], elementType(kVec2Types[u.type]))]).
filter((u) => u._result.isResultRepresentable === u._result.isIntermediateRepresentable)
).
beforeAllSubcases((t) => {
  if (elementType(kVec2Types[t.params.type]) === TypeF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = t.params._result.isResultRepresentable;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kVec2Types[t.params.type].create([t.params.x, t.params.y])],
    t.params.stage
  );
});

const kVec3Types = objectsToRecord(kAllFloatVector3);

g.test('vec3').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() with a vec3 compiles with valid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kVec3Types)).
filter((u) => stageSupportsType(u.stage, kVec3Types[u.type])).
beginSubcases().
expand('x', (u) => fullRangeForType(kVec3Types[u.type], 4)).
expand('y', (u) => fullRangeForType(kVec3Types[u.type], 4)).
expand('z', (u) => fullRangeForType(kVec3Types[u.type], 4)).
expand('_result', (u) => [calculate([u.x, u.y, u.z], elementType(kVec3Types[u.type]))]).
filter((u) => u._result.isResultRepresentable === u._result.isIntermediateRepresentable)
).
beforeAllSubcases((t) => {
  if (elementType(kVec3Types[t.params.type]) === TypeF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = t.params._result.isResultRepresentable;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kVec3Types[t.params.type].create([t.params.x, t.params.y, t.params.z])],
    t.params.stage
  );
});

const kVec4Types = objectsToRecord(kAllFloatVector4);

g.test('vec4').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() with a vec4 compiles with valid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kVec4Types)).
filter((u) => stageSupportsType(u.stage, kVec4Types[u.type])).
beginSubcases().
expand('x', (u) => fullRangeForType(kVec4Types[u.type], 3)).
expand('y', (u) => fullRangeForType(kVec4Types[u.type], 3)).
expand('z', (u) => fullRangeForType(kVec4Types[u.type], 3)).
expand('w', (u) => fullRangeForType(kVec4Types[u.type], 3)).
expand('_result', (u) => [calculate([u.x, u.y, u.z, u.w], elementType(kVec4Types[u.type]))]).
filter((u) => u._result.isResultRepresentable === u._result.isIntermediateRepresentable)
).
beforeAllSubcases((t) => {
  if (elementType(kVec4Types[t.params.type]) === TypeF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = t.params._result.isResultRepresentable;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kVec4Types[t.params.type].create([t.params.x, t.params.y, t.params.z, t.params.w])],
    t.params.stage
  );
});

const kIntegerArgumentTypes = objectsToRecord([TypeF32, ...kAllIntegerScalarsAndVectors]);

g.test('integer_argument').
desc(
  `
Validates that scalar and vector integer arguments are rejected by ${builtin}()
`
).
params((u) => u.combine('type', keysOf(kIntegerArgumentTypes))).
fn((t) => {
  const type = kIntegerArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */type === TypeF32,
    [type.create(1)],
    'constant'
  );
});