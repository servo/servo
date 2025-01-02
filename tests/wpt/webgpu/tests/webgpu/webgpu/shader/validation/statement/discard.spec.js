/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for discard`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('placement').
desc('Test that discard usage is validated').
params((u) =>
u.combine('place', ['compute', 'vertex', 'fragment', 'module', 'subfrag', 'subvert', 'subcomp'])
).
fn((t) => {
  const pos = {
    module: '',
    subvert: '',
    subfrag: '',
    subcomp: '',
    vertex: '',
    fragment: '',
    compute: ''
  };

  pos[t.params.place] = 'discard;';

  const code = `
${pos.module}

fn subvert() {
  ${pos.subvert}
}

@vertex
fn vtx() -> @builtin(position) vec4f {
  ${pos.vertex}
  subvert();
  return vec4f(1);
}

fn subfrag() {
  ${pos.subfrag}
}

@fragment
fn frag() -> @location(0) vec4f {
  ${pos.fragment}
  subfrag();
  return vec4f(1);
}

fn subcomp() {
  ${pos.subcomp}
}

@compute
@workgroup_size(1)
fn comp() {
  ${pos.compute}
  subcomp();
}
`;

  const pass = ['fragment', 'subfrag'].includes(t.params.place);
  t.expectCompileResult(pass, code);
});