/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for break`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  loop_break: {
    src: 'loop { break; }',
    pass: true,
  },
  loop_if_break: {
    src: 'loop { if true { break; } }',
    pass: true,
  },
  continuing_break_if: {
    src: 'loop { continuing { break if (true); } }',
    pass: true,
  },
  while_break: {
    src: 'while true { break; }',
    pass: true,
  },
  while_if_break: {
    src: 'while true { if true { break; } }',
    pass: true,
  },
  for_break: {
    src: 'for (;;) { break; }',
    pass: true,
  },
  for_if_break: {
    src: 'for (;;) { if true { break; } }',
    pass: true,
  },
  switch_case_break: {
    src: 'switch(1) { default: { break; } }',
    pass: true,
  },
  switch_case_if_break: {
    src: 'switch(1) { default: { if true { break; } } }',
    pass: true,
  },
  break: {
    src: 'break;',
    pass: false,
  },
  return_break: {
    src: 'return break;',
    pass: false,
  },
  if_break: {
    src: 'if true { break; }',
    pass: false,
  },
  continuing_break: {
    src: 'loop { continuing { break; } }',
    pass: false,
  },
  continuing_if_break: {
    src: 'loop { continuing { if (true) { break; } } }',
    pass: false,
  },
  switch_break: {
    src: 'switch(1) { break; }',
    pass: false,
  },
};

g.test('placement')
  .desc('Test that break placement is validated correctly')
  .params(u => u.combine('stmt', keysOf(kTests)))
  .fn(t => {
    const code = `
@vertex
fn vtx() -> @builtin(position) vec4f {
  ${kTests[t.params.stmt].src}
  return vec4f(1);
}
    `;
    t.expectCompileResult(kTests[t.params.stmt].pass, code);
  });
