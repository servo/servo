/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'normalize';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kConcreteIntegerScalarsAndVectors,
  kConvertableToFloatVectors,
  scalarTypeOf } from

'../../../../../util/conversion.js';
import { quantizeToF16, quantizeToF32 } from '../../../../../util/math.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValidArgumentTypes = objectsToRecord(kConvertableToFloatVectors);

function quantizeFunctionForScalarType(type) {
  switch (type) {
    case Type.f32:
      return quantizeToF32;
    case Type.f16:
      return quantizeToF16;
    default:
      return (v) => v;
  }
}

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() rejects invalid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValidArgumentTypes)).
filter((u) => stageSupportsType(u.stage, kValidArgumentTypes[u.type])).
beginSubcases().
expand('value', (u) => fullRangeForType(kValidArgumentTypes[u.type]))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValidArgumentTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  let expectedResult = true;

  const scalarType = scalarTypeOf(kValidArgumentTypes[t.params.type]);
  const quantizeFn = quantizeFunctionForScalarType(scalarType);

  // Should be invalid if the normalization calculations result in intermediate
  // values that exceed the maximum representable float value for the given type,
  // or if the length is smaller than the smallest representable float value.
  const v = Number(t.params.value);
  const vv = quantizeFn(v * v);
  const dp = quantizeFn(vv * kValidArgumentTypes[t.params.type].width);
  const len = quantizeFn(Math.sqrt(dp));
  if (vv === Infinity || dp === Infinity || len === 0) {
    expectedResult = false;
  }

  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kValidArgumentTypes[t.params.type].create(t.params.value)],
    t.params.stage
  );
});

const kInvalidArgumentTypes = objectsToRecord([
Type.f32,
Type.f16,
Type.abstractInt,
Type.bool,
Type.vec(2, Type.bool),
Type.vec(3, Type.bool),
Type.vec(4, Type.bool),
...kConcreteIntegerScalarsAndVectors]
);

g.test('invalid_argument').
desc(
  `
Validates that all scalar arguments and vector integer or boolean arguments are rejected by ${builtin}()
`
).
params((u) => u.combine('type', keysOf(kInvalidArgumentTypes))).
beforeAllSubcases((t) => {
  if (kInvalidArgumentTypes[t.params.type] === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = false; // should always error with invalid argument types
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kInvalidArgumentTypes[t.params.type].create(0)],
    'constant'
  );
});

const kArgCases = {
  good: '(vec3f(1, 0, 0))',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_2args: '(vec3f(),vec3f())',
  // Bad value for arg 0
  bad_0array: '(array(1.1,2.2))',
  bad_0struct: '(modf(2.2))'
};

g.test('args').
desc(`Test compilation failure of ${builtin}  with variously shaped and typed arguments`).
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