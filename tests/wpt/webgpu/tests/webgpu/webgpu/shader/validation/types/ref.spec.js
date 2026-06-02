/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for ref types
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTypes = ['bool', 'i32', 'f32', 'vec2i', 'mat2x2f', 'array<i32, 4>', 'S'];

g.test('not_typeable_var').
desc('Test that `ref` cannot be typed in a shader as an explicit var decl type.').
params((u) => u.combine('type', kTypes).combine('ref', [false, true])).
fn((t) => {
  let ty = t.params.type;
  if (t.params.ref) {
    ty = `ref<private, ${ty}>`;
  }
  const code = `
    struct S { a : u32 }
    var<private> foo : ${ty};`;
  t.expectCompileResult(!t.params.ref, code);
});

g.test('not_typeable_let').
desc('Test that `ref` cannot be typed in a shader as a let decl type.').
params((u) => u.combine('type', kTypes).combine('view', ['ptr', 'ref'])).
fn((t) => {
  const code = `
      struct S { a : u32 }
      fn foo() {
        var a : ${t.params.type};
        let b : ${t.params.view}<function, ${t.params.type}> = &a;
      }`;
  t.expectCompileResult(t.params.view === 'ptr', code);
});

g.test('not_typeable_param').
desc('Test that `ref` cannot be typed in a shader as a function parameter type.').
params((u) => u.combine('type', kTypes).combine('view', ['ptr', 'ref'])).
fn((t) => {
  const code = `
    struct S { a : u32 }
    fn foo(a : ${t.params.view}<private, ${t.params.type}>) {}`;
  t.expectCompileResult(t.params.view === 'ptr', code);
});

g.test('not_typeable_alias').
desc('Test that `ref` cannot be typed in a shader as an alias type.').
params((u) => u.combine('type', kTypes).combine('view', ['ptr', 'ref'])).
fn((t) => {
  const code = `
    struct S { a : u32 }
    alias a = ${t.params.view}<private, ${t.params.type}>;`;
  t.expectCompileResult(t.params.view === 'ptr', code);
});