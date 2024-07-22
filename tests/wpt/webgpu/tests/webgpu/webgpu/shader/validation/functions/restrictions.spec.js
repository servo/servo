/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for function restrictions`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);







const kCCommonTypeDecls = `
struct runtime_array_struct {
  arr : array<u32>
}

struct constructible {
  a : i32,
  b : u32,
  c : f32,
  d : bool,
}

struct host_shareable {
  a : i32,
  b : u32,
  c : f32,
}

struct struct_with_array {
  a : array<constructible, 4>
}

`;

const kVertexPosCases = {
  bare_position: { name: `@builtin(position) vec4f`, value: `vec4f()`, valid: true },
  nested_position: { name: `pos_struct`, value: `pos_struct()`, valid: true },
  no_bare_position: { name: `vec4f`, value: `vec4f()`, valid: false },
  no_nested_position: { name: `no_pos_struct`, value: `no_pos_struct()`, valid: false }
};

g.test('vertex_returns_position').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-restriction').
desc(`Test that a vertex shader should return position`).
params((u) => u.combine('case', keysOf(kVertexPosCases))).
fn((t) => {
  const testcase = kVertexPosCases[t.params.case];
  const code = `
struct pos_struct {
  @builtin(position) pos : vec4f
}

struct no_pos_struct {
  @location(0) x : vec4f
}

@vertex
fn main() -> ${testcase.name} {
  return ${testcase.value};
}`;

  t.expectCompileResult(testcase.valid, code);
});

g.test('entry_point_call_target').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-restriction').
desc(`Test that an entry point cannot be the target of a function call`).
params((u) =>
u.
combine('stage', ['@fragment', '@vertex', '@compute @workgroup_size(1,1,1)']).
combine('entry_point', ['with', 'without'])
).
fn((t) => {
  const use_attr = t.params.entry_point === 'with';
  let ret_attr = '';
  if (use_attr && t.params.stage === '@vertex') {
    ret_attr = '@builtin(position)';
  }
  const ret = t.params.stage.indexOf('@vertex') === 0 ? `-> ${ret_attr} vec4f` : '';
  const ret_value = t.params.stage.indexOf('@vertex') === 0 ? `return vec4f();` : '';
  const call = t.params.stage.indexOf('@vertex') === 0 ? 'let tmp = bar();' : 'bar();';
  const stage_attr = use_attr ? t.params.stage : '';
  const code = `
${stage_attr}
fn bar() ${ret} {
  ${ret_value}
}

fn foo() {
  ${call}
}
`;
  t.expectCompileResult(!use_attr, code);
});







const kFunctionRetTypeCases = {
  // Constructible types,
  u32: { name: `u32`, value: ``, valid: true },
  i32: { name: `i32`, value: ``, valid: true },
  f32: { name: `f32`, value: ``, valid: true },
  bool: { name: `bool`, value: ``, valid: true },
  f16: { name: `f16`, value: ``, valid: true },
  vec2: { name: `vec2u`, value: ``, valid: true },
  vec3: { name: `vec3i`, value: ``, valid: true },
  vec4: { name: `vec4f`, value: ``, valid: true },
  mat2x2: { name: `mat2x2f`, value: ``, valid: true },
  mat2x3: { name: `mat2x3f`, value: ``, valid: true },
  mat2x4: { name: `mat2x4f`, value: ``, valid: true },
  mat3x2: { name: `mat3x2f`, value: ``, valid: true },
  mat3x3: { name: `mat3x3f`, value: ``, valid: true },
  mat3x4: { name: `mat3x4f`, value: ``, valid: true },
  mat4x2: { name: `mat4x2f`, value: ``, valid: true },
  mat4x3: { name: `mat4x3f`, value: ``, valid: true },
  mat4x4: { name: `mat4x4f`, value: ``, valid: true },
  array1: { name: `array<u32, 4>`, value: ``, valid: true },
  array2: { name: `array<vec2f, 2>`, value: ``, valid: true },
  array3: { name: `array<constructible, 4>`, value: ``, valid: true },
  array4: { name: `array<mat2x2f, 4>`, value: ``, valid: true },
  array5: { name: `array<bool, 4>`, value: ``, valid: true },
  struct1: { name: `constructible`, value: ``, valid: true },
  struct2: { name: `struct_with_array`, value: ``, valid: true },

  // Non-constructible types.
  runtime_array: { name: `array<u32>`, value: ``, valid: false },
  runtime_struct: { name: `runtime_array_struct`, value: ``, valid: false },
  override_array: { name: `array<u32, override_size>`, value: ``, valid: false },
  atomic_u32: { name: `atomic<u32>`, value: `atomic_wg`, valid: false },
  atomic_struct: { name: `atomic_struct`, value: ``, valid: false },
  texture_sample: { name: `texture_2d<f32>`, value: `t`, valid: false },
  texture_depth: { name: `texture_depth_2d`, value: `t_depth`, valid: false },
  texture_multisampled: {
    name: `texture_multisampled_2d<f32>`,
    value: `t_multisampled`,
    valid: false
  },
  texture_storage: {
    name: `texture_storage_2d<rgba8unorm, write>`,
    value: `t_storage`,
    valid: false
  },
  sampler: { name: `sampler`, value: `s`, valid: false },
  sampler_comparison: { name: `sampler_comparison`, value: `s_depth`, valid: false },
  ptr: { name: `ptr<workgroup, atomic<u32>>`, value: `&atomic_wg`, valid: false }
};

g.test('function_return_types').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-restriction').
desc(`Test that function return types must be constructible`).
params((u) => u.combine('case', keysOf(kFunctionRetTypeCases))).
beforeAllSubcases((t) => {
  if (kFunctionRetTypeCases[t.params.case].name === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kFunctionRetTypeCases[t.params.case];
  const enable = testcase.name === 'f16' ? 'enable f16;' : '';
  const value = testcase.value === '' ? `${testcase.name}()` : testcase.value;
  const code = `
