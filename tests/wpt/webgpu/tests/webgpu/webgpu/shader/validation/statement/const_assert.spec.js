/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Parser validation tests for const_assert`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kCases = {
  no_parentheses: { code: `const_assert true;`, pass: true },
  left_parenthesis_only: { code: `const_assert(true;`, pass: false },
  right_parenthesis_only: { code: `const_assert true);`, pass: false },
  both_parentheses: { code: `const_assert(true);`, pass: true },
  condition_on_newline: {
    code: `const_assert
true;`,
    pass: true
  },
  multiline_with_parentheses: {
    code: `const_assert
(
  true
);`,
    pass: true
  },
  invalid_expression: { code: `const_assert(1!2);`, pass: false },
  no_condition_no_parentheses: { code: `const_assert;`, pass: false },
  no_condition_with_parentheses: { code: `const_assert();`, pass: false },
  not_a_boolean: { code: `const_assert 42;`, pass: false }
};

g.test('parse').
desc(`Tests that the const_assert statement parses correctly.`).
params((u) => u.combine('case', keysOf(kCases))).
fn((t) => {
  const c = kCases[t.params.case];
  t.expectCompileResult(c.pass, c.code);
});