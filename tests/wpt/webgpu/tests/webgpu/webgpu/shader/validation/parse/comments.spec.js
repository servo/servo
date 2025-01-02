/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for comments`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('comments').
desc(`Test that valid comments are handled correctly, including nesting.`).
fn((t) => {
  const code = `
/**
 * Here is my shader.
 *
 * /* I can nest /**/ comments. */
 * // I can nest line comments too.
 **/
@fragment // This is the stage
fn main(/*
no
parameters
*/) -> @location(0) vec4<f32> {
  return/*block_comments_delimit_tokens*/vec4<f32>(.4, .2, .3, .1);
}/* terminated block comments are OK at EOF...*/`;
  t.expectCompileResult(true, code);
});

g.test('line_comment_eof').
desc(`Test that line comments can come at EOF.`).
fn((t) => {
  const code = `
@fragment
fn main() -> @location(0) vec4<f32> {
  return vec4<f32>(.4, .2, .3, .1);
}
// line comments are OK at EOF...`;
  t.expectCompileResult(true, code);
});

g.test('line_comment_terminators').
desc(`Test that line comments are terminated by any blankspace other than space and \t`).
params((u) =>
u.
combine('blankspace', [
[' ', 'space'],
['\t', 'tab'],
['\u000a', 'line_feed'],
['\u000b', 'vertical_tab'],
['\u000c', 'form_feed'],
['\u000d', 'carriage_return'],
['\u000d\u000a', 'carriage_return_line_feed'],
['\u0085', 'next_line'],
['\u2028', 'line_separator'],
['\u2029', 'paragraph_separator']]
).
beginSubcases()
).
fn((t) => {
  const code = `// Line comment${t.params.blankspace[0]}const invalid_outside_comment = should_fail`;

  t.expectCompileResult([' ', '\t'].includes(t.params.blankspace[0]), code);
});

g.test('unterminated_block_comment').
desc(`Test that unterminated block comments cause an error`).
params((u) => u.combine('terminated', [true, false]).beginSubcases()).
fn((t) => {
  const code = `
/**
 * Unterminated block comment.
 *
 ${t.params.terminated ? '*/' : ''}`;

  t.expectCompileResult(t.params.terminated, code);
});