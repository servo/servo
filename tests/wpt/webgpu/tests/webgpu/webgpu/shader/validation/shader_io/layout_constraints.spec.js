/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation of address space layout constraints`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);








const kLayoutCases = {
  // Scalars
  u32: {
    type: 'u32',
    validity: true
  },
  i32: {
    type: 'i32',
    validity: true
  },
  f32: {
    type: 'f32',
    validity: true
  },
  f16: {
    type: 'f16',
    validity: true,
    f16: true
  },
  bool: {
    type: 'bool',
    validity: 'non-interface'
  },

  // Vectors
  vec2u: {
    type: 'vec2u',
    validity: true
  },
  vec3u: {
    type: 'vec3u',
    validity: true
  },
  vec4u: {
    type: 'vec4u',
    validity: true
  },
  vec2i: {
    type: 'vec2i',
    validity: true
  },
  vec3i: {
    type: 'vec3i',
    validity: true
  },
  vec4i: {
    type: 'vec4i',
    validity: true
  },
  vec2f: {
    type: 'vec2f',
    validity: true
  },
  vec3f: {
    type: 'vec3f',
    validity: true
  },
  vec4f: {
    type: 'vec4f',
    validity: true
  },
  vec2h: {
    type: 'vec2h',
    validity: true,
    f16: true
  },
  vec3h: {
    type: 'vec3h',
    validity: true,
    f16: true
  },
  vec4h: {
    type: 'vec4h',
    validity: true,
    f16: true
  },
  vec2b: {
    type: 'vec2<bool>',
    validity: 'non-interface'
  },
  vec3b: {
    type: 'vec3<bool>',
    validity: 'non-interface'
  },
  vec4b: {
    type: 'vec4<bool>',
    validity: 'non-interface'
  },

  // Matrices
  mat2x2f: {
    type: 'mat2x2f',
    validity: true
  },
  mat2x3f: {
    type: 'mat2x3f',
    validity: true
  },
  mat2x4f: {
    type: 'mat2x4f',
    validity: true
  },
  mat3x2f: {
    type: 'mat3x2f',
    validity: true
  },
  mat3x3f: {
    type: 'mat3x3f',
    validity: true
  },
  mat3x4f: {
    type: 'mat3x4f',
    validity: true
  },
  mat4x2f: {
    type: 'mat4x2f',
    validity: true
  },
  mat4x3f: {
    type: 'mat4x3f',
    validity: true
  },
  mat4x4f: {
    type: 'mat4x4f',
    validity: true
  },
  mat2x2h: {
    type: 'mat2x2h',
    validity: true,
    f16: true
  },
  mat2x3h: {
    type: 'mat2x3h',
    validity: true,
    f16: true
  },
  mat2x4h: {
    type: 'mat2x4h',
    validity: true,
    f16: true
  },
  mat3x2h: {
    type: 'mat3x2h',
    validity: true,
    f16: true
  },
  mat3x3h: {
    type: 'mat3x3h',
    validity: true,
    f16: true
  },
  mat3x4h: {
    type: 'mat3x4h',
    validity: true,
    f16: true
  },
  mat4x2h: {
    type: 'mat4x2h',
    validity: true,
    f16: true
  },
  mat4x3h: {
    type: 'mat4x3h',
    validity: true,
    f16: true
  },
  mat4x4h: {
    type: 'mat4x4h',
    validity: true,
    f16: true
  },

  // Atomics
  atomic_u32: {
    type: 'atomic<u32>',
    validity: 'atomic'
  },
  atomic_i32: {
    type: 'atomic<i32>',
    validity: 'atomic'
  },

  // Sized arrays
  array_u32: {
    type: 'array<u32, 16>',
    validity: 'non-uniform'
  },
  array_i32: {
    type: 'array<i32, 16>',
    validity: 'non-uniform'
  },
  array_f32: {
    type: 'array<f32, 16>',
    validity: 'non-uniform'
  },
  array_f16: {
    type: 'array<f16, 16>',
    validity: 'non-uniform',
    f16: true
  },
  array_bool: {
    type: 'array<bool, 16>',
    validity: 'non-interface'
  },
  array_vec2f: {
    type: 'array<vec2f, 16>',
    validity: 'non-uniform'
  },
  array_vec3f: {
    type: 'array<vec3f, 16>',
    validity: true
  },
  array_vec4f: {
    type: 'array<vec4f, 16>',
    validity: true
  },
  array_vec2h: {
    type: 'array<vec2h, 16>',
    validity: 'non-uniform',
    f16: true
  },
  array_vec3h: {
    type: 'array<vec3h, 16>',
    validity: 'non-uniform',
    f16: true
  },
  array_vec4h: {
    type: 'array<vec4h, 16>',
    validity: 'non-uniform',
    f16: true
  },
  array_vec2b: {
    type: 'array<vec2<bool>, 16>',
    validity: 'non-interface'
  },
  array_vec3b: {
    type: 'array<vec3<bool>, 16>',
    validity: 'non-interface'
  },
  array_vec4b: {
    type: 'array<vec4<bool>, 16>',
    validity: 'non-interface'
  },
  array_mat2x2f: {
    type: 'array<mat2x2f, 16>',
    validity: true
  },
  array_mat2x4f: {
    type: 'array<mat2x4f, 16>',
    validity: true
  },
  array_mat4x2f: {
    type: 'array<mat4x2f, 16>',
    validity: true
  },
  array_mat4x4f: {
    type: 'array<mat4x4f, 16>',
    validity: true
  },
  array_mat2x2h: {
    type: 'array<mat2x2h, 16>',
    validity: 'non-uniform',
    f16: true
  },
  array_mat2x4h: {
    type: 'array<mat2x4h, 16>',
    validity: true,
    f16: true
  },
  array_mat3x2h: {
    type: 'array<mat3x2h, 16>',
    validity: 'non-uniform',
    f16: true
  },
  array_mat4x2h: {
    type: 'array<mat4x2h, 16>',
    validity: true,
    f16: true
  },
  array_mat4x4h: {
    type: 'array<mat4x4h, 16>',
    validity: true,
    f16: true
  },
  array_atomic: {
    type: 'array<atomic<u32>, 16>',
    validity: 'atomic'
  },

  // Runtime arrays
  runtime_array_u32: {
    type: 'array<u32>',
    validity: 'storage'
  },
  runtime_array_i32: {
    type: 'array<i32>',
    validity: 'storage'
  },
  runtime_array_f32: {
    type: 'array<f32>',
    validity: 'storage'
  },
  runtime_array_f16: {
    type: 'array<f16>',
    validity: 'storage',
    f16: true
  },
  runtime_array_bool: {
    type: 'array<bool>',
    validity: false
  },
  runtime_array_vec2f: {
    type: 'array<vec2f>',
    validity: 'storage'
  },
  runtime_array_vec3f: {
    type: 'array<vec3f>',
    validity: 'storage'
  },
  runtime_array_vec4f: {
    type: 'array<vec4f>',
    validity: 'storage'
  },
  runtime_array_vec2h: {
    type: 'array<vec2h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_vec3h: {
    type: 'array<vec3h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_vec4h: {
    type: 'array<vec4h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_vec2b: {
    type: 'array<vec2<bool>>',
    validity: false
  },
  runtime_array_vec3b: {
    type: 'array<vec3<bool>>',
    validity: false
  },
  runtime_array_vec4b: {
    type: 'array<vec4<bool>>',
    validity: false
  },
  runtime_array_mat2x2f: {
    type: 'array<mat2x2f>',
    validity: 'storage'
  },
  runtime_array_mat2x4f: {
    type: 'array<mat2x4f>',
    validity: 'storage'
  },
  runtime_array_mat4x2f: {
    type: 'array<mat4x2f>',
    validity: 'storage'
  },
  runtime_array_mat4x4f: {
    type: 'array<mat4x4f>',
    validity: 'storage'
  },
  runtime_array_mat2x2h: {
    type: 'array<mat2x2h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_mat2x4h: {
    type: 'array<mat2x4h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_mat3x2h: {
    type: 'array<mat3x2h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_mat4x2h: {
    type: 'array<mat4x2h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_mat4x4h: {
    type: 'array<mat4x4h>',
    validity: 'storage',
    f16: true
  },
  runtime_array_atomic: {
    type: 'array<atomic<u32>>',
    validity: 'storage'
  },

  // Structs (and arrays of structs)
  array_struct_u32: {
    type: 'array<S, 16>',
    decls: 'struct S { x : u32 }',
    validity: 'non-uniform'
  },
  array_struct_u32_size16: {
    type: 'array<S, 16>',
    decls: 'struct S { @size(16) x : u32 }',
    validity: true
  },
  array_struct_vec2f: {
    type: 'array<S, 16>',
    decls: 'struct S { x : vec2f }',
    validity: 'non-uniform'
  },
  array_struct_vec2h: {
    type: 'array<S, 16>',
    decls: 'struct S { x : vec2h }',
    validity: 'non-uniform',
    f16: true
  },
  array_struct_vec2h_align16: {
    type: 'array<S, 16>',
    decls: 'struct S { @align(16) x : vec2h }',
    validity: true,
    f16: true
  },
  size_too_small: {
    type: 'S',
    decls: 'struct S { @size(2) x : u32 }',
    validity: false
  },
  struct_padding: {
    type: 'S',
    decls: `struct T { x : u32 }
    struct S { t : T, x : u32 }`,
    validity: 'non-uniform'
  },
  struct_array_u32: {
    type: 'S',
    decls: 'struct S { x : array<u32, 4> }',
    validity: 'non-uniform'
  },
  struct_runtime_array_u32: {
    type: 'S',
    decls: 'struct S { x : array<u32> }',
    validity: 'storage'
  },
  array_struct_size_5: {
    type: 'array<S, 16>',
    decls: 'struct S { @size(5) x : u32, y : u32 }',
    validity: 'non-uniform'
  },
  array_struct_size_5x2: {
    type: 'array<S, 16>',
    decls: 'struct S { @size(5) x : u32, @size(5) y : u32 }',
    validity: true
  },
  struct_size_5: {
    type: 'S',
    decls: `struct T { @size(5) x : u32 }
    struct S { x : u32, y : T }`,
    validity: 'non-uniform'
  },
  struct_size_5_align16: {
    type: 'S',
    decls: `struct T { @align(16) @size(5) x : u32 }
    struct S { x : u32, y : T }`,
    validity: true
  }
};

g.test('layout_constraints').
desc('Test address space layout constraints').
params((u) =>
u.
combine('case', keysOf(kLayoutCases)).
beginSubcases().
combine('aspace', ['storage', 'uniform', 'function', 'private', 'workgroup'])
).
beforeAllSubcases((t) => {
  const testcase = kLayoutCases[t.params.case];
  if (testcase.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kLayoutCases[t.params.case];
  const decls = testcase.decls !== undefined ? testcase.decls : '';
  let code = `
