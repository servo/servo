/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for type aliases
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('no_direct_recursion').
desc('Test that direct recursion of type aliases is rejected').
params((u) => u.combine('target', ['i32', 'T'])).
fn((t) => {
  const wgsl = `alias T = ${t.params.target};`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion').
desc('Test that indirect recursion of type aliases is rejected').
params((u) => u.combine('target', ['i32', 'S'])).
fn((t) => {
  const wgsl = `
alias S = T;
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_vector_element').
desc('Test that indirect recursion of type aliases via vector element types is rejected').
params((u) => u.combine('target', ['i32', 'V'])).
fn((t) => {
  const wgsl = `
alias V = vec4<T>;
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_matrix_element').
desc('Test that indirect recursion of type aliases via matrix element types is rejected').
params((u) => u.combine('target', ['f32', 'M'])).
fn((t) => {
  const wgsl = `
alias M = mat4x4<T>;
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'f32', wgsl);
});

g.test('no_indirect_recursion_via_array_element').
desc('Test that indirect recursion of type aliases via array element types is rejected').
params((u) => u.combine('target', ['i32', 'A'])).
fn((t) => {
  const wgsl = `
alias A = array<T, 4>;
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_array_size').
desc('Test that indirect recursion of type aliases via array size expressions is rejected').
params((u) => u.combine('target', ['i32', 'A'])).
fn((t) => {
  const wgsl = `
alias A = array<i32, T(1)>;
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_atomic').
desc('Test that indirect recursion of type aliases via atomic types is rejected').
params((u) => u.combine('target', ['i32', 'A'])).
fn((t) => {
  const wgsl = `
alias A = atomic<T>;
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_ptr_store_type').
desc('Test that indirect recursion of type aliases via pointer store types is rejected').
params((u) => u.combine('target', ['i32', 'P'])).
fn((t) => {
  const wgsl = `
alias P = ptr<function, T>;
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_struct_member').
desc('Test that indirect recursion of type aliases via struct members is rejected').
params((u) => u.combine('target', ['i32', 'S'])).
fn((t) => {
  const wgsl = `
struct S {
  a : T
}
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_struct_attribute').
desc('Test that indirect recursion of type aliases via struct members is rejected').
params((u) =>
u //
.combine('target', ['i32', 'S']).
combine('attribute', ['align', 'location', 'size'])
).
fn((t) => {
  const wgsl = `
struct S {
  @${t.params.attribute}(T(4)) a : f32
}
alias T = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

const kTypes = [
'bool',
'i32',
'u32',
'f32',
'f16',
'vec2<i32>',
'vec3<u32>',
'vec4<f32>',
'mat2x2<f32>',
'mat2x3<f32>',
'mat2x4<f32>',
'mat3x2<f32>',
'mat3x3<f32>',
'mat3x4<f32>',
'mat4x2<f32>',
'mat4x3<f32>',
'mat4x4<f32>',
'array<u32>',
'array<i32, 4>',
'array<vec2<u32>, 8>',
'S',
'T',
'atomic<u32>',
'atomic<i32>',
'ptr<function, u32>',
'ptr<private, i32>',
'ptr<workgroup, f32>',
'ptr<uniform, vec2f>',
'ptr<storage, vec2u>',
'ptr<storage, vec3i, read>',
'ptr<storage, vec4f, read_write>',
'sampler',
'sampler_comparison',
'texture_1d<f32>',
'texture_2d<u32>',
'texture_2d_array<i32>',
'texture_3d<f32>',
'texture_cube<i32>',
'texture_cube_array<u32>',
'texture_multisampled_2d<f32>',
'texture_depth_multisampled_2d',
'texture_external',
'texture_storage_1d<rgba8snorm, write>',
'texture_storage_1d<r32uint, write>',
'texture_storage_1d<r32sint, read_write>',
'texture_storage_1d<r32float, read>',
'texture_storage_2d<rgba16uint, write>',
'texture_storage_2d_array<rgba32float, write>',
'texture_storage_3d<bgra8unorm, write>',
'texture_depth_2d',
'texture_depth_2d_array',
'texture_depth_cube',
'texture_depth_cube_array',

// Pre-declared aliases (spot check)
'vec2f',
'vec3u',
'vec4i',
'mat2x2f',

// User-defined aliases
'anotherAlias',
'random_alias'];


g.test('any_type').
desc('Test that any type can be aliased').
params((u) => u.combine('type', kTypes)).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const ty = t.params.type;
  t.skipIf(
    ty.includes('texture_storage') &&
    ty.includes('read') &&
    !t.hasLanguageFeature('readonly_and_readwrite_storage_textures'),
    'Missing language feature'
  );
  const enable = ty === 'f16' ? 'enable f16;' : '';
  const code = `
    ${enable}
    struct S { x : u32 }
    struct T { y : S }
    alias anotherAlias = u32;
    alias random_alias = i32;
    alias myType = ${ty};`;
  t.expectCompileResult(true, code);
});

const kMatchCases = {
  function_param: `
    fn foo(x : u32) { }
    fn bar() {
      var x : alias_alias_u32;
      foo(x);
    }`,
  constructor: `var<private> v : u32 = alias_u32(1);`,
  template_param: `var<private> v : vec2<alias_u32> = vec2<u32>();`,
  predeclared_alias: `var<private> v : vec2<alias_alias_u32> = vec2u();`,
  struct_element: `
    struct S { x : alias_u32 }
    const c_u32 = 0u;
    const c = S(c_u32);`
};

g.test('match_non_alias').
desc('Test that type checking succeeds using aliased and unaliased type').
params((u) => u.combine('case', keysOf(kMatchCases))).
fn((t) => {
  const testcase = kMatchCases[t.params.case];
  const code = `
    alias alias_u32 = u32;
    alias alias_alias_u32 = alias_u32;
    ${testcase}`;
  t.expectCompileResult(true, code);
});