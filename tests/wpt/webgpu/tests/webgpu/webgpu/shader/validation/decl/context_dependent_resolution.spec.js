/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that context dependent names do not participate in name resolution.
That is, a declaration named the same as a context dependent name will not interfere.

Context-dependent names:
 * Attribute names
 * Built-in value names
 * Diagnostic severity control
 * Diagnostic triggering rules
 * Enable extensions
 * Language extensions
 * Swizzles
 * Interpolation type
 * Interpolation sampling
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kAttributeCases = {
  align: `struct S { @align(16) x : u32 }`,
  binding: `@group(0) @binding(0) var s : sampler;`,
  builtin: `@vertex fn main() -> @builtin(position) vec4f { return vec4f(); }`,
  // const is not writable
  // diagnostic is a keyword
  group: `@group(0) @binding(0) var s : sampler;`,
  id: `@id(1) override x : i32;`,
  interpolate: `@fragment fn main(@location(0) @interpolate(flat, either) x : i32) { }`,
  invariant: `@fragment fn main(@builtin(position) @invariant pos : vec4f) { }`,
  location: `@fragment fn main(@location(0) x : f32) { }`,
  must_use: `@must_use fn foo() -> u32 { return 0; }`,
  size: `struct S { @size(4) x : u32 }`,
  workgroup_size: `@compute @workgroup_size(1) fn main() { }`,
  compute: `@compute @workgroup_size(1) fn main() { }`,
  fragment: `@fragment fn main() { }`,
  vertex: `@vertex fn main() -> @builtin(position) vec4f { return vec4f(); }`
};

