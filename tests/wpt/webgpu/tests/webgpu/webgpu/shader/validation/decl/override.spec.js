/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for override declarations
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('no_direct_recursion').
desc('Test that direct recursion of override declarations is rejected').
params((u) => u.combine('target', ['a', 'b'])).
fn((t) => {
  const wgsl = `
override a : i32 = 42;
override b : i32 = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'a', wgsl);
});

g.test('no_indirect_recursion').
desc('Test that indirect recursion of override declarations is rejected').
params((u) => u.combine('target', ['a', 'b'])).
fn((t) => {
  const wgsl = `
override a : i32 = 42;
override b : i32 = c;
override c : i32 = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'a', wgsl);
});

const kIdCases = {
  min: {
    code: `@id(0) override x = 1;`,
    valid: true
  },
  max: {
    code: `@id(65535) override x = 1;`,
    valid: true
  },
  neg: {
    code: `@id(-1) override x = 1;`,
    valid: false
  },
  too_large: {
    code: `@id(65536) override x = 1;`,
    valid: false
  },
  duplicate: {
    code: `
      @id(1) override x = 1;
      @id(1) override y = 1;`,
    valid: false
  }
};

g.test('id').
desc('Test id attributes').
params((u) => u.combine('case', keysOf(kIdCases))).
fn((t) => {
  const testcase = kIdCases[t.params.case];
  const code = testcase.code;
  const expect = testcase.valid;
  t.expectCompileResult(expect, code);
});

const kTypeCases = {
  bool: {
    code: `override x : bool;`,
    valid: true
  },
  i32: {
    code: `override x : i32;`,
    valid: true
  },
  u32: {
    code: `override x : u32;`,
    valid: true
  },
  f32: {
    code: `override x : f32;`,
    valid: true
  },
  f16: {
    code: `enable f16;\noverride x : f16;`,
    valid: true
  },
  abs_int_conversion: {
    code: `override x = 1;`,
    valid: true
  },
  abs_float_conversion: {
    code: `override x = 1.0;`,
    valid: true
  },
  vec2_bool: {
    code: `override x : vec2<bool>;`,
    valid: false
  },
  vec2i: {
    code: `override x : vec2i;`,
    valid: false
  },
  vec3u: {
    code: `override x : vec3u;`,
    valid: false
  },
  vec4f: {
    code: `override x : vec4f;`,
    valid: false
  },
  mat2x2f: {
    code: `override x : mat2x2f;`,
    valid: false
  },
  matrix: {
    code: `override x : mat4x3<f32>;`,
    valid: false
  },
  array: {
    code: `override x : array<u32, 4>;`,
    valid: false
  },
  struct: {
    code: `struct S { x : u32 }\noverride x : S;`,
    valid: false
  },
  atomic: {
    code: `override x : atomic<u32>;`,
    valid: false
  }
};

g.test('type').
desc('Test override types').
params((u) => u.combine('case', keysOf(kTypeCases))).
beforeAllSubcases((t) => {
  if (t.params.case === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kTypeCases[t.params.case];
  const code = testcase.code;
  const expect = testcase.valid;
  t.expectCompileResult(expect, code);
});

const kInitCases = {
  no_init_no_type: {
    code: `override x;`,
    valid: false
  },
  no_init: {
    code: `override x : u32;`,
    valid: true
  },
  no_type: {
    code: `override x = 1;`,
    valid: true
  },
  init_matching_type: {
    code: `override x : u32 = 1u;`,
    valid: true
  },
  init_mismatch_type: {
    code: `override x : u32 = 1i;`,
    valid: false
  },
  init_mismatch_vector: {
    code: `override x : u32 = vec2i();`,
    valid: false
  },
  abs_int_init_convert: {
    code: `override x : f32 = 1;`,
    valid: true
  },
  abs_float_init_convert: {
    code: `override x : f32 = 1.0;`,
    valid: true
  },
  init_const_expr: {
    code: `const x = 1;\noverride y = 2 * x;`,
    valid: true
  },
  init_override_expr: {
    code: `override x = 1;\noverride y = x + 2;`,
    valid: true
  },
  init_runtime_expr: {
    code: `var<private> x = 2;\noverride y = x;`,
    valid: false
  },
  const_func_init: {
    code: `override x = max(1, 2);`,
    valid: true
  },
  non_const_func_init: {
    code: `override x = foo(1);
    fn foo(p : i32) -> i32 { return p; }`,
    valid: false
  },
  mix_order_init: {
    code: `override x = y;
    override y : i32;`,
    valid: true
  }
};

g.test('initializer').
desc('Test override initializers').
params((u) => u.combine('case', keysOf(kInitCases))).
fn((t) => {
  const testcase = kInitCases[t.params.case];
  const code = testcase.code;
  const expect = testcase.valid;
  t.expectCompileResult(expect, code);
});

g.test('function_scope').
desc('Test that override declarations are disallowed in functions').
fn((t) => {
  const code = `fn foo() { override x : u32; }`;
  t.expectCompileResult(false, code);
});