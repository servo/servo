/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'length';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {

  Type,
  kConcreteIntegerScalarsAndVectors,
  kConvertableToFloatScalar,
  kConvertableToFloatVec2,
  kConvertableToFloatVec3,
  kConvertableToFloatVec4,
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

/**
 * Evaluates the result and information about a call to length(), with a vector
 * formed from `vec` of the element type `type`.
 */
function calculate(
vec,
type)













{
  const vec_number = vec.map((e) => Number(e));
  const squareSum = vec_number.reduce((prev, curr) => prev + Number(curr) * Number(curr), 0);
  const result = Math.sqrt(squareSum);
  return {
    isIntermediateRepresentable: isRepresentable(
      squareSum,
      // AbstractInt is converted to AbstractFloat before calling into the builtin
      scalarTypeOf(type).kind === 'abstract-int' ? Type.abstractFloat : scalarTypeOf(type)
    ),
    isResultRepresentable: isRepresentable(
      result,
      // AbstractInt is converted to AbstractFloat before calling into the builtin
      scalarTypeOf(type).kind === 'abstract-int' ? Type.abstractFloat : scalarTypeOf(type)
    ),
    result
  };
}

const kScalarTypes = objectsToRecord(kConvertableToFloatScalar);

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
  if (scalarTypeOf(kScalarTypes[t.params.type]) === Type.f16) {
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

const kVec2Types = objectsToRecord(kConvertableToFloatVec2);

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
expand('_result', (u) => [calculate([u.x, u.y], scalarTypeOf(kVec2Types[u.type]))]).
filter((u) => u._result.isResultRepresentable === u._result.isIntermediateRepresentable)
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kVec2Types[t.params.type]) === Type.f16) {
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

const kVec3Types = objectsToRecord(kConvertableToFloatVec3);

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
expand('_result', (u) => [calculate([u.x, u.y, u.z], scalarTypeOf(kVec3Types[u.type]))]).
filter((u) => u._result.isResultRepresentable === u._result.isIntermediateRepresentable)
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kVec3Types[t.params.type]) === Type.f16) {
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

const kVec4Types = objectsToRecord(kConvertableToFloatVec4);

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
expand('_result', (u) => [calculate([u.x, u.y, u.z, u.w], scalarTypeOf(kVec4Types[u.type]))]).
filter((u) => u._result.isResultRepresentable === u._result.isIntermediateRepresentable)
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kVec4Types[t.params.type]) === Type.f16) {
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

const kIntegerArgumentTypes = objectsToRecord([Type.f32, ...kConcreteIntegerScalarsAndVectors]);

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
    /* expectedResult */type === Type.f32,
    [type.create(1)],
    'constant'
  );
});

const kArgCases = {
  good: '(1.1)',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_2args: '(1.0,2.0)',
  // Bad value type for arg 0
  bad_0i32: '(1i)',
  bad_0u32: '(1u)',
  bad_0bool: '(false)',
  bad_0vec2u: '(vec2u())',
  bad_0mat: '(mat2x2f())',
  bad_0array: '(array(1.1,2.2))',
  bad_0struct: '(modf(2.2))'
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