${enable}

${kCCommonTypeDecls}

struct atomic_struct {
  a : atomic<u32>
};

override override_size : u32;

var<workgroup> atomic_wg : atomic<u32>;

@group(0) @binding(0)
var t : texture_2d<f32>;
@group(0) @binding(1)
var s : sampler;
@group(0) @binding(2)
var s_depth : sampler_comparison;
@group(0) @binding(3)
var t_storage : texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4)
var t_depth : texture_depth_2d;
@group(0) @binding(5)
var t_multisampled : texture_multisampled_2d<f32>;
@group(0) @binding(6)
var t_external : texture_external;

fn foo() -> ${testcase.name} {
  return ${value};
}`;

  t.expectCompileResult(testcase.valid, code);
});






const kFunctionParamTypeCases = {
  // Constructible types,
  u32: { name: `u32`, valid: true },
  i32: { name: `i32`, valid: true },
  f32: { name: `f32`, valid: true },
  bool: { name: `bool`, valid: true },
  f16: { name: `f16`, valid: true },
  vec2: { name: `vec2u`, valid: true },
  vec3: { name: `vec3i`, valid: true },
  vec4: { name: `vec4f`, valid: true },
  mat2x2: { name: `mat2x2f`, valid: true },
  mat2x3: { name: `mat2x3f`, valid: true },
  mat2x4: { name: `mat2x4f`, valid: true },
  mat3x2: { name: `mat3x2f`, valid: true },
  mat3x3: { name: `mat3x3f`, valid: true },
  mat3x4: { name: `mat3x4f`, valid: true },
  mat4x2: { name: `mat4x2f`, valid: true },
  mat4x3: { name: `mat4x3f`, valid: true },
  mat4x4: { name: `mat4x4f`, valid: true },
  array1: { name: `array<u32, 4>`, valid: true },
  array2: { name: `array<vec2f, 2>`, valid: true },
  array3: { name: `array<constructible, 4>`, valid: true },
  array4: { name: `array<mat2x2f, 4>`, valid: true },
  array5: { name: `array<bool, 4>`, valid: true },
  struct1: { name: `constructible`, valid: true },
  struct2: { name: `struct_with_array`, valid: true },

  // Non-constructible types.
  runtime_array: { name: `array<u32>`, valid: false },
  runtime_struct: { name: `runtime_array_struct`, valid: false },
  override_array: { name: `array<u32, override_size>`, valid: false },
  atomic_u32: { name: `atomic<u32>`, valid: false },
  atomic_struct: { name: `atomic_struct`, valid: false },

  // Textures and samplers.
  texture_sample: { name: `texture_2d<f32>`, valid: true },
  texture_depth: { name: `texture_depth_2d`, valid: true },
  texture_multisampled: {
    name: `texture_multisampled_2d<f32>`,
    valid: true
  },
  texture_storage: { name: `texture_storage_2d<rgba8unorm, write>`, valid: true },
  sampler: { name: `sampler`, valid: true },
  sampler_comparison: { name: `sampler_comparison`, valid: true },

  // Valid pointers.
  ptr1: { name: `ptr<function, u32>`, valid: true },
  ptr2: { name: `ptr<function, constructible>`, valid: true },
  ptr3: { name: `ptr<private, u32>`, valid: true },
  ptr4: { name: `ptr<private, constructible>`, valid: true },

  // Pointers only valid with unrestricted_pointer_parameters
  ptr5: { name: `ptr<storage, u32>`, valid: 'with_unrestricted_pointer_parameters' },
  ptr6: { name: `ptr<storage, u32, read>`, valid: 'with_unrestricted_pointer_parameters' },
  ptr7: { name: `ptr<storage, u32, read_write>`, valid: 'with_unrestricted_pointer_parameters' },
  ptr8: { name: `ptr<uniform, u32>`, valid: 'with_unrestricted_pointer_parameters' },
  ptr9: { name: `ptr<workgroup, u32>`, valid: 'with_unrestricted_pointer_parameters' },
  ptr10: {
    name: `ptr<storage, host_shareable, read_write>`,
    valid: 'with_unrestricted_pointer_parameters'
  },
  ptr11: {
    name: `ptr<storage, host_shareable, read>`,
    valid: 'with_unrestricted_pointer_parameters'
  },
  ptr12: {
    name: `ptr<uniform, host_shareable>`,
    valid: 'with_unrestricted_pointer_parameters'
  },
  ptrWorkgroupAtomic: {
    name: `ptr<workgroup, atomic<u32>>`,
    valid: 'with_unrestricted_pointer_parameters'
  },
  ptrWorkgroupNestedAtomic: {
    name: `ptr<workgroup, array<atomic<u32>,1>>`,
    valid: 'with_unrestricted_pointer_parameters'
  },

  // Invalid pointers.
  invalid_ptr1: { name: `ptr<handle, u32>`, valid: false }, // Can't spell handle address space
  invalid_ptr2: { name: `ptr<not_an_address_space, u32>`, valid: false },
  invalid_ptr3: { name: `ptr<storage>`, valid: false }, // No store type
  invalid_ptr4: { name: `ptr<private,u32,read>`, valid: false }, // Can't specify access mode
  invalid_ptr5: { name: `ptr<private,u32,write>`, valid: false }, // Can't specify access mode
  invalid_ptr6: { name: `ptr<private,u32,read_write>`, valid: false }, // Can't specify access mode
  invalid_ptr7: { name: `ptr<private,clamp>`, valid: false }, // Invalid store type
  invalid_ptr8: { name: `ptr<function, texture_external>`, valid: false } // non-constructible pointer type
};

g.test('function_parameter_types').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-restriction').
desc(`Test validation of user-declared function parameter types`).
params((u) => u.combine('case', keysOf(kFunctionParamTypeCases))).
beforeAllSubcases((t) => {
  if (kFunctionParamTypeCases[t.params.case].name === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kFunctionParamTypeCases[t.params.case];
  const enable = testcase.name === 'f16' ? 'enable f16;' : '';
  const code = `
