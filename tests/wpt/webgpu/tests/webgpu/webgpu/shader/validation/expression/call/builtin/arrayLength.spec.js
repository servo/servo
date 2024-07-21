/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for arrayLength builtins.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('bool_type').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `
arrayLength accepts only runtime-sized arrays
`
).
fn((t) => {
  const code = `
@compute @workgroup_size(1)
fn main() {
  var b = true;
  _ = arrayLength(&b);
}`;

  t.expectCompileResult(false, code);
});

const atomic_types = ['u32', 'i32'].map((j) => `atomic<${j}>`);
const vec_types = [2, 3, 4].
map((i) => ['i32', 'u32', 'f32', 'f16'].map((j) => `vec${i}<${j}>`)).
reduce((a, c) => a.concat(c), []);
const f32_matrix_types = [2, 3, 4].
map((i) => [2, 3, 4].map((j) => `mat${i}x${j}f`)).
reduce((a, c) => a.concat(c), []);
const f16_matrix_types = [2, 3, 4].
map((i) => [2, 3, 4].map((j) => `mat${i}x${j}<f16>`)).
reduce((a, c) => a.concat(c), []);

g.test('type').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `
arrayLength accepts only runtime-sized arrays
`
).
params((u) =>
u.combine('type', [
'i32',
'u32',
'f32',
'f16',
...f32_matrix_types,
...f16_matrix_types,
...vec_types,
...atomic_types,
'T',
'array<i32, 2>',
'array<i32>']
)
).
beforeAllSubcases((t) => {
  if (t.params.type.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const code = `
struct T {
  b: i32,
}
struct S {
  ary: ${t.params.type}
}

@group(0) @binding(0) var<storage, read_write> items: S;

@compute @workgroup_size(1)
fn main() {
  _ = arrayLength(&items.ary);
}`;

  t.expectCompileResult(t.params.type === 'array<i32>', code);
});

// Note, the `write` case actually fails because you can't declare a storage buffer of
// access_mode `write`.
g.test('access_mode').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `
arrayLength runtime-sized array must have an access_mode of read or read_write
`
).
params((u) => u.combine('mode', ['read', 'read_write', 'write'])).
fn((t) => {
  const code = `
struct S {
  ary: array<i32>,
}

@group(0) @binding(0) var<storage, ${t.params.mode}> items: S;

@compute @workgroup_size(1)
fn main() {
  _ = arrayLength(&items.ary);
}`;

  t.expectCompileResult(t.params.mode !== 'write', code);
});

g.test('must_use').
desc('Test that the result must be used').
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const code = `
    @group(0) @binding(0) var<storage> v : array<u32>;
    fn foo() {
      ${t.params.use ? '_ =' : ''} arrayLength(&v);
    }`;
  t.expectCompileResult(t.params.use, code);
});