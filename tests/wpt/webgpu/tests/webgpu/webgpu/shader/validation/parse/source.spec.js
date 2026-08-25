/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for source parsing`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('valid_source').
desc(`Tests that a valid source is consumed successfully.`).
fn((t) => {
  const code = `
    @fragment
    fn main() -> @location(0) vec4<f32> {
      return vec4<f32>(.4, .2, .3, .1);
    }`;
  t.expectCompileResult(true, code);
});

g.test('empty').
desc(`Test that an empty source is consumed successfully.`).
fn((t) => {
  t.expectCompileResult(true, '');
});

g.test('invalid_source').
desc(`Tests that a source which does not match the grammar fails.`).
fn((t) => {
  t.expectCompileResult(false, 'invalid_source');
});