${enable}

${kCCommonTypeDecls}

fn foo(param : ${testcase.name}) {
}`;

  let isValid = testcase.valid;
  if (isValid === 'with_unrestricted_pointer_parameters') {
    isValid = t.hasLanguageFeature('unrestricted_pointer_parameters');
  }

  t.expectCompileResult(isValid, code);
});







const kFunctionParamValueCases = {
  // Values
  u32_literal: { value: `0u`, matches: ['u32'] },
  i32_literal: { value: `0i`, matches: ['i32'] },
  f32_literal: { value: `0f`, matches: ['f32'] },
  bool_literal: { value: `false`, matches: ['bool'] },
  abstract_int_literal: { value: `0`, matches: ['u32', 'i32', 'f32', 'f16'] },
  abstract_float_literal: { value: `0.0`, matches: ['f32', 'f16'] },
  vec2u_constructor: { value: `vec2u()`, matches: ['vec2'] },
  vec2i_constructor: { value: `vec2i()`, matches: [] },
  vec2f_constructor: { value: `vec2f()`, matches: [] },
  vec2b_constructor: { value: `vec2<bool>()`, matches: [] },
  vec3u_constructor: { value: `vec3u()`, matches: [] },
  vec3i_constructor: { value: `vec3i()`, matches: ['vec3'] },
  vec3f_constructor: { value: `vec3f()`, matches: [] },
  vec3b_constructor: { value: `vec3<bool>()`, matches: [] },
  vec4u_constructor: { value: `vec4u()`, matches: [] },
  vec4i_constructor: { value: `vec4i()`, matches: [] },
  vec4f_constructor: { value: `vec4f()`, matches: ['vec4'] },
  vec4b_constructor: { value: `vec4<bool>()`, matches: [] },
  vec2_abstract_int: { value: `vec2(0,0)`, matches: ['vec2'] },
  vec2_abstract_float: { value: `vec2(0.0,0)`, matches: [] },
  vec3_abstract_int: { value: `vec3(0,0,0)`, matches: ['vec3'] },
  vec3_abstract_float: { value: `vec3(0.0,0,0)`, matches: [] },
  vec4_abstract_int: { value: `vec4(0,0,0,0)`, matches: ['vec4'] },
  vec4_abstract_float: { value: `vec4(0.0,0,0,0)`, matches: ['vec4'] },
  mat2x2_constructor: { value: `mat2x2f()`, matches: ['mat2x2'] },
  mat2x3_constructor: { value: `mat2x3f()`, matches: ['mat2x3'] },
  mat2x4_constructor: { value: `mat2x4f()`, matches: ['mat2x4'] },
  mat3x2_constructor: { value: `mat3x2f()`, matches: ['mat3x2'] },
  mat3x3_constructor: { value: `mat3x3f()`, matches: ['mat3x3'] },
  mat3x4_constructor: { value: `mat3x4f()`, matches: ['mat3x4'] },
  mat4x2_constructor: { value: `mat4x2f()`, matches: ['mat4x2'] },
  mat4x3_constructor: { value: `mat4x3f()`, matches: ['mat4x3'] },
  mat4x4_constructor: { value: `mat4x4f()`, matches: ['mat4x4'] },
  array1_constructor: { value: `array<u32, 4>()`, matches: ['array1'] },
  array2_constructor: { value: `array<vec2f, 2>()`, matches: ['array2'] },
  array3_constructor: { value: `array<constructible, 4>()`, matches: ['array3'] },
  array4_constructor: { value: `array<mat2x2f, 4>()`, matches: ['array4'] },
  array5_constructor: { value: `array<bool, 4>()`, matches: ['array5'] },
  struct1_constructor: { value: `constructible()`, matches: ['struct1'] },
  struct2_constructor: { value: `struct_with_array()`, matches: ['struct2'] },

  // Variable references
  g_u32: { value: `g_u32`, matches: ['u32'] },
  g_i32: { value: `g_i32`, matches: ['i32'] },
  g_f32: { value: `g_f32`, matches: ['f32'] },
  g_bool: { value: `g_bool`, matches: ['bool'] },
  g_vec2: { value: `g_vec2`, matches: ['vec2'] },
  g_vec3: { value: `g_vec3`, matches: ['vec3'] },
  g_vec4: { value: `g_vec4`, matches: ['vec4'] },
  g_mat2x2: { value: `g_mat2x2`, matches: ['mat2x2'] },
  g_mat2x3: { value: `g_mat2x3`, matches: ['mat2x3'] },
  g_mat2x4: { value: `g_mat2x4`, matches: ['mat2x4'] },
  g_mat3x2: { value: `g_mat3x2`, matches: ['mat3x2'] },
  g_mat3x3: { value: `g_mat3x3`, matches: ['mat3x3'] },
  g_mat3x4: { value: `g_mat3x4`, matches: ['mat3x4'] },
  g_mat4x2: { value: `g_mat4x2`, matches: ['mat4x2'] },
  g_mat4x3: { value: `g_mat4x3`, matches: ['mat4x3'] },
  g_mat4x4: { value: `g_mat4x4`, matches: ['mat4x4'] },
  g_array1: { value: `g_array1`, matches: ['array1'] },
  g_array2: { value: `g_array2`, matches: ['array2'] },
  g_array3: { value: `g_array3`, matches: ['array3'] },
  g_array4: { value: `g_array4`, matches: ['array4'] },
  g_array5: { value: `g_array5`, matches: ['array5'] },
  g_constructible: { value: `g_constructible`, matches: ['struct1'] },
  g_struct_with_array: { value: `g_struct_with_array`, matches: ['struct2'] },
  f_u32: { value: `f_u32`, matches: ['u32'] },
  f_i32: { value: `f_i32`, matches: ['i32'] },
  f_f32: { value: `f_f32`, matches: ['f32'] },
  f_bool: { value: `f_bool`, matches: ['bool'] },
  f_vec2: { value: `f_vec2`, matches: ['vec2'] },
  f_vec3: { value: `f_vec3`, matches: ['vec3'] },
  f_vec4: { value: `f_vec4`, matches: ['vec4'] },
  f_mat2x2: { value: `f_mat2x2`, matches: ['mat2x2'] },
  f_mat2x3: { value: `f_mat2x3`, matches: ['mat2x3'] },
  f_mat2x4: { value: `f_mat2x4`, matches: ['mat2x4'] },
  f_mat3x2: { value: `f_mat3x2`, matches: ['mat3x2'] },
  f_mat3x3: { value: `f_mat3x3`, matches: ['mat3x3'] },
  f_mat3x4: { value: `f_mat3x4`, matches: ['mat3x4'] },
  f_mat4x2: { value: `f_mat4x2`, matches: ['mat4x2'] },
  f_mat4x3: { value: `f_mat4x3`, matches: ['mat4x3'] },
  f_mat4x4: { value: `f_mat4x4`, matches: ['mat4x4'] },
  f_array1: { value: `f_array1`, matches: ['array1'] },
  f_array2: { value: `f_array2`, matches: ['array2'] },
  f_array3: { value: `f_array3`, matches: ['array3'] },
  f_array4: { value: `f_array4`, matches: ['array4'] },
  f_array5: { value: `f_array5`, matches: ['array5'] },
  f_constructible: { value: `f_constructible`, matches: ['struct1'] },
  f_struct_with_array: { value: `f_struct_with_array`, matches: ['struct2'] },
  g_index_u32: { value: `g_constructible.b`, matches: ['u32'] },
  g_index_i32: { value: `g_constructible.a`, matches: ['i32'] },
  g_index_f32: { value: `g_constructible.c`, matches: ['f32'] },
  g_index_bool: { value: `g_constructible.d`, matches: ['bool'] },
  f_index_u32: { value: `f_constructible.b`, matches: ['u32'] },
  f_index_i32: { value: `f_constructible.a`, matches: ['i32'] },
  f_index_f32: { value: `f_constructible.c`, matches: ['f32'] },
  f_index_bool: { value: `f_constructible.d`, matches: ['bool'] },
  g_array_index_u32: { value: `g_struct_with_array.a[0].b`, matches: ['u32'] },
  g_array_index_i32: { value: `g_struct_with_array.a[1].a`, matches: ['i32'] },
  g_array_index_f32: { value: `g_struct_with_array.a[2].c`, matches: ['f32'] },
  g_array_index_bool: { value: `g_struct_with_array.a[3].d`, matches: ['bool'] },
  f_array_index_u32: { value: `f_struct_with_array.a[0].b`, matches: ['u32'] },
  f_array_index_i32: { value: `f_struct_with_array.a[1].a`, matches: ['i32'] },
  f_array_index_f32: { value: `f_struct_with_array.a[2].c`, matches: ['f32'] },
  f_array_index_bool: { value: `f_struct_with_array.a[3].d`, matches: ['bool'] },

  // Textures and samplers
  texture_sample: { value: `t`, matches: ['texture_sample'] },
  texture_depth: { value: `t_depth`, matches: ['texture_depth'] },
  texture_multisampled: { value: `t_multisampled`, matches: ['texture_multisampled'] },
  texture_storage: { value: `t_storage`, matches: ['texture_storage'] },
  texture_external: { value: `t_external`, matches: ['texture_external'] },
  sampler: { value: `s`, matches: ['sampler'] },
  sampler_comparison: { value: `s_depth`, matches: ['sampler_comparison'] },

  // Pointers
  ptr1: { value: `&f_u32`, matches: ['ptr1'] },
  ptr2: { value: `&f_constructible`, matches: ['ptr2'] },
  ptr3: { value: `&g_u32`, matches: ['ptr3'] },
  ptr4: { value: `&g_constructible`, matches: ['ptr4'] },

  ptr_let1: { value: `ptr_f_u32`, matches: ['ptr1'] },
  ptr_let2: { value: `ptr_f_constructible`, matches: ['ptr2'] },
  ptr_let3: { value: `ptr_g_u32`, matches: ['ptr3'] },
  ptr_let4: { value: `ptr_g_constructible`, matches: ['ptr4'] },
  ptr_let5: { value: `let_let_f_u32`, matches: ['ptr1'] },

  // Requires 'unrestricted_pointer_parameters' WGSL feature
  ptr5: {
    value: `&f_constructible.b`,
    matches: ['ptr1'],
    needsUnrestrictedPointerParameters: true
  },
  ptr6: {
    value: `&g_constructible.b`,
    matches: ['ptr3'],
    needsUnrestrictedPointerParameters: true
  },
  ptr7: {
    value: `&f_struct_with_array.a[1].b`,
    matches: ['ptr1'],
    needsUnrestrictedPointerParameters: true
  },
  ptr8: {
    value: `&g_struct_with_array.a[2]`,
    matches: ['ptr4'],
    needsUnrestrictedPointerParameters: true
  },
  ptr9: {
    value: `&ro_host_shareable.b`,
    matches: ['ptr5', 'ptr6'],
    needsUnrestrictedPointerParameters: true
  },
  ptr10: {
    value: `&rw_host_shareable`,
    matches: ['ptr10'],
    needsUnrestrictedPointerParameters: true
  },
  ptr11: {
    value: `&ro_host_shareable`,
    matches: ['ptr11'],
    needsUnrestrictedPointerParameters: true
  },
  ptr12: {
    value: `&uniform_host_shareable`,
    matches: ['ptr12'],
    needsUnrestrictedPointerParameters: true
  }
};

function parameterMatches(decl, matches) {
  for (const val of matches) {
    if (decl === val) {
      return true;
    }
  }
  return false;
}

g.test('function_parameter_matching').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-restriction').
desc(
  `Test that function parameter types match function parameter type on user-declared functions`
).
params((u) =>
u.
combine('decl', keysOf(kFunctionParamTypeCases)).
filter((u) => {
  return kFunctionParamTypeCases[u.decl].valid !== false;
}).
beginSubcases().
combine('arg', keysOf(kFunctionParamValueCases))
).
beforeAllSubcases((t) => {
  if (kFunctionParamTypeCases[t.params.decl].name === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const param = kFunctionParamTypeCases[t.params.decl];
  const arg = kFunctionParamValueCases[t.params.arg];
  const enable = param.name === 'f16' ? 'enable f16;' : '';
  const code = `
