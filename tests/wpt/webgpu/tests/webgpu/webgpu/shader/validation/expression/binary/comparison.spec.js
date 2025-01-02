/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for comparison expressions.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../common/util/data_tables.js';
import {
  isFloatType,
  kAllScalarsAndVectors,
  ScalarType,
  scalarTypeOf,
  Type,
  VectorType } from
'../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// A list of scalar and vector types.
const kScalarAndVectorTypes = objectsToRecord(kAllScalarsAndVectors);

// A list of comparison operators and a flag for whether they support boolean values or not.
const kComparisonOperators = {
  eq: { op: '==', supportsBool: true },
  ne: { op: '!=', supportsBool: true },
  gt: { op: '>', supportsBool: false },
  ge: { op: '>=', supportsBool: false },
  lt: { op: '<', supportsBool: false },
  le: { op: '<=', supportsBool: false }
};

g.test('scalar_vector').
desc(
  `
  Validates that scalar and vector comparison expressions are only accepted for compatible types.
  `
).
params((u) =>
u.
combine('lhs', keysOf(kScalarAndVectorTypes)).
combine(
  'rhs',
  // Skip vec3 and vec4 on the RHS to keep the number of subcases down.
  keysOf(kScalarAndVectorTypes).filter(
    (value) => !(value.startsWith('vec3') || value.startsWith('vec4'))
  )
).
beginSubcases().
combine('op', keysOf(kComparisonOperators))
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
const foo = lhs ${kComparisonOperators[t.params.op].op} rhs;
`;

  let valid = false;

  // Determine if the element types are comparable.
  let elementIsCompatible = false;
  if (lhsElement === Type.abstractInt) {
    // Abstract integers are comparable to any other numeric type.
    elementIsCompatible = rhsElement !== Type.bool;
  } else if (rhsElement === Type.abstractInt) {
    // Abstract integers are comparable to any other numeric type.
    elementIsCompatible = lhsElement !== Type.bool;
  } else if (lhsElement === Type.abstractFloat) {
    // Abstract floats are comparable to any other float type.
    elementIsCompatible = isFloatType(rhsElement);
  } else if (rhsElement === Type.abstractFloat) {
    // Abstract floats are comparable to any other float type.
    elementIsCompatible = isFloatType(lhsElement);
  } else {
    // Non-abstract types are only comparable to values with the exact same type.
    elementIsCompatible = lhsElement === rhsElement;
  }

  // Determine if the full type is comparable.
  if (lhs instanceof ScalarType && rhs instanceof ScalarType) {
    valid = elementIsCompatible;
  } else if (lhs instanceof VectorType && rhs instanceof VectorType) {
    // Vectors are only comparable if the vector widths match.
    valid = lhs.width === rhs.width && elementIsCompatible;
  }

  if (lhsElement === Type.bool) {
    valid &&= kComparisonOperators[t.params.op].supportsBool;
  }

  t.expectCompileResult(valid, code);
});







const kInvalidTypes = {
  mat2x2f: {
    expr: 'm',
    control: (e) => `${e}[0]`
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
    control: (e) => `textureLoad(${e}, vec2(), 0)`
  },

  sampler: {
    expr: 's',
    control: (e) => `textureSampleLevel(t, ${e}, vec2(), 0)`
  },

  struct: {
    expr: 'str',
    control: (e) => `${e}.u`
  }
};

g.test('invalid_types').
desc(
  `
  Validates that comparison expressions are never accepted for non-scalar and non-vector types.
  `
).
params((u) =>
u.
combine('op', keysOf(kComparisonOperators)).
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

struct S { u : u32 }

var<private> u : u32;
var<private> m : mat2x2f;
var<private> arr : array<i32, 4>;
var<private> str : S;

@compute @workgroup_size(1)
fn main() {
  let foo = ${expr} ${kComparisonOperators[t.params.op].op} ${expr};
}
`;

  t.expectCompileResult(t.params.control, code);
});