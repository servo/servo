/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'unpack2x16float';export const description = `
Validation tests for the ${builtin} builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { kBit, kValue } from '../../../../../util/constants.js';
import {
  kAllScalarsAndVectors,
  u32,
  i32,
  f32,
  f16,
  bool,
  vec2,
  vec3,
  vec4,
  array } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

const kAllValueTypes = objectsToRecord(kAllScalarsAndVectors);
const kValidArgumentTypes = ['u32', 'abstract-int'];
const kReturnType = 'vec2<f32>';
const kArgCases = {
  good: [u32(1)],
  bad_no_args: [],
  bad_more_args: [u32(1), u32(2)],
  bad_i32: [i32(1)],
  bad_f32: [f32(1)],
  bad_f16: [f16(1)],
  bad_bool: [bool(false)],
  bad_vec2u: [vec2(u32(1), u32(2))],
  bad_vec3u: [vec3(u32(1), u32(2), u32(3))],
  bad_vec4u: [vec4(u32(1), u32(2), u32(3), u32(4))],
  bad_array: [array(u32(1))]
};

// Return true if and only if the given value is of valid f16 bit pattern represented as uint16, and is not NaN nor Inf.
const isValidF16AsU16 = (f16AsU16) => {
  return (
    Number.isInteger(f16AsU16) && (
    kBit.f16.positive.zero <= f16AsU16 && f16AsU16 < kBit.f16.positive.infinity ||
    kBit.f16.negative.zero <= f16AsU16 && f16AsU16 < kBit.f16.negative.infinity));

};

const isValidPacked2xF16 = (packed2xF16AsU32) => {
  return (
    Number.isInteger(packed2xF16AsU32) &&
    packed2xF16AsU32 >= kValue.u32.min &&
    packed2xF16AsU32 <= kValue.u32.max &&
    isValidF16AsU16(packed2xF16AsU32 & 0xffff) &&
    isValidF16AsU16(packed2xF16AsU32 >>> 16));

};

export const g = makeTestGroup(ShaderValidationTest);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin} rejects invalid values.
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', kValidArgumentTypes).
filter((u) => stageSupportsType(u.stage, kAllValueTypes[u.type])).
beginSubcases().
expand('value', (u) => fullRangeForType(kAllValueTypes[u.type]))
).
fn((t) => {
  const type = kAllValueTypes[t.params.type];
  const value = t.params.value;
  const expectedResult = isValidPacked2xF16(Number(value));
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(value)],
    t.params.stage
  );
});

g.test('arguments').
desc(`Test that ${builtin} is validated correctly when called with different arguments.`).
params((u) =>
u.
combine('args', keysOf(kArgCases)).
beginSubcases().
expand('returnType', (u) => u.args.includes('good') ? keysOf(kAllValueTypes) : [kReturnType])
).
beforeAllSubcases((t) => {
  if (t.params.args.includes('f16') || t.params.returnType?.toString().includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = t.params.args.includes('good') && t.params.returnType === kReturnType;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    kArgCases[t.params.args],
    'constant',
    kAllValueTypes[t.params.returnType]
  );
});

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}(1u); }`);
});