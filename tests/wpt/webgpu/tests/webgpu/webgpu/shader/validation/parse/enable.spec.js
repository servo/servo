/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Parser validation tests for enable`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kCases = {
  f16: { code: `enable f16;`, pass: true },
  decl_before: {
    code: `alias i = i32;
enable f16;`,
    pass: false
  },
  after_decl: {
    code: `enable f16;
alias i = i32;`,
    pass: true
  },
  const_assert_before: {
    code: `const_assert 1 == 1;
enable f16;`,
    pass: false
  },
  const_assert_after: {
    code: `enable f16;
const_assert 1 == 1;`,
    pass: true
  },
  embedded_comment: {
    code: `/* comment

*/enable f16;`,
    pass: true
  },
  parens: {
    code: `enable(f16);`,
    pass: false
  },
  multi_line: {
    code: `enable
f16;`,
    pass: true
  },
  multiple_enables: {
    code: `enable f16;
enable f16;`,
    pass: true
  },
  multipe_entries: {
    code: `enable f16, f16, f16;`,
    pass: true
  },
  unknown: {
    code: `enable unknown;`,
    pass: false
  }
};

g.test('enable').
desc(`Tests that enables are validated correctly`).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
params((u) => u.combine('case', keysOf(kCases))).
fn((t) => {
  const c = kCases[t.params.case];
  t.expectCompileResult(c.pass, c.code);
});