g.test('attribute_names').
desc('Tests attribute names do not use name resolution').
params((u) =>
u.
combine('case', keysOf(kAttributeCases)).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
fn((t) => {
  const code = `
    ${t.params.decl} ${t.params.case} : u32 = 0;
    ${kAttributeCases[t.params.case]}
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kBuiltinCases = {
  vertex_index: `
  @vertex
  fn main(@builtin(vertex_index) idx : u32) -> @builtin(position) vec4f
  { return vec4f(); }`,
  instance_index: `
  @vertex
  fn main(@builtin(instance_index) idx : u32) -> @builtin(position) vec4f
  { return vec4f(); }`,
  position_vertex: `
  @vertex fn main() -> @builtin(position) vec4f
  { return vec4f(); }`,
  position_fragment: `@fragment fn main(@builtin(position) pos : vec4f) { }`,
  front_facing: `@fragment fn main(@builtin(front_facing) x : bool) { }`,
  frag_depth: `@fragment fn main() -> @builtin(frag_depth) f32 { return 0; }`,
  sample_index: `@fragment fn main(@builtin(sample_index) x : u32) { }`,
  sample_mask_input: `@fragment fn main(@builtin(sample_mask) x : u32) { }`,
  sample_mask_output: `@fragment fn main() -> @builtin(sample_mask) u32 { return 0; }`,
  local_invocation_id: `
  @compute @workgroup_size(1)
  fn main(@builtin(local_invocation_id) id : vec3u) { }`,
  local_invocation_index: `
  @compute @workgroup_size(1)
  fn main(@builtin(local_invocation_index) id : u32) { }`,
  global_invocation_id: `
  @compute @workgroup_size(1)
  fn main(@builtin(global_invocation_id) id : vec3u) { }`,
  workgroup_id: `
  @compute @workgroup_size(1)
  fn main(@builtin(workgroup_id) id : vec3u) { }`,
  num_workgroups: `
  @compute @workgroup_size(1)
  fn main(@builtin(num_workgroups) id : vec3u) { }`
};

g.test('builtin_value_names').
desc('Tests builtin value names do not use name resolution').
params((u) =>
u.
combine('case', keysOf(kBuiltinCases)).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
beforeAllSubcases((t) => {
  const wgsl = kBuiltinCases[t.params.case];
  t.skipIf(
    t.isCompatibility && wgsl.includes('sample_mask'),
    'sample_mask is not supported in compatibility mode'
  );
  t.skipIf(
    t.isCompatibility && wgsl.includes('sample_index'),
    'sample_index is not supported in compatibility mode'
  );
}).
fn((t) => {
  const code = `
    ${t.params.decl} ${t.params.case} : u32 = 0;
    ${kBuiltinCases[t.params.case]}
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kDiagnosticSeverityCases = {
  error: `
  diagnostic(error, derivative_uniformity);
  @diagnostic(error, derivative_uniformity) fn foo() { }
  `,
  warning: `
  diagnostic(warning, derivative_uniformity);
  @diagnostic(warning, derivative_uniformity) fn foo() { }
  `,
  off: `
  diagnostic(off, derivative_uniformity);
  @diagnostic(off, derivative_uniformity) fn foo() { }
  `,
  info: `
  diagnostic(info, derivative_uniformity);
  @diagnostic(info, derivative_uniformity) fn foo() { }
  `
};

g.test('diagnostic_severity_names').
desc('Tests diagnostic severity names do not use name resolution').
params((u) =>
u.
combine('case', keysOf(kDiagnosticSeverityCases)).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
fn((t) => {
  const code = `
    ${kDiagnosticSeverityCases[t.params.case]}
    ${t.params.decl} ${t.params.case} : u32 = 0;
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kDiagnosticRuleCases = {
  derivative_uniformity: `
  diagnostic(off, derivative_uniformity);
  @diagnostic(warning, derivative_uniformity) fn foo() { }`,
  unknown_rule: `
  diagnostic(off, unknown_rule);
  @diagnostic(warning, unknown_rule) fn foo() { }`,
  unknown: `
  diagnostic(off, unknown.rule);
  @diagnostic(warning, unknown.rule) fn foo() { }`,
  rule: `
  diagnostic(off, unknown.rule);
  @diagnostic(warning, unknown.rule) fn foo() { }`
};

g.test('diagnostic_rule_names').
desc('Tests diagnostic rule names do not use name resolution').
params((u) =>
u.
combine('case', keysOf(kDiagnosticRuleCases)).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
fn((t) => {
  const code = `
    ${kDiagnosticRuleCases[t.params.case]}
    ${t.params.decl} ${t.params.case} : u32 = 0;
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kEnableCases = {
  f16: `enable f16;`
};

g.test('enable_names').
desc('Tests enable extension names do not use name resolution').
params((u) =>
u.
combine('case', keysOf(kEnableCases)).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  const code = `
    ${kEnableCases[t.params.case]}
    ${t.params.decl} ${t.params.case} : u32 = 0;
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kLanguageCases = {
  readonly_and_readwrite_storage_textures: `requires readonly_and_readwrite_storage_textures;`,
  packed_4x8_integer_dot_product: `requires packed_4x8_integer_dot_product;`,
  unrestricted_pointer_parameters: `requires unrestricted_pointer_parameters;`,
  pointer_composite_access: `requires pointer_composite_access;`
};

g.test('language_names').
desc('Tests language extension names do not use name resolution').
params((u) =>
u.
combine('case', keysOf(kLanguageCases)).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
fn((t) => {
  t.skipIf(!t.hasLanguageFeature(t.params.case), 'Missing language feature');
  const code = `
    ${kLanguageCases[t.params.case]}
    ${t.params.decl} ${t.params.case} : u32 = 0;
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kSwizzleCases = [
'x',
'y',
'z',
'w',
'xy',
'yxz',
'wxyz',
'xyxy',
'r',
'g',
'b',
'a',
'rgb',
'arr',
'bgra',
'agra'];


g.test('swizzle_names').
desc('Tests swizzle names do not use name resolution').
params((u) =>
u.
combine('case', kSwizzleCases).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
fn((t) => {
  let code = `${t.params.decl} ${t.params.case} : u32 = 0;\n`;
  if (t.params.case.length === 1) {
    for (let i = 2; i <= 4; i++) {
      code += `${t.params.decl} ${t.params.case.padEnd(i, t.params.case[0])} : u32 = 0;\n`;
    }
  }
  code += `fn foo() {
      var x : vec4f;
      _ = x.${t.params.case};
    `;
  if (t.params.case.length === 1) {
    for (let i = 2; i <= 4; i++) {
      code += `_ = x.${t.params.case.padEnd(i, t.params.case[0])};\n`;
    }
  }
  code += `}
    fn use_var() -> u32 {
      return ${t.params.case};
    }`;
  t.expectCompileResult(true, code);
});

const kInterpolationTypeCases = ['perspective', 'linear', 'flat'];

g.test('interpolation_type_names').
desc('Tests interpolation type names do not use name resolution').
params((u) =>
u.
combine('case', kInterpolationTypeCases).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
beforeAllSubcases((t) => {
  t.skipIf(
    t.isCompatibility && t.params.case === 'linear',
    'compatibility mode does not support linear interpolation type'
  );
}).
fn((t) => {
  const attr =
  t.isCompatibility && t.params.case === 'flat' ?
  `@interpolate(flat, either)` :
  `@interpolate(${t.params.case})`;
  const code = `
    ${t.params.decl} ${t.params.case} : u32 = 0;
    @fragment fn main(@location(0) ${attr} x : f32) { }
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kInterpolationSamplingCases = ['center', 'centroid', 'sample'];

g.test('interpolation_sampling_names').
desc('Tests interpolation type names do not use name resolution').
params((u) =>
u.
combine('case', kInterpolationSamplingCases).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
beforeAllSubcases((t) => {
  t.skipIf(
    t.isCompatibility && t.params.case === 'sample',
    'compatibility mode does not support sample sampling'
  );
}).
fn((t) => {
  const code = `
    ${t.params.decl} ${t.params.case} : u32 = 0;
    @fragment fn main(@location(0) @interpolate(perspective, ${t.params.case}) x : f32) { }
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});

const kInterpolationFlatCases = ['first', 'either'];

g.test('interpolation_flat_names').
desc('Tests interpolation type names do not use name resolution').
params((u) =>
u.
combine('case', kInterpolationFlatCases).
beginSubcases().
combine('decl', ['override', 'const', 'var<private>'])
).
beforeAllSubcases((t) => {
  t.skipIf(
    t.isCompatibility && t.params.case === 'first',
    'compatibility mode does not support first sampling'
  );
}).
fn((t) => {
  const code = `
    ${t.params.decl} ${t.params.case} : u32 = 0;
    @fragment fn main(@location(0) @interpolate(flat, ${t.params.case}) x : u32) { }
    fn use_var() -> u32 {
      return ${t.params.case};
    }
    `;

  t.expectCompileResult(true, code);
});