/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for vector types
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kCases = {
  // Valid vector types
  vec2_bool: { wgsl: 'alias T = vec2<bool>;', ok: true },
  vec3_bool: { wgsl: 'alias T = vec3<bool>;', ok: true },
  vec4_bool: { wgsl: 'alias T = vec4<bool>;', ok: true },
  vec2_i32: { wgsl: 'alias T = vec2<i32>;', ok: true },
  vec3_i32: { wgsl: 'alias T = vec3<i32>;', ok: true },
  vec4_i32: { wgsl: 'alias T = vec4<i32>;', ok: true },
  vec2_u32: { wgsl: 'alias T = vec2<u32>;', ok: true },
  vec3_u32: { wgsl: 'alias T = vec3<u32>;', ok: true },
  vec4_u32: { wgsl: 'alias T = vec4<u32>;', ok: true },
  vec2_f32: { wgsl: 'alias T = vec2<f32>;', ok: true },
  vec3_f32: { wgsl: 'alias T = vec3<f32>;', ok: true },
  vec4_f32: { wgsl: 'alias T = vec4<f32>;', ok: true },
  vec2_f16: { wgsl: 'enable f16;\nalias T = vec2<f16>;', ok: true },
  vec3_f16: { wgsl: 'enable f16;\nalias T = vec3<f16>;', ok: true },
  vec4_f16: { wgsl: 'enable f16;\nalias T = vec4<f16>;', ok: true },

  // Pre-declared type aliases
  vec2i: { wgsl: 'const c : vec2i = vec2<i32>();', ok: true },
  vec3i: { wgsl: 'const c : vec3i = vec3<i32>();', ok: true },
  vec4i: { wgsl: 'const c : vec4i = vec4<i32>();', ok: true },
  vec2u: { wgsl: 'const c : vec2u = vec2<u32>();', ok: true },
  vec3u: { wgsl: 'const c : vec3u = vec3<u32>();', ok: true },
  vec4u: { wgsl: 'const c : vec4u = vec4<u32>();', ok: true },
  vec2f: { wgsl: 'const c : vec2f = vec2<f32>();', ok: true },
  vec3f: { wgsl: 'const c : vec3f = vec3<f32>();', ok: true },
  vec4f: { wgsl: 'const c : vec4f = vec4<f32>();', ok: true },
  vec2h: { wgsl: 'enable f16;\nconst c : vec2h = vec2<f16>();', ok: true },
  vec3h: { wgsl: 'enable f16;\nconst c : vec3h = vec3<f16>();', ok: true },
  vec4h: { wgsl: 'enable f16;\nconst c : vec4h = vec4<f16>();', ok: true },

  // pass
  trailing_comma: { wgsl: 'alias T = vec3<u32,>;', ok: true },
  aliased_el_ty: { wgsl: 'alias EL = i32;\nalias T = vec3<EL>;', ok: true },

  // invalid
  vec: { wgsl: 'alias T = vec;', ok: false },
  vec_f32: { wgsl: 'alias T = vec<f32>;', ok: false },
  vec1_i32: { wgsl: 'alias T = vec1<i32>;', ok: false },
  vec5_u32: { wgsl: 'alias T = vec5<u32>;', ok: false },
  missing_el_ty: { wgsl: 'alias T = vec3<>;', ok: false },
  missing_t_left: { wgsl: 'alias T = vec3 u32>;', ok: false },
  missing_t_right: { wgsl: 'alias T = vec3<u32;', ok: false },
  vec_of_array: { wgsl: 'alias T = vec3<array<i32, 2>>;', ok: false },
  vec_of_runtime_array: { wgsl: 'alias T = vec3<array<i32>>;', ok: false },
  vec_of_struct: { wgsl: 'struct S { i : i32 }\nalias T = vec3<S>;', ok: false },
  vec_of_atomic: { wgsl: 'alias T = vec3<atomic<i32>>;', ok: false },
  vec_of_matrix: { wgsl: 'alias T = vec3<mat2x2f>;', ok: false },
  vec_of_vec: { wgsl: 'alias T = vec3<vec2f>;', ok: false },
  no_bool_shortform: { wgsl: 'const c : vec2b = vec2<bool>();', ok: false }
};

g.test('vector').
desc('Tests validation of vector types').
params(
  (u) => u.combine('case', keysOf(kCases)) //
).
beforeAllSubcases((t) => {
  const c = kCases[t.params.case];
  if (c.wgsl.indexOf('enable f16') >= 0) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const c = kCases[t.params.case];
  t.expectCompileResult(c.ok, c.wgsl);
});