/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for const_assert`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

/**
 * Builds a const_assert() statement.
 * @param expr the constant expression
 * @param scope module-scope or function-scope constant expression
 * @returns the WGSL code
 */
function buildStaticAssert(expr, scope) {
  const stmt = `const_assert (${expr});`;
  return scope === 'module' ? stmt : `fn f() { ${stmt} }`;
}










const kConditionCases = {
  any_false: { expr: `any(vec3(false, false, false))`, val: false },
  any_true: { expr: `any(vec3(false, true, false))`, val: true },
  binary_op_eq_const_false: { expr: `one + 5 == two`, val: false },
  binary_op_eq_const_true: { expr: `one + 1 == two`, val: true },
  const_eq_literal_float_false: { expr: `one == 0.0`, val: false },
  const_eq_literal_float_true: { expr: `one == 1.0`, val: true },
  const_eq_literal_int_false: { expr: `one == 10`, val: false },
  const_eq_literal_int_true: { expr: `one == 1`, val: true },
  literal_false: { expr: `false`, val: false },
  literal_not_false: { expr: `!false`, val: true },
  literal_not_true: { expr: `!true`, val: false },
  literal_true: { expr: `true`, val: true },
  min_max_false: { expr: `min(three, max(two, one)) == 3`, val: false },
  min_max_true: { expr: `min(three, max(two, one)) == 2`, val: true },
  variable_false: { expr: `is_false`, val: false },
  variable_not_false: { expr: `!is_false`, val: true },
  variable_not_true: { expr: `!is_true`, val: false },
  variable_true: { expr: `is_true`, val: true }
};

const kConditionConstants = `
const one = 1;
const two = 2;
const three = 3;
const is_true = true;
const is_false = false;
`;

g.test('constant_expression_no_assert').
desc(`Test that const_assert does not assert on a true conditional expression`).
params((u) =>
u.
combine('case', keysOf(kConditionCases)).
combine('scope', ['module', 'function']).
beginSubcases()
).
fn((t) => {
  const expr = kConditionCases[t.params.case].expr;
  const val = kConditionCases[t.params.case].val;
  t.expectCompileResult(
    true,
    kConditionConstants + buildStaticAssert(val ? expr : `!(${expr})`, t.params.scope)
  );
});

g.test('constant_expression_assert').
desc(`Test that const_assert does assert on a false conditional expression`).
params((u) =>
u.
combine('case', keysOf(kConditionCases)).
combine('scope', ['module', 'function']).
beginSubcases()
).
fn((t) => {
  const expr = kConditionCases[t.params.case].expr;
  const val = kConditionCases[t.params.case].val;
  t.expectCompileResult(
    false,
    kConditionConstants + buildStaticAssert(val ? `!(${expr})` : expr, t.params.scope)
  );
});

g.test('constant_expression_logical_or_no_assert').
desc(
  `Test that const_assert does not assert on a condition expression that contains a logical-or which evaluates to true`
).
params((u) =>
u.
combine('lhs', keysOf(kConditionCases)).
combine('rhs', keysOf(kConditionCases)).
combine('scope', ['module', 'function']).
beginSubcases()
).
fn((t) => {
  const expr = `(${kConditionCases[t.params.lhs].expr}) || (${
  kConditionCases[t.params.rhs].expr
  })`;
  const val = kConditionCases[t.params.lhs].val || kConditionCases[t.params.rhs].val;
  t.expectCompileResult(
    true,
    kConditionConstants + buildStaticAssert(val ? expr : `!(${expr})`, t.params.scope)
  );
});

g.test('constant_expression_logical_or_assert').
desc(
  `Test that const_assert does assert on a condition expression that contains a logical-or which evaluates to false`
).
params((u) =>
u.
combine('lhs', keysOf(kConditionCases)).
combine('rhs', keysOf(kConditionCases)).
combine('scope', ['module', 'function']).
beginSubcases()
).
fn((t) => {
  const expr = `(${kConditionCases[t.params.lhs].expr}) || (${
  kConditionCases[t.params.rhs].expr
  })`;
  const val = kConditionCases[t.params.lhs].val || kConditionCases[t.params.rhs].val;
  t.expectCompileResult(
    false,
    kConditionConstants + buildStaticAssert(val ? `!(${expr})` : expr, t.params.scope)
  );
});

g.test('constant_expression_logical_and_no_assert').
desc(
  `Test that const_assert does not assert on a condition expression that contains a logical-and which evaluates to true`
).
params((u) =>
u.
combine('lhs', keysOf(kConditionCases)).
combine('rhs', keysOf(kConditionCases)).
combine('scope', ['module', 'function']).
beginSubcases()
).
fn((t) => {
  const expr = `(${kConditionCases[t.params.lhs].expr}) && (${
  kConditionCases[t.params.rhs].expr
  })`;
  const val = kConditionCases[t.params.lhs].val && kConditionCases[t.params.rhs].val;
  t.expectCompileResult(
    true,
    kConditionConstants + buildStaticAssert(val ? expr : `!(${expr})`, t.params.scope)
  );
});

g.test('constant_expression_logical_and_assert').
desc(
  `Test that const_assert does assert on a condition expression that contains a logical-and which evaluates to false`
).
params((u) =>
u.
combine('lhs', keysOf(kConditionCases)).
combine('rhs', keysOf(kConditionCases)).
combine('scope', ['module', 'function']).
beginSubcases()
).
fn((t) => {
  const expr = `(${kConditionCases[t.params.lhs].expr}) && (${
  kConditionCases[t.params.rhs].expr
  })`;
  const val = kConditionCases[t.params.lhs].val && kConditionCases[t.params.rhs].val;
  t.expectCompileResult(
    false,
    kConditionConstants + buildStaticAssert(val ? `!(${expr})` : expr, t.params.scope)
  );
});

g.test('evaluation_stage').
desc(`Test that the const_assert expression must be a constant expression.`).
params((u) =>
u.
combine('scope', ['module', 'function']).
combine('stage', ['constant', 'override', 'runtime']).
beginSubcases()
).
fn((t) => {
  const staticAssert = buildStaticAssert('value', t.params.scope);
  switch (t.params.stage) {
    case 'constant':
      t.expectCompileResult(true, `const value = true;\n${staticAssert}`);
      break;
    case 'override':
      t.expectCompileResult(false, `override value = true;\n${staticAssert}`);
      break;
    case 'runtime':
      t.expectCompileResult(false, `var<private> value = true;\n${staticAssert}`);
      break;
  }
});