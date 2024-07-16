/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'sign';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kFloatScalarsAndVectors,
  kConcreteSignedIntegerScalarsAndVectors,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord([
...kFloatScalarsAndVectors,
...kConcreteSignedIntegerScalarsAndVectors]
);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() inputs rejects invalid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValuesTypes)).
filter((u) => stageSupportsType(u.stage, kValuesTypes[u.type])).
beginSubcases().
expand('value', (u) => fullRangeForType(kValuesTypes[u.type]))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = true; // Result should always be representable by the type
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kValuesTypes[t.params.type].create(t.params.value)],
    t.params.stage
  );
});

const kArgCases = {
  good: '(1.0)',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_2arg: '(1.0, 1.0)',
  // Bad value for arg 0
  bad_0bool: '(false)',
  bad_0array: '(array(1.1,2.2))',
  bad_0struct: '(modf(2.2))',
  bad_0uint: '(1u)',
  bad_0vec2u: '(vec2u(1))',
  bad_0vec3u: '(vec3u(1))',
  bad_0vec4u: '(vec4u(1))'
};

g.test('args').
desc(`Test compilation failure of ${builtin} with variously shaped and typed arguments`).
params((u) => u.combine('arg', keysOf(kArgCases))).
fn((t) => {
  t.expectCompileResult(
    t.params.arg === 'good',
    `const c = ${builtin}${kArgCases[t.params.arg]};`
  );
});

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}${kArgCases['good']}; }`);
});