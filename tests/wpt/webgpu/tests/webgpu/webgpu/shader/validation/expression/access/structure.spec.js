/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for structure access expressions.

* Correct result type
* Identifier matching
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('identifier_mismatch').
desc('Tests that the member identifier must match a member in the declaration').
params((u) => u.combine('decl', ['value', 'ref'])).
fn((t) => {
  const code = `
    struct S {
      x : u32
    }
    fn foo() {
      ${t.params.decl === 'value' ? 'let' : 'var'} v : S = S();
      _ = v.y;
    }`;
  t.expectCompileResult(false, code);
});

g.test('shadowed_member').
desc('Tests that other declarations do not interfere with member determination').
params((u) => u.combine('decl', ['value', 'ref'])).
fn((t) => {
  const code = `
    struct S {
      x : u32
    }
    fn foo() {
      var x : i32;
      ${t.params.decl === 'value' ? 'let' : 'var'} v : S = S();
      let tmp : u32 = v.x;
    }`;
  t.expectCompileResult(true, code);
});

g.test('result_type').
desc('Tests correct result types are returned').
params((u) => u.combine('decl', ['value', 'ref'])).
fn((t) => {
  const types = [
  'i32',
  'u32',
  'f32',
  'bool',
  'array<u32, 4>',
  'array<T, 2>',
  'vec2f',
  'vec3u',
  'vec4i',
  'mat2x2f',
  'T'];

  let code = `
    struct T {
      a : f32
    }
    struct S {\n`;

  for (let i = 0; i < types.length; i++) {
    code += `m${i} : ${types[i]},\n`;
  }

  code += `}
    fn foo() {
      var x : i32;
      ${t.params.decl === 'value' ? 'let' : 'var'} v : S = S();\n`;

  for (let i = 0; i < types.length; i++) {
    code += `let tmp${i} : ${types[i]} = v.m${i};\n`;
  }

  code += `}`;
  t.expectCompileResult(true, code);
});

g.test('result_type_f16').
desc('Tests correct type is returned for f16').
params((u) => u.combine('decl', ['value', 'ref'])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  const code = `
    enable f16;
    struct S {
      x : f16
    }
    fn foo() {
      ${t.params.decl === 'value' ? 'let' : 'var'} v : S = S();
      let tmp : f16 = v.x;
    }`;
  t.expectCompileResult(true, code);
});

g.test('result_type_runtime_array').
desc('Tests correct type is returned for runtime arrays').
fn((t) => {
  const code = `
    struct S {
      x : array<u32>
    }
    @group(0) @binding(0) var<storage> v : S;
    fn foo() {
      let tmp : u32 = v.x[0];
      let tmp_ptr : ptr<storage, array<u32>> = &v.x;
    }`;
  t.expectCompileResult(true, code);
});