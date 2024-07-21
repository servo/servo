/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'step';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kConvertableToFloatScalarsAndVectors,
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

const kValidArgumentTypes = objectsToRecord(kConvertableToFloatScalarsAndVectors);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() error on invalid inputs.
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValidArgumentTypes)).
filter((u) => stageSupportsType(u.stage, kValidArgumentTypes[u.type])).
beginSubcases().
expand('a', (u) => fullRangeForType(kValidArgumentTypes[u.type], 5)).
expand('b', (u) => fullRangeForType(kValidArgumentTypes[u.type], 5))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValidArgumentTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = true;

  const type = kValidArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.a), type.create(t.params.b)],
    t.params.stage
  );
});

const kArgCases = {
  good: '(1.2, 2.3)',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_1arg: '(1.2)',
  bad_3arg: '(1.2, 2.3, 4.5)',
  // Bad value for arg 0
  bad_0bool: '(false, 2.3)',
  bad_0array: '(array(1.1,2.2), 2.3)',
  bad_0struct: '(modf(2.2), 2.3)',
  bad_0uint: '(1u, 2.3)',
  bad_0int: '(1i, 2.3)',
  bad_0vec2i: '(vec2i(), 2.3)',
  bad_0vec2u: '(vec2u(), 2.3)',
  bad_0vec3i: '(vec3i(), 2.3)',
  bad_0vec3u: '(vec3u(), 2.3)',
  bad_0vec4i: '(vec4i(), 2.3)',
  bad_0vec4u: '(vec4u(), 2.3)',
  // Bad value for arg 1
  bad_1bool: '(1.2, false)',
  bad_1array: '(1.2, array(1.1,2.2))',
  bad_1struct: '(1.2, modf(2.2))',
  bad_1uint: '(1.2, 1u)',
  bad_1int: '(1.2, 1i)',
  bad_1vec2i: '(1.2, vec2i())',
  bad_1vec2u: '(1.2, vec2u())',
  bad_1vec3i: '(1.2, vec3i())',
  bad_1vec3u: '(1.2, vec3u())',
  bad_1vec4i: '(1.2, vec4i())',
  bad_1vec4u: '(1.2, vec4u())'
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