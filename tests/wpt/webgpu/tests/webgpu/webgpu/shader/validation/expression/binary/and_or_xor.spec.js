/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for logical and bitwise and/or/xor expressions.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../common/util/data_tables.js';
import {
  concreteTypeOf,
  isAbstractType,
  isConvertible,
  isIntegerType,
  kAllScalarsAndVectors,
  scalarTypeOf,
  Type } from
'../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

import { resultType } from './result_type.js';

export const g = makeTestGroup(ShaderValidationTest);

// A list of operators and a flag for whether they support boolean values or not.
const kOperators = {
  and: { op: '&', supportsBool: true },
  or: { op: '|', supportsBool: true },
  xor: { op: '^', supportsBool: false }
};

// A list of scalar and vector types.
const kScalarAndVectorTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('scalar_vector').
desc(
  `
  Validates that scalar and vector expressions are only accepted for bool or compatible integer types.
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
combine('op', keysOf(kOperators))
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
  const resType =
  isIntegerType(lhsElement) && isIntegerType(rhsElement) || hasBool && op.supportsBool ?
  resultType({ lhs, rhs, canConvertScalarToVector: false }) :
  null;
  const resTypeIsTypeable = resType && !isAbstractType(scalarTypeOf(resType));
  const code = t.params.compound_assignment ?
  `
${hasF16 ? 'enable f16;' : ''}
fn f() {
  var foo = ${lhs.create(0).wgsl()};
  foo ${op.op}= ${rhs.create(0).wgsl()};
}
` :
  `
${hasF16 ? 'enable f16;' : ''}
const lhs = ${lhs.create(0).wgsl()};
const rhs = ${rhs.create(0).wgsl()};
const foo ${resTypeIsTypeable ? `: ${resType}` : ''} = lhs ${op.op} rhs;
`;

  let valid = resType !== null;
  if (valid && t.params.compound_assignment) {
    valid = valid && isConvertible(resType, concreteTypeOf(lhs));
  }

  t.expectCompileResult(valid, code);
});







const kInvalidTypes = {
  mat2x2f: {
    expr: 'm',
    control: (e) => `i32(${e}[0][0])`
  },

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

g.test('invalid_types').
desc(
  `
  Validates that expressions are never accepted for non-scalar and non-vector types.
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