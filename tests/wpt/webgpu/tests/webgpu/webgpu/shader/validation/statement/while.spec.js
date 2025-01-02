/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for 'while' statements'`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { kTestTypes } from './test_types.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('condition_type').
desc(`Tests that a 'while' condition must be a bool type`).
params((u) => u.combine('type', keysOf(kTestTypes))).
beforeAllSubcases((t) => {
  if (kTestTypes[t.params.type].requires === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kTestTypes[t.params.type];
  const code = `
${type.requires ? `enable ${type.requires};` : ''}

${type.header ?? ''}

fn f() -> bool {
  while (${type.value}) {
    return true;
  }
  return false;
}
`;

  const pass = t.params.type === 'bool';
  t.expectCompileResult(pass, code);
});

const kTests = {
  true: { wgsl: `while true {}`, pass: true },
  paren_true: { wgsl: `while (true) {}`, pass: true },
  true_break: { wgsl: `while true { break; }`, pass: true },
  true_discard: { wgsl: `while true { discard; }`, pass: true },
  true_return: { wgsl: `while true { return; }`, pass: true },
  expr: { wgsl: `while expr {}`, pass: true },
  paren_expr: { wgsl: `while (expr) {}`, pass: true },

  while: { wgsl: `while`, pass: false },
  block: { wgsl: `while{}`, pass: false },
  semicolon: { wgsl: `while;`, pass: false },
  true_lbrace: { wgsl: `while true {`, pass: false },
  true_rbrace: { wgsl: `while true }`, pass: false },

  lparen_true: { wgsl: `while (true {}`, pass: false },
  rparen_true: { wgsl: `while )true {}`, pass: false },
  true_lparen: { wgsl: `while true( {}`, pass: false },
  true_rparen: { wgsl: `while true) {}`, pass: false },
  lparen_true_lparen: { wgsl: `while (true( {}`, pass: false },
  rparen_true_rparen: { wgsl: `while )true) {}`, pass: false }
};

g.test('parse').
desc(`Test that 'while' statements are parsed correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
fn((t) => {
  const code = `
fn f() {
  let expr = true;
  ${kTests[t.params.test].wgsl}
}`;
  t.expectCompileResult(kTests[t.params.test].pass, code);
});