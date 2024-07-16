/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for matrix types
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValidCases = {
  // Basic matrices
  mat2x2_f32: `alias T = mat2x2<f32>;`,
  mat2x3_f32: `alias T = mat2x3<f32>;`,
  mat2x4_f32: `alias T = mat2x4<f32>;`,
  mat3x2_f32: `alias T = mat3x2<f32>;`,
  mat3x3_f32: `alias T = mat3x3<f32>;`,
  mat3x4_f32: `alias T = mat3x4<f32>;`,
  mat4x2_f32: `alias T = mat4x2<f32>;`,
  mat4x3_f32: `alias T = mat4x3<f32>;`,
  mat4x4_f32: `alias T = mat4x4<f32>;`,
  mat2x2_f16: `enable f16;\nalias T = mat2x2<f16>;`,
  mat2x3_f16: `enable f16;\nalias T = mat2x3<f16>;`,
  mat2x4_f16: `enable f16;\nalias T = mat2x4<f16>;`,
  mat3x2_f16: `enable f16;\nalias T = mat3x2<f16>;`,
  mat3x3_f16: `enable f16;\nalias T = mat3x3<f16>;`,
  mat3x4_f16: `enable f16;\nalias T = mat3x4<f16>;`,
  mat4x2_f16: `enable f16;\nalias T = mat4x2<f16>;`,
  mat4x3_f16: `enable f16;\nalias T = mat4x3<f16>;`,
  mat4x4_f16: `enable f16;\nalias T = mat4x4<f16>;`,

  // Pre-declared aliases
  mat2x2f: `alias T = mat2x2f;`,
  mat2x3f: `alias T = mat2x3f;`,
  mat2x4f: `alias T = mat2x4f;`,
  mat3x2f: `alias T = mat3x2f;`,
  mat3x3f: `alias T = mat3x3f;`,
  mat3x4f: `alias T = mat3x4f;`,
  mat4x2f: `alias T = mat4x2f;`,
  mat4x3f: `alias T = mat4x3f;`,
  mat4x4f: `alias T = mat4x4f;`,
  mat2x2h: `enable f16;\nalias T = mat2x2h;`,
  mat2x3h: `enable f16;\nalias T = mat2x3h;`,
  mat2x4h: `enable f16;\nalias T = mat2x4h;`,
  mat3x2h: `enable f16;\nalias T = mat3x2h;`,
  mat3x3h: `enable f16;\nalias T = mat3x3h;`,
  mat3x4h: `enable f16;\nalias T = mat3x4h;`,
  mat4x2h: `enable f16;\nalias T = mat4x2h;`,
  mat4x3h: `enable f16;\nalias T = mat4x3h;`,
  mat4x4h: `enable f16;\nalias T = mat4x4h;`,

  trailing_comma: `alias T = mat2x2<f32,>;`,

  // Abstract matrices
  abstract_2x2: `const m = mat2x2(1,1,1,1);`,
  abstract_2x3: `const m = mat2x3(1,1,1,1,1,1);`,
  abstract_2x4: `const m = mat2x4(1,1,1,1,1,1,1,1);`,

  // Base roots shadowable
  shadow_mat2x2: `alias mat2x2 = array<vec2f, 2>;`,
  shadow_mat2x3: `alias mat2x3 = array<vec2f, 3>;`,
  shadow_mat2x4: `alias mat2x4 = array<vec2f, 4>;`,
  shadow_mat3x2: `alias mat3x2 = array<vec3f, 2>;`,
  shadow_mat3x3: `alias mat3x3 = array<vec3f, 3>;`,
  shadow_mat3x4: `alias mat3x4 = array<vec3f, 4>;`,
  shadow_mat4x2: `alias mat4x2 = array<vec4f, 2>;`,
  shadow_mat4x3: `alias mat4x3 = array<vec4f, 3>;`,
  shadow_mat4x4: `alias mat4x4 = array<vec4f, 4>;`,

  // Pre-declared aliases shadowable
  shadow_mat2x2f: `alias mat2x2f = mat2x2<f32>;`,
  shadow_mat2x3f: `alias mat2x3f = mat2x3<f32>;`,
  shadow_mat2x4f: `alias mat2x4f = mat2x4<f32>;`,
  shadow_mat3x2f: `alias mat3x2f = mat3x2<f32>;`,
  shadow_mat3x3f: `alias mat3x3f = mat3x3<f32>;`,
  shadow_mat3x4f: `alias mat3x4f = mat3x4<f32>;`,
  shadow_mat4x2f: `alias mat4x2f = mat4x2<f32>;`,
  shadow_mat4x3f: `alias mat4x3f = mat4x3<f32>;`,
  shadow_mat4x4f: `alias mat4x4f = mat4x4<f32>;`,
  shadow_mat2x2h: `enable f16;\nalias mat2x2h = mat2x2<f16>;`,
  shadow_mat2x3h: `enable f16;\nalias mat2x3h = mat2x3<f16>;`,
  shadow_mat2x4h: `enable f16;\nalias mat2x4h = mat2x4<f16>;`,
  shadow_mat3x2h: `enable f16;\nalias mat3x2h = mat3x2<f16>;`,
  shadow_mat3x3h: `enable f16;\nalias mat3x3h = mat3x3<f16>;`,
  shadow_mat3x4h: `enable f16;\nalias mat3x4h = mat3x4<f16>;`,
  shadow_mat4x2h: `enable f16;\nalias mat4x2h = mat4x2<f16>;`,
  shadow_mat4x3h: `enable f16;\nalias mat4x3h = mat4x3<f16>;`,
  shadow_mat4x4h: `enable f16;\nalias mat4x4h = mat4x4<f16>;`,

  // Alias
  alias: `alias E = f32; alias T = mat2x2<E>;`
};

