/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'exp2';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { kValue } from '../../../../../util/constants.js';
import {
  TypeF16,
  TypeF32,
  elementType,
  kAllFloatScalarsAndVectors,
  kAllIntegerScalarsAndVectors } from
'../../../../../util/conversion.js';
import { isRepresentable } from '../../../../../util/floating_point.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kAllFloatScalarsAndVectors);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() rejects invalid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValuesTypes)).
filter((u) => stageSupportsType(u.stage, kValuesTypes[u.type])).
beginSubcases().
combine('value', [
-1e2,
-1e3,
-4,
-3,
-2,
-1,
-1e-1,
-1e-2,
-1e-3,
0,
1e-3,
1e-2,
1e-1,
1,
2,
3,
4,
1e2,
1e3,
Math.log2(kValue.f16.positive.max) - 0.1,
Math.log2(kValue.f16.positive.max) + 0.1,
Math.log2(kValue.f32.positive.max) - 0.1,
Math.log2(kValue.f32.positive.max) + 0.1]
)
).
beforeAllSubcases((t) => {
  if (elementType(kValuesTypes[t.params.type]) === TypeF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kValuesTypes[t.params.type];
  const expectedResult = isRepresentable(Math.pow(2, t.params.value), elementType(type));
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.value)],
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
    [type.create(0)],
    'constant'
  );
});