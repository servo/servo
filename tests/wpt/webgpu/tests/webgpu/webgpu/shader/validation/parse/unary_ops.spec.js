/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for unary ops`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  not_bool_literal: {
    src: 'let a = !true;',
    pass: true,
  },
  not_bool_expr: {
    src: `let a = !(1 == 2);`,
    pass: true,
  },
  not_not_bool_literal: {
    src: 'let a = !!true;',
    pass: true,
  },
  not_not_bool_expr: {
    src: `let a = !!(1 == 2);`,
    pass: true,
  },
  not_int_literal: {
    src: `let a = !42;`,
    pass: false,
  },
  not_int_expr: {
    src: `let a = !(40 + 2);`,
    pass: false,
  },
};

g.test('all')
  .desc('Test that unary operators are validated correctly')
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
