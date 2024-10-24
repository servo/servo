/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for short-circuiting && and || expressions.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../common/util/data_tables.js';
import {
  kAllScalarsAndVectors,
  ScalarType,
  scalarTypeOf,
  Type } from
'../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// A list of scalar and vector types.
const kScalarAndVectorTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('scalar_vector').
desc(
  `
  Validates that scalar and vector short-circuiting operators are only accepted for scalar booleans.
  `
).
params((u) =>
u.
combine('op', ['&&', '||']).
combine('lhs', keysOf(kScalarAndVectorTypes)).
combine(
  'rhs',
  // Skip vec3 and vec4 on the RHS to keep the number of subcases down.
  keysOf(kScalarAndVectorTypes).filter(
    (value) => !(value.startsWith('vec3') || value.startsWith('vec4'))
  )
).
beginSubcases()
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
  const lhs = kScalarAndVectorTypes[t.params.lhs];
  const rhs = kScalarAndVectorTypes[t.params.rhs];
  const lhsElement = scalarTypeOf(lhs);
  const rhsElement = scalarTypeOf(rhs);
  const hasF16 = lhsElement === Type.f16 || rhsElement === Type.f16;
  const code = `
${hasF16 ? 'enable f16;' : ''}
const lhs = ${lhs.create(0).wgsl()};
const rhs = ${rhs.create(0).wgsl()};
const foo = lhs ${t.params.op} rhs;
`;

  // Determine if the types are compatible.
  let valid = false;
  if (lhs instanceof ScalarType && rhs instanceof ScalarType) {
    valid = lhsElement === Type.bool && rhsElement === Type.bool;
  }

  t.expectCompileResult(valid, code);
});







const kInvalidTypes = {
  mat2x2f: {
    expr: 'm',
    control: (e) => `bool(${e}[0][0])`
  },

  array: {
    expr: 'arr',
    control: (e) => `${e}[0]`
  },

  ptr: {
    expr: '(&b)',
    control: (e) => `*${e}`
  },

  atomic: {
    expr: 'a',
    control: (e) => `bool(atomicLoad(&${e}))`
  },

  texture: {
    expr: 't',
    control: (e) => `bool(textureLoad(${e}, vec2(), 0).x)`
  },

  sampler: {
    expr: 's',
    control: (e) => `bool(textureSampleLevel(t, ${e}, vec2(), 0).x)`
  },

  struct: {
    expr: 'str',
    control: (e) => `${e}.b`
  }
};

g.test('invalid_types').
desc(
  `
  Validates that short-circuiting expressions are never accepted for non-scalar and non-vector types.
  `
).
params((u) =>
u.
combine('op', ['&&', '||']).
combine('type', keysOf(kInvalidTypes)).
combine('control', [true, false]).
beginSubcases()
).
fn((t) => {
  const type = kInvalidTypes[t.params.type];
  const expr = t.params.control ? type.control(type.expr) : type.expr;
  const code = `
@group(0) @binding(0) var t : texture_2d<f32>;
@group(0) @binding(1) var s : sampler;
@group(0) @binding(2) var<storage, read_write> a : atomic<i32>;

struct S { b : bool }

var<private> b : bool;
var<private> m : mat2x2f;
var<private> arr : array<bool, 4>;
var<private> str : S;

@compute @workgroup_size(1)
fn main() {
  let foo = ${expr} ${t.params.op} ${expr};
}
`;

  t.expectCompileResult(t.params.control, code);
});

// A map from operator to the value of the LHS that will cause short-circuiting.
const kLhsForShortCircuit = {
  '&&': false,
  '||': true
};

// A list of expressions that are invalid unless guarded by a short-circuiting expression.
const kInvalidRhsExpressions = {
  overflow: 'i32(1<<thirty_one) < 0',
  div_zero_i32: '(1 / zero_i32) == 0',
  div_zero_f32: '(one_f32 / 0) == 0',
  builtin: 'sqrt(-one_f32) == 0'
};

g.test('invalid_rhs_const').
desc(
  `
  Validates that a short-circuiting expression with a const-expression LHS guards the evaluation of its RHS expression.
  `
).
params((u) =>
u.
combine('op', ['&&', '||']).
combine('rhs', keysOf(kInvalidRhsExpressions)).
combine('short_circuit', [true, false]).
beginSubcases()
).
fn((t) => {
  let lhs = kLhsForShortCircuit[t.params.op];
  if (!t.params.short_circuit) {
    lhs = !lhs;
  }
  const code = `
const thirty_one = 31u;
const zero_i32 = 0i;
const one_f32 = 1.0f;

@compute @workgroup_size(1)
fn main() {
  let foo = ${lhs} ${t.params.op} ${kInvalidRhsExpressions[t.params.rhs]};
}
`;

  t.expectCompileResult(t.params.short_circuit, code);
});

g.test('invalid_rhs_override').
desc(
  `
  Validates that a short-circuiting expression with an override-expression LHS guards the evaluation of its RHS expression.
  `
).
params((u) =>
u.
combine('op', ['&&', '||']).
combine('rhs', keysOf(kInvalidRhsExpressions)).
combine('short_circuit', [true, false]).
beginSubcases()
).
fn((t) => {
  let lhs = kLhsForShortCircuit[t.params.op];
  if (!t.params.short_circuit) {
    lhs = !lhs;
  }
  const code = `
override cond : bool;
override zero_i32 = 0i;
override one_f32 = 1.0f;
override thirty_one = 31u;
override foo = cond ${t.params.op} ${kInvalidRhsExpressions[t.params.rhs]};
`;

  const constants = {};
  constants['cond'] = lhs ? 1 : 0;
  t.expectPipelineResult({
    expectedResult: t.params.short_circuit,
    code,
    constants,
    reference: ['foo']
  });
});

// A list of expressions that are invalid unless guarded by a short-circuiting expression.
// The control case will use `value = 10`, the failure case will use `value = 1`.
const kInvalidArrayCounts = {
  negative: 'value - 2',
  sqrt_neg1: 'u32(sqrt(value - 2))',
  nested: '10 + array<i32, value - 2>()[0]'
};

g.test('invalid_array_count_on_rhs').
desc(
  `
  Validates that an invalid array count expression is not guarded by a short-circuiting expression.
  `
).
params((u) =>
u.
combine('op', ['&&', '||']).
combine('rhs', keysOf(kInvalidArrayCounts)).
combine('control', [true, false]).
beginSubcases()
).
fn((t) => {
  const lhs = t.params.op === '&&' ? 'false' : 'true';
  const code = `
const value = ${t.params.control ? '10' : '1'};

@compute @workgroup_size(1)
fn main() {
  let foo = ${lhs} ${t.params.op} array<bool, ${kInvalidArrayCounts[t.params.rhs]}>()[0];
}
`;

  t.expectCompileResult(t.params.control, code);
});