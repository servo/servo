/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for 'if' statements'`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { kTestTypes } from './test_types.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('condition_type').
desc(`Tests that an 'if' condition must be a bool type`).
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
  if ${type.value} {
    return true;
  }
  return false;
}
`;

  const pass = t.params.type === 'bool';
  t.expectCompileResult(pass, code);
});

g.test('else_condition_type').
desc(`Tests that an 'else if' condition must be a bool type`).
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

fn f(c : bool) -> bool {
  if (c) {
    return true;
  } else if (${type.value}) {
    return true;
  }
  return false;
}
`;

  const pass = t.params.type === 'bool';
  t.expectCompileResult(pass, code);
});

const kTests = {
  true: { wgsl: `if true {}`, pass: true },
  paren_true: { wgsl: `if (true) {}`, pass: true },
  expr: { wgsl: `if expr {}`, pass: true },
  paren_expr: { wgsl: `if (expr) {}`, pass: true },

  true_else: { wgsl: `if true {} else {}`, pass: true },
  paren_true_else: { wgsl: `if (true) {} else {}`, pass: true },
  expr_else: { wgsl: `if expr {} else {}`, pass: true },
  paren_expr_else: { wgsl: `if (expr) {} else {}`, pass: true },

  true_else_if_true: { wgsl: `if true {} else if true {}`, pass: true },
  paren_true_else_if_paren_true: { wgsl: `if (true) {} else if (true) {}`, pass: true },
  true_else_if_paren_true: { wgsl: `if true {} else if (true) {}`, pass: true },
  paren_true_else_if_true: { wgsl: `if (true) {} else if true {}`, pass: true },

  expr_else_if_expr: { wgsl: `if expr {} else if expr {}`, pass: true },
  paren_expr_else_if_paren_expr: { wgsl: `if (expr) {} else if (expr) {}`, pass: true },
  expr_else_if_paren_expr: { wgsl: `if expr {} else if (expr) {}`, pass: true },
  paren_expr_else_if_expr: { wgsl: `if (expr) {} else if expr {}`, pass: true },

  if: { wgsl: `if`, pass: false },
  block: { wgsl: `if{}`, pass: false },
  semicolon: { wgsl: `if;`, pass: false },
  true_lbrace: { wgsl: `if true {`, pass: false },
  true_rbrace: { wgsl: `if true }`, pass: false },

  lparen_true: { wgsl: `if (true {}`, pass: false },
  rparen_true: { wgsl: `if )true {}`, pass: false },
  true_lparen: { wgsl: `if true( {}`, pass: false },
  true_rparen: { wgsl: `if true) {}`, pass: false },

  true_else_if_no_block: { wgsl: `if true {} else if `, pass: false },
  true_else_if: { wgsl: `if true {} else if {}`, pass: false },
  true_else_if_semicolon: { wgsl: `if true {} else if ;`, pass: false },
  true_else_if_true_lbrace: { wgsl: `if true {} else if true {`, pass: false },
  true_else_if_true_rbrace: { wgsl: `if true {} else if true }`, pass: false },

  true_else_if_lparen_true: { wgsl: `if true {} else if (true {}`, pass: false },
  true_else_if_rparen_true: { wgsl: `if true {} else if )true {}`, pass: false },
  true_else_if_true_lparen: { wgsl: `if true {} else if true( {}`, pass: false },
  true_else_if_true_rparen: { wgsl: `if true {} else if true) {}`, pass: false },

  else: { wgsl: `else { }`, pass: false },
  else_if: { wgsl: `else if true { }`, pass: false },
  true_elif: { wgsl: `if (true) { } elif (true) {}`, pass: false },
  true_elsif: { wgsl: `if (true) { } elsif (true) {}`, pass: false },
  elif: { wgsl: `elif (true) {}`, pass: false },
  elsif: { wgsl: `elsif (true) {}`, pass: false }
};

g.test('parse').
desc(`Test that 'if' statements are parsed correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
fn((t) => {
  const code = `
fn f() {
  let expr = true;
  ${kTests[t.params.test].wgsl}
}`;
  t.expectCompileResult(kTests[t.params.test].pass, code);
});