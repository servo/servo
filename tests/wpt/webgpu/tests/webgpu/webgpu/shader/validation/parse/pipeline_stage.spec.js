/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for pipeline stage`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValidVertex = new Set(['', '@vertex', '@\tvertex', '@/^comment^/vertex']);
const kInvalidVertex = new Set(['@mvertex', '@vertex()', '@vertex )', '@vertex(']);
g.test('vertex_parsing')
  .desc(`Test that @vertex is parsed correctly.`)
  .params(u => u.combine('val', new Set([...kValidVertex, ...kInvalidVertex])))
  .fn(t => {
    const v = t.params.val.replace(/\^/g, '*');
    const r = t.params.val !== '' ? '@builtin(position)' : '';
    const code = `
${v}
fn main() -> ${r} vec4<f32> {
  return vec4<f32>(.4, .2, .3, .1);
}`;
    t.expectCompileResult(kValidVertex.has(t.params.val), code);
  });

const kValidFragment = new Set(['', '@fragment', '@\tfragment', '@/^comment^/fragment']);
const kInvalidFragment = new Set(['@mfragment', '@fragment()', '@fragment )', '@fragment(']);
g.test('fragment_parsing')
  .desc(`Test that @fragment is parsed correctly.`)
  .params(u => u.combine('val', new Set([...kValidFragment, ...kInvalidFragment])))
  .fn(t => {
    const v = t.params.val.replace(/\^/g, '*');
    const r = t.params.val !== '' ? '@location(0)' : '';
    const code = `
${v}
fn main() -> ${r} vec4<f32> {
  return vec4<f32>(.4, .2, .3, .1);
}`;
    t.expectCompileResult(kValidFragment.has(t.params.val), code);
  });

const kValidCompute = new Set(['', '@compute', '@\tcompute', '@/^comment^/compute']);
const kInvalidCompute = new Set(['@mcompute', '@compute()', '@compute )', '@compute(']);
g.test('compute_parsing')
  .desc(`Test that @compute is parsed correctly.`)
  .params(u => u.combine('val', new Set([...kValidCompute, ...kInvalidCompute])))
  .fn(t => {
    let v = t.params.val.replace(/\^/g, '*');
    // Always add a workgroup size unless there is no parameter
    if (v !== '') {
      v += '\n@workgroup_size(1)';
    }
    const code = `
${v}
fn main() {}
`;
    t.expectCompileResult(kValidCompute.has(t.params.val), code);
  });

g.test('multiple_entry_points')
  .desc(`Test that multiple entry points are allowed.`)
  .fn(t => {
    const code = `
@compute @workgroup_size(1) fn compute_1() {}
@compute @workgroup_size(1) fn compute_2() {}

@fragment fn frag_1() -> @location(2) vec4f { return vec4f(1); }
@fragment fn frag_2() -> @location(2) vec4f { return vec4f(1); }
@fragment fn frag_3() -> @location(2) vec4f { return vec4f(1); }

@vertex fn vtx_1() -> @builtin(position) vec4f { return vec4f(1); }
@vertex fn vtx_2() -> @builtin(position) vec4f { return vec4f(1); }
@vertex fn vtx_3() -> @builtin(position) vec4f { return vec4f(1); }
`;
    t.expectCompileResult(true, code);
  });

g.test('duplicate_compute_on_function')
  .desc(`Test that duplcate @compute attributes are not allowed.`)
  .params(u => u.combine('dupe', ['', '@compute']))
  .fn(t => {
    const code = `
@compute ${t.params.dupe} @workgroup_size(1) fn compute_1() {}
`;
    t.expectCompileResult(t.params.dupe === '', code);
  });

g.test('duplicate_fragment_on_function')
  .desc(`Test that duplcate @fragment attributes are not allowed.`)
  .params(u => u.combine('dupe', ['', '@fragment']))
  .fn(t => {
    const code = `
@fragment ${t.params.dupe} fn vtx() -> @location(0) vec4f { return vec4f(1); }
`;
    t.expectCompileResult(t.params.dupe === '', code);
  });

g.test('duplicate_vertex_on_function')
  .desc(`Test that duplcate @vertex attributes are not allowed.`)
  .params(u => u.combine('dupe', ['', '@vertex']))
  .fn(t => {
    const code = `
@vertex ${t.params.dupe} fn vtx() -> @builtin(position) vec4f { return vec4f(1); }
`;
    t.expectCompileResult(t.params.dupe === '', code);
  });

g.test('placement')
  .desc('Tests the locations @align is allowed to appear')
  .params(u =>
    u
      .combine('scope', [
        'private-var',
        'storage-var',
        'struct-member',
        'fn-param',
        'fn-var',
        'fn-return',
        'while-stmt',
        undefined,
      ])
      .combine('attr', ['@compute', '@fragment', '@vertex'])
  )
  .fn(t => {
    const scope = t.params.scope;

    const attr = t.params.attr;
    const code = `
      ${scope === 'private-var' ? attr : ''}
      var<private> priv_var : i32;

      ${scope === 'storage-var' ? attr : ''}
      @group(0) @binding(0)
      var<storage> stor_var : i32;

      struct A {
        ${scope === 'struct-member' ? attr : ''}
        a : i32,
      }

      @vertex
      fn f(
        ${scope === 'fn-param' ? attr : ''}
        @location(0) b : i32,
      ) -> ${scope === 'fn-return' ? attr : ''} @builtin(position) vec4f {
        ${scope === 'fn-var' ? attr : ''}
        var<function> func_v : i32;

        ${scope === 'while-stmt' ? attr : ''}
        while false {}

        return vec4(1, 1, 1, 1);
      }
    `;

    t.expectCompileResult(scope === undefined, code);
  });
