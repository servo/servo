/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for 'for' statements'`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { kTestTypes } from './test_types.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('condition_type').
desc(`Tests that a 'for' condition must be a bool type`).
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
  for (; ${type.value};) {
    return true;
  }
  return false;
}
`;

  const pass = t.params.type === 'bool';
  t.expectCompileResult(pass, code);
});

const kTests = {
  break: { wgsl: `for (;;) { break; }`, pass: true },

  init_var: { wgsl: `for (var i = 1;;) { break; }`, pass: true },
  init_var_type: { wgsl: `for (var i : i32 = 1;;) { break; }`, pass: true },
  init_var_function: { wgsl: `for (var<function> i = 1;;) { break; }`, pass: true },
  init_var_function_type: { wgsl: `for (var<function> i : i32 = 1;;) { break; }`, pass: true },
  init_let: { wgsl: `for (let i = 1;;) { break; }`, pass: true },
  init_let_type: { wgsl: `for (let i : u32 = 1;;) { break; }`, pass: true },
  init_const: { wgsl: `for (const i = 1;;) { break; }`, pass: true },
  init_const_type: { wgsl: `for (const i : f32 = 1;;) { break; }`, pass: true },
  init_call: { wgsl: `for (x();;) { break; }`, pass: true },
  init_phony: { wgsl: `for (_ = v;;) { break; }`, pass: true },
  init_increment: { wgsl: `for (v++;;) { break; }`, pass: true },
  init_compound_assign: { wgsl: `for (v += 3;;) { break; }`, pass: true },

  cond_true: { wgsl: `for (;true;) { break; }`, pass: true },

  cont_call: { wgsl: `for (;;x()) { break; }`, pass: true },
  cont_phony: { wgsl: `for (;;_ = v) { break; }`, pass: true },
  cont_increment: { wgsl: `for (;;v++) { break; }`, pass: true },
  cont_compound_assign: { wgsl: `for (;;v += 3) { break; }`, pass: true },

  init_cond: { wgsl: `for (var i = 1; i < 5;) {}`, pass: true },
  cond_cont: { wgsl: `for (;v < 5; v++) {}`, pass: true },
  init_cond_cont: { wgsl: `for (var i = 0; i < 5; i++) {}`, pass: true },
  init_shadow: { wgsl: `for (var f = 0; f < 5; f++) {}`, pass: true },

  no_semicolon: { wgsl: `for () { break; }`, pass: false },
  one_semicolon: { wgsl: `for (;) { break; }`, pass: false },
  no_paren: { wgsl: `for ;; { break; }`, pass: false },
  empty: { wgsl: `for (;;) {}`, pass: false }, // note: fails due to behavior-analysis
  init_expr: { wgsl: `for (true;;) { break; }`, pass: false },
  cond_stmt: { wgsl: `for (;var i = 0;) { break; }`, pass: false },
  cont_expr: { wgsl: `for (;;true) { break; }`, pass: false },
  cont_var: { wgsl: `for (;;var i = 1) { break; }`, pass: false },
  cont_var_type: { wgsl: `for (;;var i : i32 = 1) { break; }`, pass: false },
  cont_var_function: { wgsl: `for (;;var<function> i = 1) { break; }`, pass: false },
  cont_var_function_type: { wgsl: `for (;;var<function> i : i32 = 1) { break; }`, pass: false },
  cont_let: { wgsl: `for (;;let i = 1) { break; }`, pass: false },
  cont_let_type: { wgsl: `for (;;let i : u32 = 1) { break; }`, pass: false }
};

g.test('parse').
desc(`Test that 'for' statements are parsed correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
fn((t) => {
  const code = `
fn f() {
  var v = 1;
  ${kTests[t.params.test].wgsl}
}

fn x() {}
`;
  t.expectCompileResult(kTests[t.params.test].pass, code);
});