${enable}

${kCCommonTypeDecls}
@group(0) @binding(0)
var t : texture_2d<f32>;
@group(0) @binding(1)
var s : sampler;
@group(0) @binding(2)
var s_depth : sampler_comparison;
@group(0) @binding(3)
var t_storage : texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4)
var t_depth : texture_depth_2d;
@group(0) @binding(5)
var t_multisampled : texture_multisampled_2d<f32>;
@group(0) @binding(6)
var t_external : texture_external;

@group(1) @binding(0)
var<storage> ro_host_shareable : host_shareable;
@group(1) @binding(1)
var<storage, read_write> rw_host_shareable : host_shareable;
@group(1) @binding(2)
var<uniform> uniform_host_shareable : host_shareable;

fn bar(param : ${param.name}) { }

var<private> g_u32 : u32;
var<private> g_i32 : i32;
var<private> g_f32 : f32;
var<private> g_bool : bool;
var<private> g_vec2 : vec2u;
var<private> g_vec3 : vec3i;
var<private> g_vec4 : vec4f;
var<private> g_mat2x2 : mat2x2f;
var<private> g_mat2x3 : mat2x3f;
var<private> g_mat2x4 : mat2x4f;
var<private> g_mat3x2 : mat3x2f;
var<private> g_mat3x3 : mat3x3f;
var<private> g_mat3x4 : mat3x4f;
var<private> g_mat4x2 : mat4x2f;
var<private> g_mat4x3 : mat4x3f;
var<private> g_mat4x4 : mat4x4f;
var<private> g_array1 : array<u32, 4>;
var<private> g_array2 : array<vec2f, 2>;
var<private> g_array3 : array<constructible, 4>;
var<private> g_array4 : array<mat2x2f, 4>;
var<private> g_array5 : array<bool, 4>;
var<private> g_constructible : constructible;
var<private> g_struct_with_array : struct_with_array;

