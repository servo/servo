/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for swizzle assignments.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('valid').
desc('Valid swizzle assignments').
params((u) =>
u.combine('elemType', ['f32', 'i32', 'u32']).combine('vecSize', [2, 3, 4])
).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const { elemType, vecSize } = t.params;
  const swizzle = 'xyzw'.substring(0, vecSize);
  const vecType = `vec${swizzle.length}<${elemType}>`;
  const code = `
@fragment
fn main() {
  var v = vec4<${elemType}>(0);
  v.${swizzle} = ${vecType}(1);
}
`;
  t.expectCompileResult(true, code);
});

g.test('invalid_lhs_not_reference').
desc('Invalid swizzle assignment where LHS is not a reference').
params((u) => u.combine('lhs', ['vec4f()', 'const_vec', 'foo()'])).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
const const_vec = vec4f();

fn foo() -> vec4f {
  return vec4f();
}

@fragment
fn main() {
  var v = vec4f();
  ${t.params.lhs}.xyz = vec3(0.0);
}
`;
  t.expectCompileResult(false, code);
});

g.test('invalid_duplicate_components').
desc('Invalid swizzle assignment with duplicate LHS components').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
@fragment
fn main() {
  var v = vec4f();
  v.xx = vec2f();
}
`;
  t.expectCompileResult(false, code);
});

g.test('invalid_component_mismatch').
desc('Invalid swizzle assignment with mismatched number of components').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
@fragment
fn main() {
  var v = vec4f();
  v.xy = vec3f();
}
`;
  t.expectCompileResult(false, code);
});

g.test('invalid_component_oob').
desc('Invalid swizzle assignment with components out of bounds').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
@fragment
fn main() {
  var v = vec3f();
  v.xyzw = vec4f();
}
`;
  t.expectCompileResult(false, code);
});

g.test('invalid_type_mismatch').
desc('Invalid swizzle assignment with mismatched types').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
@fragment
fn main() {
  var v = vec4f();
  v.xy = vec2i();
}
`;
  t.expectCompileResult(false, code);
});

g.test('invalid_mixed_letter_schemes').
desc('Invalid swizzle assignment with mixed letter schemes (xyzw vs. rgba)').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
@fragment
fn main() {
  var v = vec4f();
  v.xr = vec2i();
}
`;
  t.expectCompileResult(false, code);
});

g.test('invalid_address_of_swizzle_view').
desc('Invalid to take the address of a swizzle view').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
@fragment
fn main() {
  var v = vec4f();
  let p = &v.xy;
}
`;
  t.expectCompileResult(false, code);
});

g.test('invalid_index_into_swizzle_view').
desc('Invalid to index into a swizzle view on the lhs').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('swizzle_assignment');
  const code = `
@fragment
fn main() {
  var v = vec2u();
  v.xy[0] = 1;
`;
  t.expectCompileResult(false, code);
});