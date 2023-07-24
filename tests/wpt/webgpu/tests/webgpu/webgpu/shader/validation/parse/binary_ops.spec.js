/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for binary ops`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  and_bool_literal_bool_literal: {
    src: `let a = true & true;`,
    pass: true,
  },
  and_bool_expr_bool_expr: {
    src: `let a = (1 == 2) & (3 == 4);`,
    pass: true,
  },
  and_bool_literal_bool_expr: {
    src: `let a = true & (1 == 2);`,
    pass: true,
  },
  and_bool_expr_bool_literal: {
    src: `let a = (1 == 2) & true;`,
    pass: true,
  },
  and_bool_literal_int_literal: {
    src: `let a = true & 1;`,
    pass: false,
  },
  and_int_literal_bool_literal: {
    src: `let a = 1 & true;`,
    pass: false,
  },
  and_bool_expr_int_literal: {
    src: `let a = (1 == 2) & 1;`,
    pass: false,
  },
  and_int_literal_bool_expr: {
    src: `let a = 1 & (1 == 2);`,
    pass: false,
  },

  or_bool_literal_bool_literal: {
    src: `let a = true | true;`,
    pass: true,
  },
  or_bool_expr_bool_expr: {
    src: `let a = (1 == 2) | (3 == 4);`,
    pass: true,
  },
  or_bool_literal_bool_expr: {
    src: `let a = true | (1 == 2);`,
    pass: true,
  },
  or_bool_expr_bool_literal: {
    src: `let a = (1 == 2) | true;`,
    pass: true,
  },
  or_bool_literal_int_literal: {
    src: `let a = true | 1;`,
    pass: false,
  },
  or_int_literal_bool_literal: {
    src: `let a = 1 | true;`,
    pass: false,
  },
  or_bool_expr_int_literal: {
    src: `let a = (1 == 2) | 1;`,
    pass: false,
  },
  or_int_literal_bool_expr: {
    src: `let a = 1 | (1 == 2);`,
    pass: false,
  },
};

g.test('all')
  .desc('Test that binary operators are validated correctly')
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