fn foo() {
  var f_u32 : u32;
  var f_i32 : i32;
  var f_f32 : f32;
  var f_bool : bool;
  var f_vec2 : vec2u;
  var f_vec3 : vec3i;
  var f_vec4 : vec4f;
  var f_mat2x2 : mat2x2f;
  var f_mat2x3 : mat2x3f;
  var f_mat2x4 : mat2x4f;
  var f_mat3x2 : mat3x2f;
  var f_mat3x3 : mat3x3f;
  var f_mat3x4 : mat3x4f;
  var f_mat4x2 : mat4x2f;
  var f_mat4x3 : mat4x3f;
  var f_mat4x4 : mat4x4f;
  var f_array1 : array<u32, 4>;
  var f_array2 : array<vec2f, 2>;
  var f_array3 : array<constructible, 4>;
  var f_array4 : array<mat2x2f, 4>;
  var f_array5 : array<bool, 4>;
  var f_constructible : constructible;
  var f_struct_with_array : struct_with_array;
  let ptr_f_u32 = &f_u32;
  let ptr_f_constructible = &f_constructible;
  let ptr_g_u32 = &g_u32;
  let ptr_g_constructible = &g_constructible;
  let let_let_f_u32 = ptr_f_u32;

  bar(${arg.value});
}
`;

  const needsUnrestrictedPointerParameters =
  (kFunctionParamTypeCases[t.params.decl].valid === 'with_unrestricted_pointer_parameters' ||
  arg.needsUnrestrictedPointerParameters) ??
  false;

  let isValid = parameterMatches(t.params.decl, arg.matches);
  if (isValid && needsUnrestrictedPointerParameters) {
    isValid = t.hasLanguageFeature('unrestricted_pointer_parameters');
  }

  t.expectCompileResult(isValid, code);
});

g.test('no_direct_recursion').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-restriction').
desc(`Test that functions cannot be directly recursive`).
fn((t) => {
  const code = `
