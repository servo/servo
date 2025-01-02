/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for let declarations
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);







const kTypeCases = {
  bool: {
    code: `let x : bool = true;`,
    valid: true
  },
  i32: {
    code: `let x : i32 = 1i;`,
    valid: true
  },
  u32: {
    code: `let x : u32 = 1u;`,
    valid: true
  },
  f32: {
    code: `let x : f32 = 1f;`,
    valid: true
  },
  f16: {
    code: `let x : f16 = 1h;`,
    valid: true
  },
  vec2i: {
    code: `let x : vec2i = vec2i();`,
    valid: true
  },
  vec3u: {
    code: `let x : vec3u = vec3u();`,
    valid: true
  },
  vec4f: {
    code: `let x : vec4f = vec4f();`,
    valid: true
  },
  mat2x2: {
    code: `let x : mat2x2f = mat2x2f();`,
    valid: true
  },
  mat4x3f: {
    code: `let x : mat4x3<f32> = mat4x3<f32>();`,
    valid: true
  },
  array_sized: {
    code: `let x : array<u32, 4> = array(1,2,3,4);`,
    valid: true
  },
  array_runtime: {
    code: `let x : array<u32> = array(1,2,3);`,
    valid: false
  },
  struct: {
    code: `let x : S = S(0);`,
    valid: true,
    decls: `struct S { x : u32 }`
  },
  atomic: {
    code: `let x : atomic<u32> = 0;`,
    valid: false
  },
  ptr_function: {
    code: `
      var x : i32;
      let y : ptr<function, i32> = &x;`,
    valid: true
  },
  ptr_storage: {
    code: `let y : ptr<storage, i32> = &x[0];`,
    valid: true,
    decls: `@group(0) @binding(0) var<storage> x : array<i32, 4>;`
  },
  load_rule: {
    code: `
      var x : i32 = 1;
      let y : i32 = x;`,
    valid: true
  }
};

g.test('type').
desc('Test let types').
params((u) => u.combine('case', keysOf(kTypeCases))).
beforeAllSubcases((t) => {
  if (t.params.case === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kTypeCases[t.params.case];
  const code = `
${t.params.case === 'f16' ? 'enable f16;' : ''}
${testcase.decls ?? ''}
fn foo() {
  ${testcase.code}
}`;
  const expect = testcase.valid;
  t.expectCompileResult(expect, code);
});

const kInitCases = {
  no_init: {
    code: `let x : u32;`,
    valid: false
  },
  no_type: {
    code: `let x = 1;`,
    valid: true
  },
  init_matching_type: {
    code: `let x : u32 = 1u;`,
    valid: true
  },
  init_mismatch_type: {
    code: `let x : u32 = 1i;`,
    valid: false
  },
  ptr_type_mismatch: {
    code: `var x : i32;\nlet y : ptr<function, u32> = &x;`,
    valid: false
  },
  ptr_access_mismatch: {
    code: `let y : ptr<storage, u32, read> = &x;`,
    valid: false,
    decls: `@group(0) @binding(0) var<storage, read_write> x : u32;`
  },
  ptr_addrspace_mismatch: {
    code: `let y = ptr<storage, u32> = &x;`,
    valid: false,
    decls: `@group(0) @binding(0) var<uniform> x : u32;`
  },
  init_const_expr: {
    code: `let y = x * 2;`,
    valid: true,
    decls: `const x = 1;`
  },
  init_override_expr: {
    code: `let y = x + 1;`,
    valid: true,
    decls: `override x = 1;`
  },
  init_runtime_expr: {
    code: `var x = 1;\nlet y = x << 1;`,
    valid: true
  }
};

g.test('initializer').
desc('Test let initializers').
params((u) => u.combine('case', keysOf(kInitCases))).
fn((t) => {
  const testcase = kInitCases[t.params.case];
  const code = `
${testcase.decls ?? ''}
fn foo() {
  ${testcase.code}
}`;
  const expect = testcase.valid;
  t.expectCompileResult(expect, code);
});

g.test('module_scope').
desc('Test that let declarations are disallowed module scope').
fn((t) => {
  const code = `let x = 0;`;
  t.expectCompileResult(false, code);
});

const kTestTypes = [
'f32',
'i32',
'u32',
'bool',
'vec2<f32>',
'vec2<i32>',
'vec2<u32>',
'vec2<bool>',
'vec3<f32>',
'vec3<i32>',
'vec3<u32>',
'vec3<bool>',
'vec4<f32>',
'vec4<i32>',
'vec4<u32>',
'vec4<bool>',
'mat2x2<f32>',
'mat2x3<f32>',
'mat2x4<f32>',
'mat3x2<f32>',
'mat3x3<f32>',
'mat3x4<f32>',
'mat4x2<f32>',
'mat4x3<f32>',
'mat4x4<f32>',
// [1]: 12 is a random number here. find a solution to replace it.
'array<f32, 12>',
'array<i32, 12>',
'array<u32, 12>',
'array<bool, 12>'];


g.test('initializer_type').
desc(
  `
  If present, the initializer's type must match the store type of the variable.
  Testing scalars, vectors, and matrices of every dimension and type.
  TODO: add test for: structs - arrays of vectors and matrices - arrays of different length
`
).
params((u) => u.beginSubcases().combine('lhsType', kTestTypes).combine('rhsType', kTestTypes)).
fn((t) => {
  const { lhsType, rhsType } = t.params;

  const code = `
      @fragment
      fn main() {
        let a : ${lhsType} = ${rhsType}();
      }
    `;

  const expectation = lhsType === rhsType;
  t.expectCompileResult(expectation, code);
});