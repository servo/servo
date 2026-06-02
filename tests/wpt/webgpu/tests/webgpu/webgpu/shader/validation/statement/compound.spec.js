/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for compound statements`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  missing_start: {
    src: '}',
    pass: false
  },
  missing_end: {
    src: '{',
    pass: false
  },
  empty: {
    src: '{}',
    pass: true
  },
  semicolon: {
    src: '{;}',
    pass: true
  },
  semicolons: {
    src: '{;;}',
    pass: true
  },
  decl: {
    src: '{const c = 1;}',
    pass: true
  },
  nested: {
    src: '{ {} }',
    pass: true
  }
};

g.test('parse').
desc('Test that compound statments parse').
params((u) => u.combine('stmt', keysOf(kTests))).
fn((t) => {
  const code = `
@vertex
fn vtx() -> @builtin(position) vec4f {
  ${kTests[t.params.stmt].src}
  return vec4f(1);
}
    `;
  t.expectCompileResult(kTests[t.params.stmt].pass, code);
});