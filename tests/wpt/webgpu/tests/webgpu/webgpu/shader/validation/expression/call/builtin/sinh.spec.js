/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'sinh';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kConvertableToFloatScalarsAndVectors,
  scalarTypeOf } from
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

const kValuesTypes = objectsToRecord(kConvertableToFloatScalarsAndVectors);

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
expand('value', (u) => fullRangeForType(kValuesTypes[u.type]))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kValuesTypes[t.params.type];
  const expectedResult = isRepresentable(
    Math.sinh(Number(t.params.value)),
    // AbstractInt is converted to AbstractFloat before calling into the builtin
    scalarTypeOf(type).kind === 'abstract-int' ? Type.abstractFloat : scalarTypeOf(type)
  );
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.value)],
    t.params.stage
  );
});

const kArgCases = {
  good: '(1.2)',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_2arg: '(1.2, 2.3)',
  // Bad value for arg 0
  bad_0bool: '(false)',
  bad_0array: '(array(1.1,2.2))',
  bad_0struct: '(modf(2.2))',
  bad_0uint: '(1u)',
  bad_0int: '(1i)',
  bad_0vec2i: '(vec2i())',
  bad_0vec2u: '(vec2u())',
  bad_0vec3i: '(vec3i())',
  bad_0vec3u: '(vec3u())',
  bad_0vec4i: '(vec4i())',
  bad_0vec4u: '(vec4u())'
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