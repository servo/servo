/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for continue`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  continue: {
    src: 'continue;',
    pass: false
  },
  compound_continue: {
    src: '{ continue; }',
    pass: false
  },
  loop_continue: {
    src: 'loop { if false { break; } continue; }',
    pass: true
  },
  while_continue: {
    src: 'while true { continue; }',
    pass: true
  },
  for_continue: {
    src: 'for (;true;) { continue; }',
    pass: true
  },
  continuing_continue: {
    src: 'loop { continuing { continue; } }',
    pass: false
  },
  continuing_nested_loop_continue: {
    src: 'loop { if false { break; } continuing { loop { if false { break; } continue; } } }',
    pass: true
  },
  if_continue: {
    src: 'if true { continue; }',
    pass: false
  },
  nested_if_continue: {
    src: 'while true { if true { continue; } }',
    pass: true
  },
  switch_case_continue: {
    src: 'switch(1) { default: { continue; } }',
    pass: false
  },
  nested_switch_case_continue: {
    src: 'while true { switch(1) { default: { continue; } } }',
    pass: true
  },
  return_continue: {
    src: 'return continue;',
    pass: false
  },
  loop_continue_after_decl_used_in_continuing: {
    src: 'loop { let cond = false; continue; continuing { break if cond; } }',
    pass: true
  },
  loop_continue_before_decl_used_in_continuing: {
    src: 'loop { continue; let cond = false; continuing { break if cond; } }',
    pass: false
  },
  loop_continue_before_decl_not_used_in_continuing: {
    src: 'loop { continue; let cond = false; continuing { break if false; } }',
    pass: true
  },
  loop_nested_continue_before_decl_used_in_continuing: {
    src: 'loop { if false { continue; } let cond = false; continuing { break if cond; } }',
    pass: false
  },
  loop_continue_expression: {
    src: 'loop { if false { break; } continue true; }',
    pass: false
  },
  for_init_continue: {
    src: 'for (continue;;) { break; }',
    pass: false
  },
  for_condition_continue: {
    src: 'for (;continue;) { break; }',
    pass: false
  },
  for_continue_continue: {
    src: 'for (;;continue) { break; }',
    pass: false
  }
};

g.test('placement').
desc('Test that continue placement is validated correctly').
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

g.test('module_scope').
desc('Test that continue is not valid at module-scope.').
fn((t) => {
  const code = `
continue;
    `;
  t.expectCompileResult(false, code);
});