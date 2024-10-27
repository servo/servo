/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for division and remainder expressions.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../common/util/data_tables.js';
import { assert } from '../../../../../common/util/util.js';
import { kBit } from '../../../../util/constants.js';
import {
  ScalarType,
  Type,

  VectorType,
  concreteTypeOf,
  isAbstractType,
  isConvertible,
  kAllScalarsAndVectors,
  kConcreteNumericScalarsAndVectors,
  scalarTypeOf } from
'../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';
import {
  kConstantAndOverrideStages,
  validateConstOrOverrideBinaryOpEval } from
'../call/builtin/const_override_validation.js';

import { resultType } from './result_type.js';

export const g = makeTestGroup(ShaderValidationTest);

// A list of operators tested in this file.
const kOperators = {
  div: { op: '/' },
  rem: { op: '%' }
};

// A list of scalar and vector types.
const kScalarAndVectorTypes = objectsToRecord(kAllScalarsAndVectors);
const kConcreteNumericScalarAndVectorTypes = objectsToRecord(kConcreteNumericScalarsAndVectors);

g.test('scalar_vector').
desc(
  `
  Validates that scalar and vector expressions are only accepted for compatible numeric types.
  `
).
params((u) =>
u.
combine('lhs', keysOf(kScalarAndVectorTypes)).
combine(
  'rhs',
  // Skip vec3 and vec4 on the RHS to keep the number of subcases down.
  // vec3 + vec3 and vec4 + vec4 is tested in execution tests.
  keysOf(kScalarAndVectorTypes).filter(
    (value) => !(value.startsWith('vec3') || value.startsWith('vec4'))
  )
).
combine('compound_assignment', [false, true]).
beginSubcases().
combine('op', keysOf(kOperators)).
combine('rhs_value', [0, 1])
).
beforeAllSubcases((t) => {
  if (
  scalarTypeOf(kScalarAndVectorTypes[t.params.lhs]) === Type.f16 ||
  scalarTypeOf(kScalarAndVectorTypes[t.params.rhs]) === Type.f16)
  {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const op = kOperators[t.params.op];
  const lhs = kScalarAndVectorTypes[t.params.lhs];
  const rhs = kScalarAndVectorTypes[t.params.rhs];
  const lhsElement = scalarTypeOf(lhs);
  const rhsElement = scalarTypeOf(rhs);
  const hasBool = lhsElement === Type.bool || rhsElement === Type.bool;
  const hasF16 = lhsElement === Type.f16 || rhsElement === Type.f16;
  const resType = resultType({ lhs, rhs, canConvertScalarToVector: true });
  const resTypeIsTypeable = resType && !isAbstractType(scalarTypeOf(resType));
  const code = t.params.compound_assignment ?
  `
${hasF16 ? 'enable f16;' : ''}
fn f() {
  var v = ${lhs.create(0).wgsl()};
  v ${op.op}= ${rhs.create(t.params.rhs_value).wgsl()};
}
` :
  `
${hasF16 ? 'enable f16;' : ''}
const lhs = ${lhs.create(1).wgsl()};
const rhs = ${rhs.create(t.params.rhs_value).wgsl()};
const foo ${resTypeIsTypeable ? `: ${resType}` : ''} = lhs ${op.op} rhs;
`;

  const scalarLHS = scalarTypeOf(concreteTypeOf(lhs));
  const integral = scalarLHS === Type.u32 || scalarLHS === Type.i32;
  let valid = !hasBool && resType !== null;
  if (valid && t.params.compound_assignment) {
    valid =
    valid &&
    isConvertible(resType, concreteTypeOf(lhs)) && (
    !integral || t.params.rhs_value === 1);
  } else {
    valid = valid && t.params.rhs_value === 1;
  }
  t.expectCompileResult(valid, code);
});

g.test('scalar_vector_out_of_range').
desc(
  `
    Checks that constant or override evaluation of div/rem operations on scalar/vectors that produce out of division by 0 or out of range values cause validation errors.
      - Checks for all concrete numeric scalar and vector types, including scalar * vector and vector * scalar.
      - Checks for all vector elements that could cause the out of range to happen.
      - Checks for valid small cases and 0, also the minimum i32.
  `
).
params((u) =>
u.
combine('op', keysOf(kOperators)).
combine('lhs', keysOf(kConcreteNumericScalarAndVectorTypes)).
expand('rhs', (p) => {
  if (kScalarAndVectorTypes[p.lhs] instanceof VectorType) {
    return [p.lhs, scalarTypeOf(kScalarAndVectorTypes[p.lhs]).toString()];
  }
  return [p.lhs];
}).
beginSubcases().
expand('swap', (p) => {
  if (p.lhs === p.rhs) {
    return [false];
  }
  return [false, true];
}).
combine('nonOneIndex', [0, 1, 2, 3]).
filter((p) => {
  const lType = kScalarAndVectorTypes[p.lhs];
  if (lType instanceof VectorType) {
    return lType.width > p.nonOneIndex;
  }
  return p.nonOneIndex === 0;
}).
expandWithParams((p) => {
  // When lhs is a non-const expression, division by zero is only an error for integral types.
  const partialDivByZeroIsError = [Type.i32, Type.u32].includes(
    scalarTypeOf(kScalarAndVectorTypes[p.rhs])
  );
  const cases = [
  { leftValue: 42, rightValue: 0, error: true, leftRuntime: false },
  { leftValue: 42, rightValue: 0, error: partialDivByZeroIsError, leftRuntime: true },
  { leftValue: 0, rightValue: 0, error: partialDivByZeroIsError, leftRuntime: true },
  { leftValue: 0, rightValue: 42, error: false, leftRuntime: false }];

  if (p.lhs === 'i32') {
    cases.push({
      leftValue: -kBit.i32.negative.min,
      rightValue: -1,
      error: true,
      leftRuntime: false
    });
    cases.push({
      leftValue: -kBit.i32.negative.min + 1,
      rightValue: -1,
      error: false,
      leftRuntime: false
    });
  }
  return cases;
}).
combine('stage', kConstantAndOverrideStages)
).
beforeAllSubcases((t) => {
  if (
  scalarTypeOf(kScalarAndVectorTypes[t.params.lhs]) === Type.f16 ||
  scalarTypeOf(kScalarAndVectorTypes[t.params.rhs]) === Type.f16)
  {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const { op, leftValue, rightValue, error, leftRuntime, nonOneIndex, swap } = t.params;
  let { lhs, rhs } = t.params;

  // Handle the swapping of LHS and RHS to test all cases of scalar * vector.
  if (swap) {
    [rhs, lhs] = [lhs, rhs];
  }

  // Creates either a scalar with the value, or a vector with the value only at a specific index.
  const create = (type, index, value) => {
    if (type instanceof ScalarType) {
      return type.create(value);
    } else {
      assert(type instanceof VectorType);
      const values = new Array(type.width);
      values.fill(1);
      values[index] = value;
      return type.create(values);
    }
  };

  // Check if there is overflow
  validateConstOrOverrideBinaryOpEval(
    t,
    kOperators[op].op,
    !error,
    leftRuntime ? 'runtime' : t.params.stage,
    create(kScalarAndVectorTypes[lhs], nonOneIndex, leftValue),
    t.params.stage,
    create(kScalarAndVectorTypes[rhs], nonOneIndex, rightValue)
  );
});







const kInvalidTypes = {
  array: {
    expr: 'arr',
    control: (e) => `${e}[0]`
  },

  ptr: {
    expr: '(&u)',
    control: (e) => `*${e}`
  },

  atomic: {
    expr: 'a',
    control: (e) => `atomicLoad(&${e})`
  },

  texture: {
    expr: 't',
    control: (e) => `i32(textureLoad(${e}, vec2(), 0).x)`
  },

  sampler: {
    expr: 's',
    control: (e) => `i32(textureSampleLevel(t, ${e}, vec2(), 0).x)`
  },

  struct: {
    expr: 'str',
    control: (e) => `${e}.u`
  }
};

g.test('invalid_type_with_itself').
desc(
  `
  Validates that expressions are never accepted for non-scalar, non-vector, and non-matrix types.
  `
).
params((u) =>
u.
combine('op', keysOf(kOperators)).
combine('type', keysOf(kInvalidTypes)).
combine('control', [true, false]).
beginSubcases()
).
fn((t) => {
  const op = kOperators[t.params.op];
  const type = kInvalidTypes[t.params.type];
  const expr = t.params.control ? type.control(type.expr) : type.expr;
  const code = `
@group(0) @binding(0) var t : texture_2d<f32>;
@group(0) @binding(1) var s : sampler;
@group(0) @binding(2) var<storage, read_write> a : atomic<i32>;

struct S { u : u32 }

var<private> u : u32;
var<private> m : mat2x2f;
var<private> arr : array<i32, 4>;
var<private> str : S;

@compute @workgroup_size(1)
fn main() {
  let foo = ${expr} ${op.op} ${expr};
}
`;

  t.expectCompileResult(t.params.control, code);
});