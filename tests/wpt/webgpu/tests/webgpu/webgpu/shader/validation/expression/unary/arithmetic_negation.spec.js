/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for arithmetic negation expressions.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../common/util/data_tables.js';
import { kAllScalarsAndVectors, scalarTypeOf, Type } from '../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// A list of scalar and vector types.
const kScalarAndVectorTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('scalar_vector').
desc(
  `
  Validates that scalar and vector numeric negation expressions are accepted for numerical types that are signed.
  `
).
params((u) => u.combine('type', keysOf(kScalarAndVectorTypes)).beginSubcases()).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kScalarAndVectorTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kScalarAndVectorTypes[t.params.type];
  const elementTy = scalarTypeOf(type);
  const hasF16 = elementTy === Type.f16;
  const code = `
${hasF16 ? 'enable f16;' : ''}
const rhs = ${type.create(0).wgsl()};
const foo = -rhs;
`;

  t.expectCompileResult(elementTy.signed, code);
});







const kInvalidTypes = {
  mat2x2f: {
    expr: 'm',
    control: (e) => `${e}[0][0]`
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
    control: (e) => `atomicLoad(&${e})`
  },

  texture: {
    expr: 't',
    control: (e) => `textureLoad(${e}, vec2(), 0).x`
  },

  sampler: {
    expr: 's',
    control: (e) => `textureSampleLevel(t, ${e}, vec2(), 0).x`
  },

  struct: {
    expr: 'str',
    control: (e) => `${e}.b`
  }
};

g.test('invalid_types').
desc(
  `
  Validates that arithmetic negation expressions are never accepted for non-scalar and non-vector types.
  `
).
params((u) =>
u.combine('type', keysOf(kInvalidTypes)).combine('control', [true, false]).beginSubcases()
).
fn((t) => {
  const type = kInvalidTypes[t.params.type];
  const expr = t.params.control ? type.control(type.expr) : type.expr;
  const code = `
@group(0) @binding(0) var t : texture_2d<f32>;
@group(0) @binding(1) var s : sampler;
@group(0) @binding(2) var<storage, read_write> a : atomic<i32>;

struct S { b : i32 }

var<private> b : i32;
var<private> m : mat2x2f;
var<private> arr : array<i32, 4>;
var<private> str : S;

@compute @workgroup_size(1)
fn main() {
  let foo = -${expr};
}
`;

  t.expectCompileResult(t.params.control, code);
});