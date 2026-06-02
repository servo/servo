/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for continuing`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  continuing_break_if: {
    src: 'loop { continuing { break if true; } }',
    pass: true
  },
  continuing_empty: {
    src: 'loop { if a == 4 { break; } continuing { } }',
    pass: true
  },
  continuing_break_if_parens: {
    src: 'loop { continuing { break if (true); } }',
    pass: true
  },
  continuing_discard: {
    src: 'loop { if a == 4 { break; } continuing { discard; } }',
    pass: true
  },
  continuing_continue_nested: {
    src: 'loop { if a == 4 { break; } continuing { loop { if a == 4 { break; } continue; } } }',
    pass: true
  },
  continuing_continue: {
    src: 'loop { if a == 4 { break; } continuing { continue; } }',
    pass: false
  },
  continuing_break: {
    src: 'loop { continuing { break; } }',
    pass: false
  },
  continuing_for: {
    src: 'loop { if a == 4 { break; } continuing { for(;a < 4;) { } } }',
    pass: true
  },
  continuing_for_break: {
    src: 'loop { if a == 4 { break; } continuing { for(;;) { break; } } }',
    pass: true
  },
  continuing_while: {
    src: 'loop { if a == 4 { break; } continuing { while a < 4 { } } }',
    pass: true
  },
  continuing_while_break: {
    src: 'loop { if a == 4 { break; } continuing { while true { break; } } }',
    pass: true
  },
  continuing_semicolon: {
    src: 'loop { if a == 4 { break; } continuing { ; } }',
    pass: true
  },
  continuing_functionn_call: {
    src: 'loop { if a == 4 { break; } continuing { _ = b(); } }',
    pass: true
  },
  continuing_let: {
    src: 'loop { if a == 4 { break; } continuing { let c = b(); } }',
    pass: true
  },
  continuing_var: {
    src: 'loop { if a == 4 { break; } continuing { var a = b(); } }',
    pass: true
  },
  continuing_const: {
    src: 'loop { if a == 4 { break; } continuing { const a = 1; } }',
    pass: true
  },
  continuing_block: {
    src: 'loop { if a == 4 { break; } continuing { { } } }',
    pass: true
  },
  continuing_const_assert: {
    src: 'loop { if a == 4 { break; } continuing { const_assert(1 != 2); } }',
    pass: true
  },
  continuing_loop: {
    src: 'loop { if a == 4 { break; } continuing { loop { break; } } }',
    pass: true
  },
  continuing_if: {
    src: 'loop { if a == 4 { break; } continuing { if true { } else if false { } else { } } }',
    pass: true
  },
  continuing_switch: {
    src: 'loop { if a == 4 { break; } continuing { switch 2 { default: { } } } }',
    pass: true
  },
  continuing_switch_break: {
    src: 'loop { if a == 4 { break; } continuing { switch 2 { default: { break; } } } }',
    pass: true
  },
  continuing_loop_nested_continuing: {
    src: 'loop { if a == 4 { break; } continuing { loop { if a == 4 { break; } continuing { } } } }',
    pass: true
  },
  continuing_inc: {
    src: 'loop { if a == 4 { break; } continuing { a += 1; } }',
    pass: true
  },
  continuing_dec: {
    src: 'loop { if a == 4 { break; } continuing { a -= 1; } }',
    pass: true
  },
  while: {
    src: 'while a < 4 { continuing { break if true; } }',
    pass: false
  },
  for: {
    src: 'for (;a < 4;) { continuing { break if true; } }',
    pass: false
  },
  switch_case: {
    src: 'switch(1) { default: { continuing { break if true; } } }',
    pass: false
  },
  switch: {
    src: 'switch(1) { continuing { break if true; } }',
    pass: false
  },
  continuing: {
    src: 'continuing { break if true; }',
    pass: false
  },
  return: {
    src: 'return continuing { break if true; }',
    pass: false
  },
  if_body: {
    src: 'if true { continuing { break if true; } }',
    pass: false
  },
  if: {
    src: 'if true { } continuing { break if true; } }',
    pass: false
  },
  if_else: {
    src: 'if true { } else { } continuing { break if true; } }',
    pass: false
  },
  continuing_continuing: {
    src: 'loop { if a == 4 { break; } continuing { continuing { break if true; } } }',
    pass: false
  },
  no_body: {
    src: 'loop { if a == 4 { break; } continuing }',
    pass: false
  },
  return_in_continue: {
    src: 'loop { if a == 4 { break; } continuing { return vec4f(2); } }',
    pass: false
  },
  return_if_nested_in_continue: {
    src: 'loop { if a == 4 { break; } continuing { if true { return vec4f(2); } } }',
    pass: false
  },
  return_for_nested_in_continue: {
    src: 'loop { if a == 4 { break; } continuing { for(;a < 4;) { return vec4f(2); } } }',
    pass: false
  }
};

g.test('placement').
desc('Test that continuing placement is validated correctly').
params((u) => u.combine('stmt', keysOf(kTests))).
fn((t) => {
  const code = `
fn b() -> i32 {
  return 1;
}

@fragment
fn frag() -> @location(0) vec4f {
  var a = 0;
  ${kTests[t.params.stmt].src}
  return vec4f(1);
}
    `;
  t.expectCompileResult(kTests[t.params.stmt].pass, code);
});