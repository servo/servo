/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for declarations in compound statements.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// 9.1 Compound Statements
//    When a declaration is one of those statements, its identifier is in scope from
//    the start of the next statement until the end of the compound statement.
//
// Enumerating cases: Consider a declaration X inside a compound statement.
// The X declaration should be tested with potential uses and potentially
// conflicting declarations in positions [a, b, c, d, e], in the following:
//     a { b; X; c; { d; } } e;

const kConflictTests = {
  a: {
    src: 'let x = 1; { let x = 1; }',
    pass: true
  },
  bc: {
    src: '{let x = 1; let x = 1; }',
    pass: false
  },
  d: {
    src: '{let x = 1; { let x = 1; }}',
    pass: true
  },
  e: {
    src: '{let x = 1; } let x = 1;',
    pass: true
  }
};

g.test('decl_conflict').
desc(
  'Test a potentially conflicting declaration relative to a declaration in a compound statement'
).
params((u) => u.combine('case', keysOf(kConflictTests))).
fn((t) => {
  const wgsl = `
@vertex fn vtx() -> @builtin(position) vec4f {
  ${kConflictTests[t.params.case].src}
  return vec4f(1);
}`;
  t.expectCompileResult(kConflictTests[t.params.case].pass, wgsl);
});

const kUseTests = {
  a: {
    src: 'let y = x; { let x = 1; }',
    pass: false // not visible
  },
  b: {
    src: '{ let y = x; let x = 1; }',
    pass: false // not visible
  },
  self: {
    src: '{ let x = (x);}',
    pass: false // not visible
  },
  c_yes: {
    src: '{ const x = 1; const_assert x == 1; }',
    pass: true
  },
  c_no: {
    src: '{ const x = 1; const_assert x == 2; }',
    pass: false
  },
  d_yes: {
    src: '{ const x = 1; { const_assert x == 1; }}',
    pass: true
  },
  d_no: {
    src: '{ const x = 1; { const_assert x == 2; }}',
    pass: false
  },
  e: {
    src: '{ const x = 1; } let y = x;',
    pass: false // not visible
  }
};

g.test('decl_use').
desc('Test a use of a declaration in a compound statement').
params((u) => u.combine('case', keysOf(kUseTests))).
fn((t) => {
  const wgsl = `
@vertex fn vtx() -> @builtin(position) vec4f {
  ${kUseTests[t.params.case].src}
  return vec4f(1);
}`;
  t.expectCompileResult(kUseTests[t.params.case].pass, wgsl);
});