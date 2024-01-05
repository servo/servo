/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for override declarations
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
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