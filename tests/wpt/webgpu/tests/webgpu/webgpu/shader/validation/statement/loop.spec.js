/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for 'loop' statements'`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { kTestTypes } from './test_types.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('break_if_type').
desc(`Tests that a 'break if' condition must be a bool type`).
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

fn f() {
  loop {
    continuing {
      break if ${type.value};
    }
  }
}
`;

  const pass = t.params.type === 'bool';
  t.expectCompileResult(pass, code);
});

const kTests = {
  break: { wgsl: `loop { break; }`, pass: true },
  return: { wgsl: `loop { return; }`, pass: true },
  break_continuing: { wgsl: `loop { break; continuing {} }`, pass: true },
  var_break: { wgsl: `loop { var a = 1; break; }`, pass: true },
  var_break_continuing_inc: {
    wgsl: `loop { var a = 1; break; continuing { a += 1; }}`,
    pass: true
  },
  var_break_continuing_discard: {
    wgsl: `loop { var a = 1; break; continuing { discard; }}`,
    pass: true
  },
  continuing_break_if: {
    wgsl: `loop { continuing { break if true; } }`,
    pass: true
  },

  expr_break: { wgsl: `loop expr { break; }`, pass: false },
  loop: { wgsl: `loop`, pass: false },
  continuing_break: { wgsl: `loop { continuing {} break; }`, pass: false },
  break_continuing_continue: { wgsl: `loop { break; continuing { continue; } }`, pass: false },
  break_continuing_return: { wgsl: `loop { break; continuing { return; } }`, pass: false },
  break_continuing_if_break: {
    wgsl: `loop { break; continuing { if true { break; } }`,
    pass: false
  },
  break_continuing_if_return: {
    wgsl: `loop { break; continuing { if true { return; } }`,
    pass: false
  },
  break_continuing_lbrace: { wgsl: `loop { break; continuing { }`, pass: false },
  break_continuing_rbrace: { wgsl: `loop { break; continuing } }`, pass: false },
  continuing: { wgsl: `loop { continuing {} }`, pass: false },
  semicolon: { wgsl: `loop;`, pass: false },
  lbrace: { wgsl: `loop {`, pass: false },
  rbrace: { wgsl: `loop }`, pass: false },
  lparen: { wgsl: `loop ({}`, pass: false },
  rparen: { wgsl: `loop ){}`, pass: false },

  // note: these parse, but fails due to behavior-analysis
  continue: { wgsl: `loop { continue; }`, pass: false },
  discard: { wgsl: `loop { discard; }`, pass: false },
  empty: { wgsl: `loop{}`, pass: false }
};

g.test('parse').
desc(`Test that 'loop' statements are parsed correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
fn((t) => {
  const code = `
fn f() {
  let expr = true;
  ${kTests[t.params.test].wgsl}
}`;
  t.expectCompileResult(kTests[t.params.test].pass, code);
});