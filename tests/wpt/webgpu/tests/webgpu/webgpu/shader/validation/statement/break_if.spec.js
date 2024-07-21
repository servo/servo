/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for 'break if' statements'`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { kTestTypes } from './test_types.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('condition_type').
desc(`Tests that an 'break if' condition must be a bool type`).
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
  compound_break: {
    src: '{ break if true; }',
    pass: false
  },
  loop_break: {
    src: 'loop { break if true; }',
    pass: false
  },
  loop_if_break: {
    src: 'loop { if true { break if false; } }',
    pass: false
  },
  continuing_break_if: {
    src: 'loop { continuing { break if true; } }',
    pass: true
  },
  continuing_break_if_parens: {
    src: 'loop { continuing { break if (true); } }',
    pass: true
  },
  continuing_break_if_not_last: {
    src: 'loop { continuing { break if (true); let a = 4;} }',
    pass: false
  },
  while_break: {
    src: 'while true { break if true; }',
    pass: false
  },
  while_if_break: {
    src: 'while true { if true { break if true; } }',
    pass: false
  },
  for_break: {
    src: 'for (;;) { break if true; }',
    pass: false
  },
  for_if_break: {
    src: 'for (;;) { if true { break if true; } }',
    pass: false
  },
  switch_case_break: {
    src: 'switch(1) { default: { break if true; } }',
    pass: false
  },
  switch_case_if_break: {
    src: 'switch(1) { default: { if true { break if true; } } }',
    pass: false
  },
  break: {
    src: 'break if true;',
    pass: false
  },
  return_break: {
    src: 'return break if true;',
    pass: false
  },
  if_break: {
    src: 'if true { break if true; }',
    pass: false
  },
  continuing_if_break: {
    src: 'loop { continuing { if (true) { break if true; } } }',
    pass: false
  },
  switch_break: {
    src: 'switch(1) { break if true; }',
    pass: false
  }
};

g.test('placement').
desc('Test that break if placement is validated correctly').
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