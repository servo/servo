/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for struct types
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('no_direct_recursion').
desc('Test that direct recursion of structures is rejected').
params((u) => u.combine('target', ['i32', 'S'])).
fn((t) => {
  const wgsl = `
struct S {
  a : ${t.params.target}
}`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion').
desc('Test that indirect recursion of structures is rejected').
params((u) => u.combine('target', ['i32', 'S'])).
fn((t) => {
  const wgsl = `
struct S {
  a : T
}
struct T {
  a : ${t.params.target}
}`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_array_element').
desc('Test that indirect recursion of structures via array element types is rejected').
params((u) => u.combine('target', ['i32', 'S'])).
fn((t) => {
  const wgsl = `
struct S {
  a : array<${t.params.target}, 4>
}
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});

g.test('no_indirect_recursion_via_array_size').
desc('Test that indirect recursion of structures via array size expressions is rejected').
params((u) => u.combine('target', ['S1', 'S2'])).
fn((t) => {
  const wgsl = `
struct S1 {
  a : i32,
}
struct S2 {
  a : i32,
  b : array<i32, ${t.params.target}().a + 1>,
}
`;
  t.expectCompileResult(t.params.target === 'S1', wgsl);
});

g.test('no_indirect_recursion_via_struct_attribute').
desc('Test that indirect recursion of structures via struct members is rejected').
params((u) =>
u //
.combine('target', ['S1', 'S2']).
combine('attribute', ['align', 'location', 'size'])
).
fn((t) => {
  const wgsl = `
struct S1 {
  a : i32
}
struct S2 {
  @${t.params.attribute}(${t.params.target}(4).a) a : i32
}
`;
  t.expectCompileResult(t.params.target === 'S1', wgsl);
});

g.test('no_indirect_recursion_via_struct_member_nested_in_alias').
desc(
  `Test that indirect recursion of structures via struct members is rejected when the member type
    is an alias that contains the structure`
).
params((u) => u.combine('target', ['i32', 'A'])).
fn((t) => {
  const wgsl = `
alias A = array<S2, 4>;
struct S1 {
  a : ${t.params.target}
}
struct S2 {
  a : S1
}
`;
  t.expectCompileResult(t.params.target === 'i32', wgsl);
});







const kStructureCases = {
  bool: {
    code: `struct S { x : bool }`,
    valid: true
  },
  u32: {
    code: `struct S { x : u32 }`,
    valid: true
  },
  i32: {
    code: `struct S { x : i32 }`,
    valid: true
  },
  f32: {
    code: `struct S { x : f32 }`,
    valid: true
  },
  f16: {
    code: `struct S { x : f16 }`,
    valid: true,
    f16: true
  },

  vec2u: {
    code: `struct S { x : vec2u }`,
    valid: true
  },
  vec3i: {
    code: `struct S { x : vec3i }`,
    valid: true
  },
  vec4f: {
    code: `struct S { x : vec4f }`,
    valid: true
  },
  vec4h: {
    code: `struct S { x : vec4h }`,
    valid: true,
    f16: true
  },

  mat2x2f: {
    code: `struct S { x : mat2x2f }`,
    valid: true
  },
  mat3x4h: {
    code: `struct S { x : mat3x4h }`,
    valid: true,
    f16: true
  },

  atomic_u32: {
    code: `struct S { x : atomic<u32> }`,
    valid: true
  },
  atomic_i32: {
    code: `struct S { x : atomic<i32> }`,
    valid: true
  },

  array_u32_4: {
    code: `struct S { x : array<u32, 4> }`,
    valid: true
  },
  array_u32: {
    code: `struct S { x : array<u32> }`,
    valid: true
  },
  array_u32_not_last: {
    code: `struct S { x : array<u32>, y : u32 }`,
    valid: false
  },
  array_u32_override: {
    code: `override o : u32;
    struct S { x : array<u32, o> }`,
    valid: false
  },

  structure: {
    code: `struct S { x : u32 }
    struct T { x : S }`,
    valid: true
  },
  structure_structure_rta: {
    code: `struct S { x : array<u32> }
    struct T { x : S }`,
    valid: false
  },

  pointer: {
    code: `struct S { x : ptr<function, u32> }`,
    valid: false
  },

  texture: {
    code: `struct S { x : texture_2d<f32> }`,
    valid: false
  },
  sampler: {
    code: `struct S { x : sampler }`,
    valid: false
  },
  sampler_comparison: {
    code: `struct S { x : sampler_comparison }`,
    valid: false
  },

  many_members: {
    code: `struct S {
      m1 : u32,
      m2 : i32,
      m3 : vec4f,
      m4 : array<u32, 8>,
      m5 : array<f32>
    }`,
    valid: true
  },

  trailing_comma: {
    code: `struct S { x : u32, }`,
    valid: true
  },

  empty: {
    code: `struct S { }`,
    valid: false
  },

  name_collision1: {
    code: `struct S { x : u32 }
    struct S { x : u32 }`,
    valid: false
  },
  name_collision2: {
    code: `fn S() { }
    struct S { x : u32 }`,
    valid: false
  },
  name_collision3: {
    code: `struct S { x : u32 }
    alias S = u32;`,
    valid: false
  },
  member_collision: {
    code: `struct S { x : u32, x : u32 }`,
    valid: false
  },
  no_name: {
    code: `struct { x : u32 }`,
    valid: false
  },
  missing_l_brace: {
    code: `struct S x : u32 }`,
    valid: false
  },
  missing_r_brace: {
    code: `struct S { x : u32`,
    valid: false
  },
  bad_name: {
    code: `struct 123 { x : u32 }`,
    valid: false
  },
  bad_delimiter: {
    code: `struct S { x : u32; y : u32 }`,
    valid: false
  },
  missing_delimiter: {
    code: `struct S { x : u32 y : u32 }`,
    valid: false
  },
  bad_member_decl: {
    code: `struct S { x u32 }`,
    valid: false
  }
};

g.test('structures').
desc('Validation tests for structures').
params((u) => u.combine('case', keysOf(kStructureCases))).
beforeAllSubcases((t) => {
  const testcase = kStructureCases[t.params.case];
  if (testcase.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kStructureCases[t.params.case];
  const code = `${testcase.f16 ? 'enable f16;' : ''}
    ${testcase.code}`;
  t.expectCompileResult(testcase.valid, code);
});