fn foo() {
  foo();
}`;

  t.expectCompileResult(false, code);
});

g.test('no_indirect_recursion').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-restriction').
desc(`Test that functions cannot be indirectly recursive`).
fn((t) => {
  const code = `
fn bar() {
  foo();
}
fn foo() {
  bar();
}`;

  t.expectCompileResult(false, code);
});

g.test('param_names_must_differ').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-declaration-sec').
desc(`Test that function parameters must have different names`).
params((u) => u.combine('p1', ['a', 'b', 'c']).combine('p2', ['a', 'b', 'c'])).
fn((t) => {
  const code = `fn foo(${t.params.p1} : u32, ${t.params.p2} : f32) { }`;
  t.expectCompileResult(t.params.p1 !== t.params.p2, code);
});

const kParamUseCases = {
  body: `fn foo(param : u32) {
    let tmp = param;
  }`,
  var: `var<private> v : u32 = param;
  fn foo(param : u32) { }`,
  const: `const c : u32 = param;
  fn foo(param : u32) { }`,
  override: `override o : u32 = param;
  fn foo(param : u32) { }`,
  function: `fn bar() { let tmp = param; }
  fn foo(param : u32) { }`
};

g.test('param_scope_is_function_body').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-declaration-sec').
desc(`Test that function parameters are only in scope in the function body`).
params((u) => u.combine('use', keysOf(kParamUseCases))).
fn((t) => {
  t.expectCompileResult(t.params.use === 'body', kParamUseCases[t.params.use]);
});

g.test('param_number_matches_call').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-calls').
desc(`Test that function calls have an equal number of arguments as the number of parameters`).
params((u) =>
u.
combine('num_args', [0, 1, 2, 3, 4, 255]).
combine('num_params', [0, 1, 2, 3, 4, 255])
).
fn((t) => {
  let code = `
    fn bar(`;
  for (let i = 0; i < t.params.num_params; i++) {
    code += `p${i} : u32,`;
  }
  code += `) { }\n`;
  code += `fn foo() {\nbar(`;
  for (let i = 0; i < t.params.num_args; i++) {
    code += `0,`;
  }
  code += `);\n}`;
  t.expectCompileResult(t.params.num_args === t.params.num_params, code);
});

const kParamsTypes = ['u32', 'i32', 'f32'];






const kArgValues = {
  abstract_int: {
    value: '0',
    matches: ['u32', 'i32', 'f32']
  },
  abstract_float: {
    value: '0.0',
    matches: ['f32']
  },
  unsigned_int: {
    value: '0u',
    matches: ['u32']
  },
  signed_int: {
    value: '0i',
    matches: ['i32']
  },
  float: {
    value: '0f',
    matches: ['f32']
  }
};

function checkArgTypeMatch(param_type, arg_matches) {
  for (const match of arg_matches) {
    if (match === param_type) {
      return true;
    }
  }
  return false;
}

g.test('call_arg_types_match_1_param').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-calls').
desc(`Test that the argument types match in order`).
params((u) =>
u.
combine('p1_type', kParamsTypes) //
.beginSubcases().
combine('arg1_value', keysOf(kArgValues))
).
fn((t) => {
  const code = `
