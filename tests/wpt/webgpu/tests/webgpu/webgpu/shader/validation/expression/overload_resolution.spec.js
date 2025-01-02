/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for implicit conversions and overload resolution`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../common/util/data_tables.js';
import {
  kAllNumericScalarsAndVectors,
  isConvertible,
  VectorType } from
'../../../util/conversion.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);







const kImplicitConversionCases = {
  absint_to_bool: {
    expr: `any(1)`,
    valid: false
  },
  absint_to_u32: {
    expr: `1 == 1u`,
    valid: true
  },
  absint_to_i32: {
    expr: `1 == 1i`,
    valid: true
  },
  absint_to_f32: {
    expr: `1 == 1f`,
    valid: true
  },
  absint_to_f16: {
    expr: `1 == 1h`,
    valid: true,
    f16: true
  },
  absfloat_to_bool: {
    expr: `any(1.0)`,
    valid: false
  },
  absfloat_to_u32: {
    expr: `1.0 == 1u`,
    valid: false
  },
  absfloat_to_i32: {
    expr: `1.0 == 1i`,
    valid: false
  },
  absfloat_to_f32: {
    expr: `1.0 == 1f`,
    valid: true
  },
  absfloat_to_f16: {
    expr: `1.0 == 1h`,
    valid: true,
    f16: true
  },
  vector_absint_to_bool: {
    expr: `any(vec2(1))`,
    valid: false
  },
  vector_absint_to_u32: {
    expr: `all(vec2(1) == vec2u(1u))`,
    valid: true
  },
  vector_absint_to_i32: {
    expr: `all(vec3(1) == vec3i(1i))`,
    valid: true
  },
  vector_absint_to_f32: {
    expr: `all(vec4(1) == vec4f(1f))`,
    valid: true
  },
  vector_absint_to_f16: {
    expr: `all(vec2(1) == vec2h(1h))`,
    valid: true,
    f16: true
  },
  vector_absfloat_to_bool: {
    expr: `any(vec2(1.0))`,
    valid: false
  },
  vector_absfloat_to_u32: {
    expr: `all(vec2(1.0) == vec2u(1u))`,
    valid: false
  },
  vector_absfloat_to_i32: {
    expr: `all(vec3(1.0) == vec2i(1i))`,
    valid: false
  },
  vector_absfloat_to_f32: {
    expr: `all(vec4(1.0) == vec4f(1f))`,
    valid: true
  },
  vector_absfloat_to_f16: {
    expr: `all(vec2(1.0) == vec2h(1h))`,
    valid: true,
    f16: true
  },
  vector_swizzle_integer: {
    expr: `vec2(1).x == 1i`,
    valid: true
  },
  vector_swizzle_float: {
    expr: `vec2(1).y == 1f`,
    valid: true
  },
  vector_default_ctor_integer: {
    expr: `all(vec3().xy == vec2i())`,
    valid: true
  },
  vector_default_ctor_abstract: {
    expr: `all(vec3().xy == vec2())`,
    valid: true
  },
  vector_swizzle_abstract: {
    expr: `vec4(1f).x == 1`,
    valid: true
  },
  vector_abstract_to_integer: {
    expr: `all(vec4(1) == vec4i(1))`,
    valid: true
  },
  vector_wrong_result_i32: {
    expr: `vec2(1,2f).x == 1i`,
    valid: false
  },
  vector_wrong_result_f32: {
    expr: `vec2(1,2i).y == 2f`,
    valid: false
  },
  vector_wrong_result_splat: {
    expr: `vec2(1.0).x == 1i`,
    valid: false
  },
  array_absint_to_bool: {
    expr: `any(array(1)[0])`,
    valid: false
  },
  array_absint_to_u32: {
    expr: `array(1)[0] == array<u32,1>(1u)[0]`,
    valid: true
  },
  array_absint_to_i32: {
    expr: `array(1)[0] == array<i32,1>(1i)[0]`,
    valid: true
  },
  array_absint_to_f32: {
    expr: `array(1)[0] == array<f32,1>(1f)[0]`,
    valid: true
  },
  array_absint_to_f16: {
    expr: `array(1)[0] == array<f16,1>(1h)[0]`,
    valid: true,
    f16: true
  },
  array_absfloat_to_bool: {
    expr: `any(array(1.0)[0])`,
    valid: false
  },
  array_absfloat_to_u32: {
    expr: `array(1.0)[0] == array<u32,1>(1u)[0]`,
    valid: false
  },
  array_absfloat_to_i32: {
    expr: `array(1.0)[0] == array<i32,1>(1i)[0]`,
    valid: false
  },
  array_absfloat_to_f32: {
    expr: `array(1.0)[0] == array<f32,1>(1f)[0]`,
    valid: true
  },
  array_absfloat_to_f16: {
    expr: `array(1.0)[0] == array<f16,1>(1h)[0]`,
    valid: true,
    f16: true
  },
  mat2x2_index_absint: {
    expr: `all(mat2x2(1,2,3,4)[0] == vec2(1,2))`,
    valid: true
  },
  mat2x2_index_absfloat: {
    expr: `all(mat2x2(1,2,3,4)[1] == vec2(3.0,4.0))`,
    valid: true
  },
  mat2x2_index_float: {
    expr: `all(mat2x2(0,0,0,0)[1] == vec2f())`,
    valid: true
  },
  mat2x2_wrong_result: {
    expr: `all(mat2x2(0f,0,0,0)[0] == vec2h())`,
    valid: false,
    f16: true
  }
};

g.test('implicit_conversions').
desc('Test implicit conversions').
params((u) => u.combine('case', keysOf(kImplicitConversionCases))).
beforeAllSubcases((t) => {
  if (kImplicitConversionCases[t.params.case].f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kImplicitConversionCases[t.params.case];
  const code = `${testcase.f16 ? 'enable f16;' : ''}
    const_assert ${testcase.expr};`;
  t.expectCompileResult(testcase.valid, code);
});

const kTypes = objectsToRecord(kAllNumericScalarsAndVectors);
const kTypeKeys = keysOf(kTypes);

g.test('overload_resolution').
desc('Test overload resolution').
params((u) =>
u.
combine('arg1', kTypeKeys).
combine('arg2', kTypeKeys).
beginSubcases().
combine('op', ['min', 'max']).
filter((t) => {
  if (t.arg1 === t.arg2) {
    return false;
  }
  const t1 = kTypes[t.arg1];
  const t2 = kTypes[t.arg2];
  const t1IsVector = t1 instanceof VectorType;
  const t2IsVector = t2 instanceof VectorType;
  if (t1IsVector !== t2IsVector) {
    return false;
  }
  if (t1IsVector && t2IsVector && t1.size !== t2.size) {
    return false;
  }
  return true;
})
).
beforeAllSubcases((t) => {
  const t1 = kTypes[t.params.arg1];
  const t2 = kTypes[t.params.arg2];
  if (t1.requiresF16() || t2.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const t1 = kTypes[t.params.arg1];
  const t2 = kTypes[t.params.arg2];
  const resTy = isConvertible(t1, t2) ? t2 : t1;
  const enable = `${t1.requiresF16() || t2.requiresF16() ? 'enable f16;' : ''}`;
  const min = 50;
  const max = 100;
  const res = t.params.op === 'min' ? min : max;
  const v1 = t1.create(min).wgsl();
  const v2 = t2.create(max).wgsl();
  const resV = resTy.create(res).wgsl();
  const expr = `${t.params.op}(${v1}, ${v2}) == ${resV}`;
  const assertExpr = t1 instanceof VectorType ? `all(${expr})` : expr;
  const code = `${enable}
    const_assert ${assertExpr};`;
  t.expectCompileResult(isConvertible(t1, t2) || isConvertible(t2, t1), code);
});