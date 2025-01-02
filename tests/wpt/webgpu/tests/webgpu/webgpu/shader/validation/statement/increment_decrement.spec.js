/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation for phony assignment statements`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { scalarTypeOf, Type } from '../../../util/conversion.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);








const kConstructibleTypes = [
'bool',
'i32',
'u32',
'f32',
'f16',
'vec2f',
'vec3h',
'vec4u',
'vec3b',
'mat2x3f',
'mat4x2h',
'abstractInt',
'abstractFloat'];


const kConstructibleCases = {
  ...kConstructibleTypes.reduce(
    (acc, t) => ({
      ...acc,
      [t]: {
        value: Type[t].create(1).wgsl(),
        pass: t === 'i32' || t === 'u32' || t === 'abstractInt',
        usesF16: scalarTypeOf(Type[t]).kind === 'f16'
      }
    }),
    {}
  ),
  array: { value: 'array(1,2,3)', pass: false },
  struct: { value: 'S(1,2)', pass: false, gdecl: 'struct S{ a:u32, b:u32}' },
  atomic_u32: { value: 'xu', pass: false, gdecl: 'var<workgroup> xu: atomic<u32>;' },
  atomic_i32: { value: 'xi', pass: false, gdecl: 'var<workgroup> xi: atomic<i32>;' }
};

g.test('var_init_type').
desc(`Test increment and decrement of a variable of various types`).
params((u) => u.combine('type', keysOf(kConstructibleCases)).combine('direction', ['up', 'down'])).
beforeAllSubcases((t) => {
  const c = kConstructibleCases[t.params.type];
  if (c.usesF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const { value, pass, usesF16, gdecl } = kConstructibleCases[t.params.type];
  const operator = t.params.direction === 'up' ? '++' : '--';
  const code = `
${usesF16 ? 'enable f16;' : ''}
${gdecl ?? ''}
fn f() {
  var a = ${value};
  a${operator};
}`;
  t.expectCompileResult(pass, code);
});









const kComponentCases = {
  v2u_x: { type: 'vec2u', wgsl: 'a.x', pass: true },
  v2u_y: { type: 'vec2u', wgsl: 'a.y', pass: true },
  v3u_x: { type: 'vec3u', wgsl: 'a.x', pass: true },
  v3u_y: { type: 'vec3u', wgsl: 'a.y', pass: true },
  v3u_z: { type: 'vec3u', wgsl: 'a.z', pass: true },
  v4u_x: { type: 'vec4u', wgsl: 'a.x', pass: true },
  v4u_y: { type: 'vec4u', wgsl: 'a.y', pass: true },
  v4u_z: { type: 'vec4u', wgsl: 'a.z', pass: true },
  v4u_w: { type: 'vec4u', wgsl: 'a.w', pass: true },
  v2i_x: { type: 'vec2i', wgsl: 'a.x', pass: true },
  v2i_y: { type: 'vec2i', wgsl: 'a.y', pass: true },
  v3i_x: { type: 'vec3i', wgsl: 'a.x', pass: true },
  v3i_y: { type: 'vec3i', wgsl: 'a.y', pass: true },
  v3i_z: { type: 'vec3i', wgsl: 'a.z', pass: true },
  v4i_x: { type: 'vec4i', wgsl: 'a.x', pass: true },
  v4i_y: { type: 'vec4i', wgsl: 'a.y', pass: true },
  v4i_z: { type: 'vec4i', wgsl: 'a.z', pass: true },
  v4i_w: { type: 'vec4i', wgsl: 'a.w', pass: true },
  v2u_xx: { type: 'vec2u', wgsl: 'a.xx', pass: false },
  v2u_indexed: { type: 'vec2u', wgsl: 'a[0]', pass: true },
  v2f_x: { type: 'vec2f', wgsl: 'a.x', pass: false },
  v2h_x: { type: 'vec2h', wgsl: 'a.x', pass: false, usesF16: true },
  mat2x2f: { type: 'mat2x2f', wgsl: 'a[0][0]', pass: false },
  mat2x2h: { type: 'mat2x2h', wgsl: 'a[0][0]', pass: false, usesF16: true },
  array: { type: 'array<i32,2>', wgsl: 'a', pass: false },
  array_i: { type: 'array<i32,2>', wgsl: 'a[0]', pass: true },
  array_f: { type: 'array<f32,2>', wgsl: 'a[0]', pass: false },
  struct: { type: 'S', wgsl: 'S', pass: false, gdecl: 'struct S{field:i32}' },
  struct_var: { type: 'S', wgsl: 'a', pass: false, gdecl: 'struct S{field:i32}' },
  struct_field: { type: 'S', wgsl: 'a.field', pass: true, gdecl: 'struct S{field:i32}' }
};

g.test('component').
desc('Test increment and decrement of component of various types').
params((u) => u.combine('type', keysOf(kComponentCases)).combine('direction', ['up', 'down'])).
beforeAllSubcases((t) => {
  const c = kComponentCases[t.params.type];
  if (c.usesF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const { type, wgsl, pass, usesF16, gdecl } = kComponentCases[t.params.type];
  const operator = t.params.direction === 'up' ? '++' : '--';
  const code = `
