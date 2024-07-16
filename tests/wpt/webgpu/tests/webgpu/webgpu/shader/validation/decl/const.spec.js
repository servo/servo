/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for const declarations
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('no_direct_recursion').
desc('Test that direct recursion of const declarations is rejected').
params((u) => u.combine('target', ['a', 'b'])).
fn((t) => {
  const wgsl = `
const a : i32 = 42;
const b : i32 = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'a', wgsl);
});

g.test('no_indirect_recursion').
desc('Test that indirect recursion of const declarations is rejected').
params((u) => u.combine('target', ['a', 'b'])).
fn((t) => {
  const wgsl = `
const a : i32 = 42;
const b : i32 = c;
const c : i32 = ${t.params.target};
`;
  t.expectCompileResult(t.params.target === 'a', wgsl);
});

g.test('no_indirect_recursion_via_array_size').
desc('Test that indirect recursion of const declarations via array size expressions is rejected').
params((u) => u.combine('target', ['a', 'b'])).
fn((t) => {
  const wgsl = `
const a = 4;
const b = c[0];
const c = array<i32, ${t.params.target}>(4, 4, 4, 4);
`;
  t.expectCompileResult(t.params.target === 'a', wgsl);
});

g.test('no_indirect_recursion_via_struct_attribute').
desc('Test that indirect recursion of const declarations via struct members is rejected').
params((u) =>
u //
.combine('target', ['a', 'b']).
combine('attribute', ['align', 'location', 'size'])
).
fn((t) => {
  const wgsl = `
struct S {
  @${t.params.attribute}(${t.params.target}) a : i32
}
const a = 4;
const b = S(4).a;
`;
  t.expectCompileResult(t.params.target === 'a', wgsl);
});

const kTypeCases = {
  bool: {
    code: `const x : bool = true;`,
    valid: true
  },
  i32: {
    code: `const x : i32 = 1i;`,
    valid: true
  },
  u32: {
    code: `const x : u32 = 1u;`,
    valid: true
  },
  f32: {
    code: `const x : f32 = 1f;`,
    valid: true
  },
  f16: {
    code: `enable f16;\nconst x : f16 = 1h;`,
    valid: true
  },
  abstract_int: {
    code: `
      const x = 0xffffffffff;
      const_assert x == 0xffffffffff;`,
    valid: true
  },
  abstract_float: {
    code: `
      const x = 3937509.87755102;
      const_assert x != 3937510.0;
      const_assert x != 3937509.75;`,
    valid: true
  },
  vec2i: {
    code: `const x : vec2i = vec2i();`,
    valid: true
  },
  vec3u: {
    code: `const x : vec3u = vec3u();`,
    valid: true
  },
  vec4f: {
    code: `const x : vec4f = vec4f();`,
    valid: true
  },
  mat2x2: {
    code: `const x : mat2x2f = mat2x2f();`,
    valid: true
  },
  mat4x3f: {
    code: `const x : mat4x3<f32> = mat4x3<f32>();`,
    valid: true
  },
  array_sized: {
    code: `const x : array<u32, 4> = array(1,2,3,4);`,
    valid: true
  },
  array_runtime: {
    code: `const x : array<u32> = array(1,2,3);`,
    valid: false
  },
  struct: {
    code: `struct S { x : u32 }\nconst x : S = S(0);`,
    valid: true
  },
  atomic: {
    code: `const x : atomic<u32> = 0;`,
    valid: false
  },
  vec_abstract_int: {
    code: `
      const x = vec2(0xffffffffff,0xfffffffff0);
      const_assert x.x == 0xffffffffff;
      const_assert x.y == 0xfffffffff0;`,
    valid: true
  },
  array_abstract_int: {
    code: `
      const x = array(0xffffffffff,0xfffffffff0);
      const_assert x[0] == 0xffffffffff;
      const_assert x[1] == 0xfffffffff0;`,
    valid: true
  }
};

g.test('type').
desc('Test const types').
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
  no_init: {
    code: `const x : u32;`,
    valid: false
  },
  no_type: {
    code: `const x = 0;`,
    valid: true
  },
  no_init_no_type: {
    code: `const x;`,
    valid: false
  },
  init_matching_type: {
    code: `const x : i32 = 1i;`,
    valid: true
  },
  init_mismatch_type: {
    code: `const x : u32 = 1i;`,
    valid: false
  },
  abs_int_init_convert: {
    code: `const x : u32 = 1;`,
    valid: true
  },
  abs_float_init_convert: {
    code: `const x : f32 = 1.0;`,
    valid: true
  },
  init_const_expr: {
    code: `const x = 0;\nconst y = x + 2;`,
    valid: true
  },
  init_override_expr: {
    code: `override x : u32;\nconst y = x * 2;`,
    valid: false
  },
  init_runtime_expr: {
    code: `var<private> x = 1i;\nconst y = x - 1;`,
    valid: false
  },
  init_func: {
    code: `const x = max(1,2);`,
    valid: true
  },
  init_non_const_func: {
    code: `const x = foo(1);
    fn foo(p : i32) -> i32 { return p; }`,
    valid: false
  }
};

g.test('initializer').
desc('Test const initializers').
params((u) => u.combine('case', keysOf(kInitCases))).
fn((t) => {
  const testcase = kInitCases[t.params.case];
  const code = testcase.code;
  const expect = testcase.valid;
  t.expectCompileResult(expect, code);
});

g.test('function_scope').
desc('Test that const declarations are allowed in functions').
fn((t) => {
  const code = `fn foo() { const x = 0; }`;
  t.expectCompileResult(true, code);
});

g.test('immutable').
desc('Test that const declarations are immutable').
fn((t) => {
  const code = `
    const x = 0;
    fn foo() {
      x = 1;
    }`;
  t.expectCompileResult(false, code);
});

g.test('assert').
desc('Test value can be checked by a const_assert').
fn((t) => {
  const code = `
    const x = 0;
    const_assert x == 0;`;
  t.expectCompileResult(true, code);
});

g.test('placement').
desc('Tests @const is not allowed to appear').
params((u) =>
u.combine('scope', [
'private-var',
'storage-var',
'struct-member',
'fn-decl',
'fn-param',
'fn-var',
'fn-return',
'while-stmt',
undefined]
)
).
fn((t) => {
  const scope = t.params.scope;

  const attr = '@const';
  const code = `
      ${scope === 'private-var' ? attr : ''}
      var<private> priv_var : i32;

      ${scope === 'storage-var' ? attr : ''}
      @group(0) @binding(0)
      var<storage> stor_var : i32;

      struct A {
        ${scope === 'struct-member' ? attr : ''}
        a : i32,
      }

      @vertex
      ${scope === 'fn-decl' ? attr : ''}
      fn f(
        ${scope === 'fn-param' ? attr : ''}
        @location(0) b : i32,
      ) -> ${scope === 'fn-return' ? attr : ''} @builtin(position) vec4f {
        ${scope === 'fn-var' ? attr : ''}
        var<function> func_v : i32;

        ${scope === 'while-stmt' ? attr : ''}
        while false {}

        return vec4(1, 1, 1, 1);
      }
    `;

  t.expectCompileResult(scope === undefined, code);
});