g.test('valid').
desc('Valid matrix type tests').
params((u) => u.combine('case', keysOf(kValidCases))).
beforeAllSubcases((t) => {
  const code = kValidCases[t.params.case];
  if (code.indexOf('f16') >= 0) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const code = kValidCases[t.params.case];
  t.expectCompileResult(true, code);
});

const kInvalidCases = {
  // Invalid component types
  mat2x2_i32: `alias T = mat2x2<i32>;`,
  mat3x3_u32: `alias T = mat3x3<u32>;`,
  mat4x4_bool: `alias T = mat4x4<bool>;`,
  mat2x2_vec4f: `alias T = mat2x2<vec2f>;`,
  mat2x2_array: `alias T = mat2x2<array<f32, 2>>;`,
  mat2x2_struct: `struct S { x : f32 }\nalias T = mat2x2<S>;`,

  // Invalid dimensions
  mat1x1: `alias T = mat1x1<f32>;`,
  mat2x1: `alias T = mat2x1<f32>;`,
  mat2x5: `alias T = mat2x5<f32>;`,
  mat5x5: `alias T = mat5x5<f32>;`,
  mat2x: `alias T = mat2x<f32>;`,
  matx2: `alias T = matx2<f32>;`,
  mat2: `alias T = mat2<f32>;`,
  mat: `alias T = mat;`,
  mat_f32: `alias T = mat<f32>;`,

  // Half-precision aliases require enable
  no_enable_mat2x2h: `alias T = mat2x2h;`,
  no_enable_mat2x3h: `alias T = mat2x3h;`,
  no_enable_mat2x4h: `alias T = mat2x4h;`,
  no_enable_mat3x2h: `alias T = mat3x2h;`,
  no_enable_mat3x3h: `alias T = mat3x3h;`,
  no_enable_mat3x4h: `alias T = mat3x4h;`,
  no_enable_mat4x2h: `alias T = mat4x2h;`,
  no_enable_mat4x3h: `alias T = mat4x3h;`,
  no_enable_mat4x4h: `alias T = mat4x4h;`,

  missing_template: `alias T = mat2x2;`,
  missing_left_template: `alias T = mat2x2f32>;`,
  missing_right_template: `alias T = mat2x2<f32;`,
  missing_comp: `alias T = mat2x2<>;`,
  mat2x2i: `alias T = mat2x2i;`,
  mat2x2u: `alias T = mat2x2u;`,
  mat2x2b: `alias T = mat2x2b;`
};

g.test('invalid').
desc('Invalid matrix type tests').
params((u) => u.combine('case', keysOf(kInvalidCases))).
beforeAllSubcases((t) => {
  const code = kInvalidCases[t.params.case];
  if (code.indexOf('f16') >= 0) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const code = kInvalidCases[t.params.case];
  t.expectCompileResult(false, code);
});