${usesF16 ? 'enable f16;' : ''}
${gdecl ?? ''}
fn f() {
  var a: ${type};
  ${wgsl}${operator};
}`;
  t.expectCompileResult(pass, code);
});










const kTests = {
  var: { wgsl: 'a++;', pass: true },
  vector: { wgsl: 'v++;', pass: false },
  paren_var_paren: { wgsl: '(a)++;', pass: true },
  star_and_var: { wgsl: '*&a++;', pass: true },
  paren_star_and_var_paren: { wgsl: '(*&a)++;', pass: true },
  many_star_and_var: { wgsl: '*&*&*&a++;', pass: true },

  space: { wgsl: 'a ++;', pass: true },
  tab: { wgsl: 'a\t++;', pass: true },
  newline: { wgsl: 'a\n++;', pass: true },
  cr: { wgsl: 'a\r++;', pass: true },
  space_space: { wgsl: 'a ++ ;', pass: true },
  plus_space_plus: { wgsl: 'a+ +;', pass: false },
  minux_space_minus: { wgsl: 'a- -;', pass: false },

  no_var: { wgsl: '++;', pass: false },
  no_semi: { wgsl: 'a++', pass: false },
  prefix: { wgsl: '++a;', pass: false },

  postfix_x: { wgsl: 'v++.x;', pass: false },
  postfix_r: { wgsl: 'v++.r;', pass: false },
  postfix_index: { wgsl: 'v++[0];', pass: false },
  postfix_field: { wgsl: 'a++.foo;', pass: false },

  literal_i32: { wgsl: '12i++;', pass: false },
  literal_u32: { wgsl: '12u++;', pass: false },
  literal_abstract_int: { wgsl: '12++;', pass: false },
  literal_abstract_float: { wgsl: '12.0++;', pass: false },
  literal_f32: { wgsl: '12.0f++;', pass: false },

  assign_to: { wgsl: 'a++ = 1;', pass: false },

  at_global: { wgsl: '', pass: false, gdecl: 'var<private> g:i32; g++;' },
  private: { wgsl: 'g++;', pass: true, gdecl: 'var<private> g:i32;' },
  workgroup: { wgsl: 'g++;', pass: true, gdecl: 'var<workgroup> g:i32;' },
  storage_rw: {
    wgsl: 'g++;',
    pass: true,
    gdecl: '@group(0) @binding(0) var<storage,read_write> g: i32;'
  },
  storage_r: {
    wgsl: 'g++;',
    pass: false,
    gdecl: '@group(0) @binding(0) var<storage,read> g: i32;'
  },
  storage: { wgsl: 'g++;', pass: false, gdecl: '@group(0) @binding(0) var<storage,read> g: i32;' },
  uniform: { wgsl: 'g++;', pass: false, gdecl: '@group(0) @binding(0) var<uniform> g: i32;' },
  texture: { wgsl: 'g++;', pass: false, gdecl: '@group(0) @binding(0) var g: texture_2d<u32>;' },
  texture_x: {
    wgsl: 'g.x++;',
    pass: false,
    gdecl: '@group(0) @binding(0) var g: texture_2d<u32>;'
  },
  texture_storage: {
    wgsl: 'g++;',
    pass: false,
    gdecl: '@group(0) @binding(0) var g: texture_storage_2d<r32uint>;'
  },
  texture_storage_x: {
    wgsl: 'g.x++;',
    pass: false,
    gdecl: '@group(0) @binding(0) var g: texture_storage_2d<r32uint>;'
  },
  sampler: { wgsl: 'g++;', pass: false, gdecl: '@group(0) @binding(0) var g: sampler;' },
  sampler_comparison: {
    wgsl: 'g++;',
    pass: false,
    gdecl: '@group(0) @binding(0) var g: sampler_comparison;'
  },
  override: { wgsl: 'g++;', pass: false, gdecl: 'override g:i32;' },
  global_const: { wgsl: 'g++;', pass: false, gdecl: 'const g:i32 = 0;' },
  workgroup_atomic: { wgsl: 'g++;', pass: false, gdecl: 'var<workgroup> g:atomic<i32>;' },
  storage_atomic: {
    wgsl: 'g++;',
    pass: false,
    gdecl: '@group(0) @binding(0) var<storage,read_write> g:atomic<u32>;'
  },

  subexpr: { wgsl: 'a = b++;', pass: false },
  expr_paren: { wgsl: '(a++);', pass: false },
  expr_add: { wgsl: '0 + a++;', pass: false },
  expr_negate: { wgsl: '-a++;', pass: false },
  inc_inc: { wgsl: 'a++++;', pass: false },
  inc_space_inc: { wgsl: 'a++ ++;', pass: false },
  inc_dec: { wgsl: 'a++--;', pass: false },
  inc_space_dec: { wgsl: 'a++ --;', pass: false },
  paren_inc: { wgsl: '(a++)++;', pass: false },
  paren_dec: { wgsl: '(a++)--;', pass: false },

  in_block: { wgsl: '{ a++; }', pass: true },
  in_for_init: { wgsl: 'for (a++;false;) {}', pass: true },
  in_for_cond: { wgsl: 'for (;a++;) {}', pass: false },
  in_for_update: { wgsl: 'for (;false;a++) {}', pass: true },
  in_for_update_semi: { wgsl: 'for (;false;a++;) {}', pass: false },
  in_continuing: { wgsl: 'loop { continuing { a++; break if true;}}', pass: true },

  let: { wgsl: 'let c = a; c++;', pass: false },
  const: { wgsl: 'const c = 1; c++;', pass: false },
  builtin: { wgsl: 'max++', pass: false },
  enum: { wgsl: 'r32uint++', pass: false },
  param: { wgsl: '', pass: false, gdecl: 'fn bump(p: i32) { p++;}' }
};

g.test('parse').
desc(`Test that increment and decrement statements are parsed correctly.`).
params((u) => u.combine('test', keysOf(kTests)).combine('direction', ['up', 'down'])).
fn((t) => {
  const c = kTests[t.params.test];
  let { wgsl, gdecl } = c;
  gdecl = gdecl ?? '';
  if (t.params.direction === 'down') {
    wgsl = wgsl.replace('++', '--');
    gdecl = gdecl.replace('++', '--');
  }

  const code = `
${gdecl}
fn f() {
  var a: u32;
  var b: u32;
  var v: vec4u;
  ${wgsl}
}`;
  t.expectCompileResult(c.pass, code);
});