fn bar(p1 : ${t.params.p1_type}) { }
fn foo() {
  bar(${kArgValues[t.params.arg1_value].value});
}`;

  const res = checkArgTypeMatch(t.params.p1_type, kArgValues[t.params.arg1_value].matches);
  t.expectCompileResult(res, code);
});

g.test('call_arg_types_match_2_params').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-calls').
desc(`Test that the argument types match in order`).
params((u) =>
u.
combine('p1_type', kParamsTypes).
combine('p2_type', kParamsTypes).
beginSubcases().
combine('arg1_value', keysOf(kArgValues)).
combine('arg2_value', keysOf(kArgValues))
).
fn((t) => {
  const code = `
fn bar(p1 : ${t.params.p1_type}, p2 : ${t.params.p2_type}) { }
fn foo() {
  bar(${kArgValues[t.params.arg1_value].value}, ${kArgValues[t.params.arg2_value].value});
}`;

  const res =
  checkArgTypeMatch(t.params.p1_type, kArgValues[t.params.arg1_value].matches) &&
  checkArgTypeMatch(t.params.p2_type, kArgValues[t.params.arg2_value].matches);
  t.expectCompileResult(res, code);
});

g.test('call_arg_types_match_3_params').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#function-calls').
desc(`Test that the argument types match in order`).
params((u) =>
u.
combine('p1_type', kParamsTypes).
combine('p2_type', kParamsTypes).
combine('p3_type', kParamsTypes).
beginSubcases().
combine('arg1_value', keysOf(kArgValues)).
combine('arg2_value', keysOf(kArgValues)).
combine('arg3_value', keysOf(kArgValues))
).
fn((t) => {
  const code = `
fn bar(p1 : ${t.params.p1_type}, p2 : ${t.params.p2_type}, p3 : ${t.params.p3_type}) { }
fn foo() {
  bar(${kArgValues[t.params.arg1_value].value},
      ${kArgValues[t.params.arg2_value].value},
      ${kArgValues[t.params.arg3_value].value});
}`;

  const res =
  checkArgTypeMatch(t.params.p1_type, kArgValues[t.params.arg1_value].matches) &&
  checkArgTypeMatch(t.params.p2_type, kArgValues[t.params.arg2_value].matches) &&
  checkArgTypeMatch(t.params.p3_type, kArgValues[t.params.arg3_value].matches);
  t.expectCompileResult(res, code);
});

g.test('param_name_can_shadow_function_name').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests that a function parameter can shadow the function name`).
fn((t) => {
  const code = `
fn foo(foo: i32) -> i32 {
  return foo;
}
`;
  t.expectCompileResult(true, code);
});

g.test('param_name_can_shadow_alias').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests that a function parameter can shadow an alias`).
fn((t) => {
  const code = `
alias foo = f32;
fn test(foo: i32) -> i32 {
  return foo;
}
`;
  t.expectCompileResult(true, code);
});

g.test('param_name_can_shadow_global').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests that a function parameter can shadow a global`).
fn((t) => {
  const code = `
const foo: f32 = 1.2f;

fn test(foo: i32) -> i32 {
  return foo;
}
`;
  t.expectCompileResult(true, code);
});

g.test('param_comma_placement').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests validation of commas in function parameter lists`).
params((u) =>
u.
combine('param_1', [true, false]).
combine('param_2', [true, false]).
combine('comma', [true, false])
).
fn((t) => {
  const has_p1 = t.params.param_1;
  const has_p2 = t.params.param_2;
  const has_c = t.params.comma;

  const p1 = has_p1 ? 'foo: i32' : '';
  const p2 = has_p2 ? 'bar: f32' : '';
  const comma = has_c ? ', ' : ' ';

  const code = `
fn test(${p1}${comma}${p2}) {
}
`;

  const success =
  !has_p1 && !has_p2 && !has_c || // no params
  has_p1 && !has_p2 || // only p1, which can have a trailing comma or not
  !has_p1 && has_p2 && !has_c || // just p2
  has_p1 && has_p2 && has_c; // both params comma separated
  t.expectCompileResult(success, code);
});

g.test('param_type_can_be_alias').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests that a function parameter type can be an alias`).
fn((t) => {
  const code = `
alias foo = f32;
fn test(foo: foo) -> foo {
  return foo;
}
`;
  t.expectCompileResult(true, code);
});

