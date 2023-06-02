/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for const_assert`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

/**
 * Builds a const_assert() statement, which checks that @p expr is equal to @p expect_true.
 * @param expect_true true if @p expr should evaluate to true
 * @param expr the constant expression
 * @param scope module-scope or function-scope constant expression
 * @returns the WGSL code
 */
function buildStaticAssert(expect_true, expr, scope) {
  const stmt = expect_true ? `const_assert ${expr};` : `const_assert !(${expr});`;
  return scope === 'module' ? stmt : `fn f() { ${stmt} }`;
}

const kConditionCases = {
  true_literal: `true`,
  not_false: `!false`,
  const_eq_literal_int: `one == 1`,
  const_eq_literal_float: `one == 1.0`,
  binary_op_eq_const: `one+1 == two`,
  any: `any(vec3(false, true, false))`,
  min_max: `min(three, max(two, one)) == 2`,
};

g.test('constant_expression')
  .desc(`Test that const_assert validates the condition expression.`)
  .params(u =>
    u
      .combine('case', keysOf(kConditionCases))
      .combine('scope', ['module', 'function'])
      .beginSubcases()
  )
  .fn(t => {
    const constants = `
const one = 1;
const two = 2;
const three = 2;
`;
    const expr = kConditionCases[t.params.case];
    t.expectCompileResult(true, constants + buildStaticAssert(true, expr, t.params.scope));
    t.expectCompileResult(false, constants + buildStaticAssert(false, expr, t.params.scope));
  });

g.test('evaluation_stage')
  .desc(`Test that the const_assert expression must be a constant expression.`)
  .params(u =>
    u
      .combine('scope', ['module', 'function'])
      .combine('stage', ['constant', 'override', 'runtime'])
      .beginSubcases()
  )
  .fn(t => {
    const staticAssert = buildStaticAssert(true, 'value', t.params.scope);
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