${testcase.f16 ? 'enable f16;' : ''}
${decls}

`;

  switch (t.params.aspace) {
    case 'storage':
      code += `@group(0) @binding(0) var<storage, read_write> v : ${testcase.type};\n`;
      break;
    case 'uniform':
      code += `@group(0) @binding(0) var<uniform> v : ${testcase.type};\n`;
      break;
    case 'workgroup':
      code += `var<workgroup> v : ${testcase.type};\n`;
      break;
    case 'private':
      code += `var<private> v : ${testcase.type};\n`;
      break;
    default:
      break;
  }

  code += `@compute @workgroup_size(1,1,1)
    fn main() {
    `;

  if (t.params.aspace === 'function') {
    code += `var v : ${testcase.type};\n`;
  }
  code += `}\n`;

  const is_interface = t.params.aspace === 'uniform' || t.params.aspace === 'storage';
  const supports_atomic = t.params.aspace === 'storage' || t.params.aspace === 'workgroup';
  const expect =
  testcase.validity === true ||
  testcase.validity === 'non-uniform' && t.params.aspace !== 'uniform' ||
  testcase.validity === 'non-interface' && !is_interface ||
  testcase.validity === 'storage' && t.params.aspace === 'storage' ||
  testcase.validity === 'atomic' && supports_atomic;
  t.expectCompileResult(expect, code);
});