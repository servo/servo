/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'pow';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kConcreteIntegerScalarsAndVectors,
  kConvertableToFloatScalarsAndVectors,
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

const kValidArgumentTypes = objectsToRecord(kConvertableToFloatScalarsAndVectors);

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

TODO(http://github.com/gpuweb/issues/4527): This validation matches what is currently in Tint but
it needs to be clarified in the spec if this is the desired behavior.
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
  let expectedResult = true;

  const a = Number(t.params.a);
  const b = Number(t.params.b);
  if (a < 0 || a === 0 && b <= 0) {
    expectedResult = false;
  }

  if (expectedResult) {
    const scalarType = scalarTypeOf(kValidArgumentTypes[t.params.type]);
    const quantizeFn = quantizeFunctionForScalarType(scalarType);

    // Should be invalid if the pow calculation results in values that exceed
    // the maximum representable float value for the given type
    const p = quantizeFn(Math.pow(a, b));
    if (!Number.isFinite(p)) {
      expectedResult = false;
    }
  }

  const type = kValidArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.a), type.create(t.params.b)],
    t.params.stage
  );
});

const kInvalidArgumentTypes = objectsToRecord([
Type.bool,
Type.vec(2, Type.bool),
Type.vec(3, Type.bool),
Type.vec(4, Type.bool),
...kConcreteIntegerScalarsAndVectors]
);

g.test('invalid_argument').
desc(
  `
Validates that all integer or boolean scalar and vector arguments are rejected by ${builtin}()
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
  const type = kInvalidArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(1), type.create(2)],
    'constant'
  );
});

const kArgCases = {
  good: '(2.0, 2.0)',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_1args: '(2.0)',
  bad_3args: '(2.0,2.0,2.0)',
  // Bad value for arg 0
  bad_0bool: '(false, 2.0)',
  bad_0array: '(array(1.1,2.2), 2.0)',
  bad_0struct: '(modf(2.2), 2.0)',
  bad_0uint: '(1u, 2.0)',
  bad_0int: '(1i, 2.0)',
  bad_0vec2i: '(vec2i(), 2.0)',
  bad_0vec2u: '(vec2u(), 2.0)',
  bad_0vec3i: '(vec3i(), 2.0)',
  bad_0vec3u: '(vec3u(), 2.0)',
  bad_0vec4i: '(vec4i(), 2.0)',
  bad_0vec4u: '(vec4u(), 2.0)',
  // Bad value for arg 1
  bad_1bool: '(2.0, false)',
  bad_1array: '(2.0, array(1.1,2.2))',
  bad_1struct: '(2.0, modf(2.2))',
  bad_1uint: '(2.0, 1u)',
  bad_1int: '(2.0, 1i)',
  bad_1vec2i: '(2.0, vec2i())',
  bad_1vec2u: '(2.0, vec2u())',
  bad_1vec3i: '(2.0, vec3i())',
  bad_1vec3u: '(2.0, vec3u())',
  bad_1vec4i: '(2.0, vec4i())',
  bad_1vec4u: '(2.0, vec4u())'
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