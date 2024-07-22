/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for enumerant types.

* Values cannot be declared with the type
* Enumerant values cannot be used as values
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kEnumerantTypes = ['access_mode', 'address_space', 'texel_format'];

g.test('type_declaration').
desc('Tests that enumerants cannot be used as a type').
params((u) => u.combine('enum', kEnumerantTypes)).
fn((t) => {
  const code = `alias T = ${t.params.enum};`;
  t.expectCompileResult(false, code);
});

const kValueDecls = ['var', 'let', 'const', 'override'];

g.test('value_type').
desc('Tests that enumerant types cannot be the type of declaration').
params((u) => u.combine('enum', kEnumerantTypes).beginSubcases().combine('decl', kValueDecls)).
fn((t) => {
  const decl = `${t.params.decl} x : ${t.params.enum};`;
  let code = ``;
  if (t.params.decl === 'override') {
    code = `${decl}`;
  } else {
    code = `fn foo() {
        ${decl}
      }`;
  }
  t.expectCompileResult(false, code);
});

const kEnumerantValues = [
// Access modes
'read',
'write',
'read_write',

// Address spaces
'function',
'private',
'workgroup',
'storage',
'uniform',
'handle',

// Texel formats
'rgba8unorm',
'rgba8snorm',
'rgba8uint',
'rgba8sint',
'rgba16uint',
'rgba16sint',
'rgba16float',
'r32uint',
'r32sint',
'r32float',
'rg32uint',
'rg32sint',
'rg32float',
'rgba32uint',
'rgba32sint',
'rgba32float',
'bgra8unorm'];


g.test('decl_value').
desc('Tests that enumerant values cannot be used as declaration value').
params((u) => u.combine('value', kEnumerantValues).beginSubcases().combine('decl', kValueDecls)).
fn((t) => {
  const decl = `${t.params.decl} x = ${t.params.value};`;
  let code = ``;
  if (t.params.decl === 'override') {
    code = `${decl}`;
  } else {
    code = `fn foo() {
        ${decl}
      }`;
  }
  t.expectCompileResult(false, code);
});