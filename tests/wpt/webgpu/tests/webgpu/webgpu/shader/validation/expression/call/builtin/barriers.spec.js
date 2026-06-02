/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for {storage,texture,workgroup}Barrier() builtins.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kEntryPoints = {
  none: { supportsBarrier: true, code: `` },
  compute: {
    supportsBarrier: true,
    code: `@compute @workgroup_size(1)
fn main() {
  foo();
}`
  },
  vertex: {
    supportsBarrier: false,
    code: `@vertex
fn main() -> @builtin(position) vec4f {
  foo();
  return vec4f();
}`
  },
  fragment: {
    supportsBarrier: false,
    code: `@fragment
fn main() {
  foo();
}`
  },
  compute_and_fragment: {
    supportsBarrier: false,
    code: `@compute @workgroup_size(1)
fn main1() {
  foo();
}

@fragment
fn main2() {
  foo();
}
`
  },
  fragment_without_call: {
    supportsBarrier: true,
    code: `@fragment
fn main() {
}
`
  }
};

g.test('only_in_compute').
specURL('https://www.w3.org/TR/WGSL/#sync-builtin-functions').
desc(
  `
Synchronization functions must only be used in the compute shader stage.
`
).
params((u) =>
u.
combine('entry_point', keysOf(kEntryPoints)).
combine('call', ['bar', 'storageBarrier', 'textureBarrier', 'workgroupBarrier'])
).
fn((t) => {
  if (t.params.call.startsWith('textureBarrier')) {
    t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures');
  }

  const config = kEntryPoints[t.params.entry_point];
  const code = `
${config.code}
fn bar() {}

fn foo() {
  ${t.params.call}();
}`;
  t.expectCompileResult(t.params.call === 'bar' || config.supportsBarrier, code);
});

g.test('no_return_value').
specURL('https://www.w3.org/TR/WGSL/#sync-builtin-functions').
desc(
  `
Barrier functions do not return a value.
`
).
params((u) =>
u.
combine('assign', [false, true]).
combine('rhs', ['bar', 'storageBarrier', 'textureBarrier', 'workgroupBarrier'])
).
fn((t) => {
  if (t.params.rhs.startsWith('textureBarrier')) {
    t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures');
  }

  const code = `
fn bar() {}

fn foo() {
  ${t.params.assign ? '_ = ' : ''} ${t.params.rhs}();
}`;
  t.expectCompileResult(!t.params.assign || t.params.rhs === 'bar()', code);
});