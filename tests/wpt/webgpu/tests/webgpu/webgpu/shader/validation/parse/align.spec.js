/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for @align`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  blank: {
    src: '',
    pass: true
  },
  one: {
    src: '@align(1)',
    pass: true
  },
  four_a: {
    src: '@align(4)',
    pass: true
  },
  four_i: {
    src: '@align(4i)',
    pass: true
  },
  four_u: {
    src: '@align(4u)',
    pass: true
  },
  four_hex: {
    src: '@align(0x4)',
    pass: true
  },
  trailing_comma: {
    src: '@align(4,)',
    pass: true
  },
  const_u: {
    src: '@align(u_val)',
    pass: true
  },
  const_i: {
    src: '@align(i_val)',
    pass: true
  },
  const_expr: {
    src: '@align(i_val + 4 - 6)',
    pass: true
  },
  large: {
    src: '@align(1073741824)',
    pass: true
  },
  tabs: {
    src: '@\talign\t(4)',
    pass: true
  },
  comment: {
    src: '@/*comment*/align/*comment*/(4)',
    pass: true
  },
  misspelling: {
    src: '@malign(4)',
    pass: false
  },
  empty: {
    src: '@align()',
    pass: false
  },
  missing_left_paren: {
    src: '@align 4)',
    pass: false
  },
  missing_right_paren: {
    src: '@align(4',
    pass: false
  },
  multiple_values: {
    src: '@align(4, 2)',
    pass: false
  },
  non_power_two: {
    src: '@align(3)',
    pass: false
  },
  const_f: {
    src: '@align(f_val)',
    pass: false
  },
  one_f: {
    src: '@align(1.0)',
    pass: false
  },
  four_f: {
    src: '@align(4f)',
    pass: false
  },
  four_h: {
    src: '@align(4h)',
    pass: false
  },
  no_params: {
    src: '@align',
    pass: false
  },
  zero_a: {
    src: '@align(0)',
    pass: false
  },
  negative: {
    src: '@align(-4)',
    pass: false
  },
  large_no_power_two: {
    src: '@align(2147483646)',
    pass: false
  },
  larger_than_max_i32: {
    src: '@align(2147483648)',
    pass: false
  }
};

g.test('parsing').
desc(`Test that @align is parsed correctly.`).
params((u) => u.combine('align', keysOf(kTests))).
fn((t) => {
  const src = kTests[t.params.align].src;
  const code = `
const i_val: i32 = 4;
const u_val: u32 = 4;
const f_val: f32 = 4.2;
struct B {
  ${src} a: i32,
}

@group(0) @binding(0)
var<uniform> uniform_buffer: B;