g.test('function_name_required').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests the function name is required`).
params((u) => u.combine('name', [true, false])).
fn((t) => {
  const has_name = t.params.name;
  const name = has_name ? 'name' : '';
  const code = `
fn ${name}() -> i32 {
  return 1;
}
`;
  t.expectCompileResult(has_name, code);
});

g.test('param_type_required').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests the parameter type is required`).
params((u) => u.combine('ty', [true, false]).combine('colon', [true, false])).
fn((t) => {
  const has_ty = t.params.ty;
  const has_colon = t.params.colon;
  const ty = has_ty ? 'i32' : '';
  const colon = has_colon ? ':' : ' ';
  const code = `
fn f(foo${colon}${ty}) -> i32 {
  return 1;
}
`;
  t.expectCompileResult(has_ty && has_colon, code);
});

g.test('body_required').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests the function body is required`).
params((u) => u.combine('body', ['braces', 'semi', ''])).
fn((t) => {
  const body = t.params.body === 'braces' ? '{}' : t.params.body === 'semi' ? ';' : '';

  const code = `
fn f() ${body}

fn other() {}
`;
  t.expectCompileResult(body === '{}', code);
});

g.test('parens_required').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests that the parens for a function are required`).
params((u) => u.combine('parens', [true, false]).combine('param', [true, false])).
fn((t) => {
  const has_parens = t.params.parens;
  const has_param = t.params.param;

  let args = '';
  if (has_parens) {
    args += '(';
  }
  if (has_param) {
    args += 'foo: i32';
  }

  if (has_parens) {
    args += ')';
  }

  const code = `
fn f ${args} {}
`;
  t.expectCompileResult(has_parens, code);
});

g.test('non_module_scoped_function').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests that a non-module-scope function is rejected`).
params((u) => u.combine('loc', ['inner', 'outer'])).
fn((t) => {
  const o = `fn a() -> i32 { return 1; }`;

  let inner = '';
  let outer = '';

  if (t.params.loc === 'inner') {
    inner = o;
  } else {
    outer = o;
  }

  const code = `
${outer}
fn b() {
  ${inner}
}
`;
  t.expectCompileResult(t.params.loc === 'outer', code);
});

const kAttributes = {
  align: {
    attr: '@align(5)',
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  binding: {
    attr: '@binding(5)',
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  builtin: {
    attr: '@builtin(position)',
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  compute: {
    attr: `@compute`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  const: {
    attr: '@const',
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  diagnostic: {
    attr: `@diagnostic(off, derivative_uniformity)`,
    pass: {
      func: true,
      param: false,
      ret: false
    }
  },
  fragment: {
    attr: `@fragment`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  group: {
    attr: `@group(1)`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  id: {
    attr: `@id(1)`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  interpolate: {
    attr: `@interpolate(linear, center)`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  invariant: {
    attr: `@invariant`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  location: {
    attr: `@location(0)`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  must_use: {
    attr: `@must_use`,
    pass: {
      func: true,
      param: false,
      ret: false
    }
  },
  size: {
    attr: `@size(10)`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  vertex: {
    attr: `@vertex`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  },
  workgroup_size: {
    attr: `@workgroup_size(1)`,
    pass: {
      func: false,
      param: false,
      ret: false
    }
  }
};

g.test('function_attributes').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests the attributes for a function`).
params((u) =>
u.combine('case', keysOf(kAttributes)).combine('placement', ['func', 'param', 'ret'])
).
fn((t) => {
  const d = kAttributes[t.params.case];
  const func = t.params.placement === 'func';
  const param = t.params.placement === 'param';
  const ret = t.params.placement === 'ret';

  const code = `
${func ? d.attr : ''}
fn b(${param ? d.attr : ''} foo: i32) -> ${ret ? d.attr : ''} i32{
  return 1;
}
`;
  const succeed =
  t.params.placement === 'func' ?
  d.pass.func :
  t.params.placement === 'params' ?
  d.pass.param :
  d.pass.ret;
  t.expectCompileResult(succeed, code);
});

g.test('must_use_requires_return').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests the must_use attribute requires a return`).
params((u) => u.combine('ret', [true, false])).
fn((t) => {
  let ret = '';
  let ret_stmt = '';

  if (t.params.ret) {
    ret = '-> i32';
    ret_stmt = 'return 1;';
  }

  const code = `
@must_use
fn b() ${ret} {
  ${ret_stmt}
}
`;
  t.expectCompileResult(t.params.ret, code);
});

g.test('overload').
specURL('https://www.w3.org/TR/WGSL/#function-declaration-sec').
desc(`Tests that user functions can not overload `).
params((u) => u.combine('overload', [true, false])).
fn((t) => {
  let code = 'fn a(f: i32) {}\n';

  if (t.params.overload) {
    code += 'fn a(f: u32) {}';
  }
  t.expectCompileResult(t.params.overload === false, code);
});