@fragment
fn main() -> @location(0) vec4<f32> {
  return vec4<f32>(.4, .2, .3, .1);
}`;
  t.expectCompileResult(kTests[t.params.align].pass, code);
});

g.test('required_alignment').
desc('Test that the align with an invalid size is an error').
params((u) =>
u.
combine('address_space', ['storage', 'uniform']).
combine('align', [1, 2, 'alignment', 32]).
combine('type', [
{ name: 'i32', storage: 4, uniform: 4 },
{ name: 'u32', storage: 4, uniform: 4 },
{ name: 'f32', storage: 4, uniform: 4 },
{ name: 'f16', storage: 2, uniform: 2 },
{ name: 'atomic<i32>', storage: 4, uniform: 4 },
{ name: 'vec2<i32>', storage: 8, uniform: 8 },
{ name: 'vec2<f16>', storage: 4, uniform: 4 },
{ name: 'vec3<u32>', storage: 16, uniform: 16 },
{ name: 'vec3<f16>', storage: 8, uniform: 8 },
{ name: 'vec4<f32>', storage: 16, uniform: 16 },
{ name: 'vec4<f16>', storage: 8, uniform: 8 },
{ name: 'mat2x2<f32>', storage: 8, uniform: 8 },
{ name: 'mat3x2<f32>', storage: 8, uniform: 8 },
{ name: 'mat4x2<f32>', storage: 8, uniform: 8 },
{ name: 'mat2x2<f16>', storage: 4, uniform: 4 },
{ name: 'mat3x2<f16>', storage: 4, uniform: 4 },
{ name: 'mat4x2<f16>', storage: 4, uniform: 4 },
{ name: 'mat2x3<f32>', storage: 16, uniform: 16 },
{ name: 'mat3x3<f32>', storage: 16, uniform: 16 },
{ name: 'mat4x3<f32>', storage: 16, uniform: 16 },
{ name: 'mat2x3<f16>', storage: 8, uniform: 8 },
{ name: 'mat3x3<f16>', storage: 8, uniform: 8 },
{ name: 'mat4x3<f16>', storage: 8, uniform: 8 },
{ name: 'mat2x4<f32>', storage: 16, uniform: 16 },
{ name: 'mat3x4<f32>', storage: 16, uniform: 16 },
{ name: 'mat4x4<f32>', storage: 16, uniform: 16 },
{ name: 'mat2x4<f16>', storage: 8, uniform: 8 },
{ name: 'mat3x4<f16>', storage: 8, uniform: 8 },
{ name: 'mat4x4<f16>', storage: 8, uniform: 8 },
{ name: 'array<vec2<i32>, 2>', storage: 8, uniform: 16 },
{ name: 'array<vec4<i32>, 2>', storage: 8, uniform: 16 },
{ name: 'S', storage: 8, uniform: 16 }]
).
beginSubcases()
).
beforeAllSubcases((t) => {
  if (t.params.type.name.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  // While this would fail validation, it doesn't fail for any reasons related to alignment.
  // Atomics are not allowed in uniform address space as they have to be read_write.
  if (t.params.address_space === 'uniform' && t.params.type.name.startsWith('atomic')) {
    t.skip('No atomics in uniform address space');
  }

  let code = '';
  if (t.params.type.name.includes('f16')) {
    code += 'enable f16;\n';
  }

  // Testing the struct case, generate the structf
  if (t.params.type.name === 'S') {
    code += `struct S {
        a: mat4x2<f32>,          // Align 8
        b: array<vec${
    t.params.address_space === 'storage' ? 2 : 4
    }<i32>, 2>,  // Storage align 8, uniform 16
      }
      `;
  }

  let align = t.params.align;
  if (t.params.align === 'alignment') {
    // Alignment value listed in the spec
    if (t.params.address_space === 'storage') {
      align = `${t.params.type.storage}`;
    } else {
      align = `${t.params.type.uniform}`;
    }
  }

  let address_space = 'uniform';
  if (t.params.address_space === 'storage') {
    // atomics require read_write, not just the default of read
    address_space = 'storage, read_write';
  }

  code += `struct MyStruct {
      @align(${align}) a: ${t.params.type.name},
    }

    @group(0) @binding(0)
    var<${address_space}> a : MyStruct;`;

  code += `
    @fragment
    fn main() -> @location(0) vec4<f32> {
      return vec4<f32>(.4, .2, .3, .1);
    }`;

  // An array of `vec2` in uniform will not validate because, while the alignment on the array
  // itself is fine, the `vec2` element inside the array will have the wrong alignment. Uniform
  // requires that inner vec2 to have an align 16 which can only be done by specifying `vec4`
  // instead.
  const fails =
  t.params.address_space === 'uniform' && t.params.type.name.startsWith('array<vec2');

  t.expectCompileResult(!fails, code);
});

g.test('placement').
desc('Tests the locations @align is allowed to appear').
params((u) =>
u.
combine('scope', [
'private-var',
'storage-var',
'struct-member',
'fn-decl',
'fn-param',
'fn-var',
'fn-return',
'while-stmt',
undefined]
).
combine('attribute', [
{
  'private-var': false,
  'storage-var': false,
  'struct-member': true,
  'fn-decl': false,
  'fn-param': false,
  'fn-var': false,
  'fn-return': false,
  'while-stmt': false
}]
).
beginSubcases()
).
fn((t) => {
  const scope = t.params.scope;

  const attr = '@align(32)';
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

  t.expectCompileResult(scope === undefined || t.params.attribute[scope], code);
});

g.test('multi_align').
desc('Tests that align multiple times is an error').
params((u) => u.combine('multi', [true, false])).
fn((t) => {
  let code = `struct A {
      @align(128) `;

  if (t.params.multi === true) {
    code += '@align(128) ';
  }

  code += `a : i32,
      }

      @fragment
      fn main() -> @location(0) vec4<f32> {
        return vec4(1., 1., 1., 1.);
      }`;

  t.expectCompileResult(!t.params.multi, code);
});