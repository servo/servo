/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for group`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  const_expr: {
    src: `const z = 5;
    const y = 2;
    @group(z + y)`,
    pass: true
  },
  override_expr: {
    src: `override z = 5;
    @group(z)`,
    pass: false
  },

  zero: {
    src: `@group(0)`,
    pass: true
  },
  one: {
    src: `@group(1)`,
    pass: true
  },
  comment: {
    src: `@/* comment */group(1)`,
    pass: true
  },
  split_line: {
    src: '@ \n group(1)',
    pass: true
  },
  trailing_comma: {
    src: `@group(1,)`,
    pass: true
  },
  int_literal: {
    src: `@group(1i)`,
    pass: true
  },
  uint_literal: {
    src: `@group(1u)`,
    pass: true
  },
  hex_literal: {
    src: `@group(0x1)`,
    pass: true
  },

  negative: {
    src: `@group(-1)`,
    pass: false
  },
  missing_value: {
    src: `@group()`,
    pass: false
  },
  missing_left_paren: {
    src: `@group 1)`,
    pass: false
  },
  missing_right_paren: {
    src: `@group(1`,
    pass: false
  },
  multiple_values: {
    src: `@group(1,2)`,
    pass: false
  },
  f32_val_literal: {
    src: `@group(1.0)`,
    pass: false
  },
  f32_val: {
    src: `@group(1f)`,
    pass: false
  },
  no_params: {
    src: `@group`,
    pass: false
  },
  misspelling: {
    src: `@agroup(1)`,
    pass: false
  },
  multi_group: {
    src: `@group(1) @group(1)`,
    pass: false
  }
};
g.test('group').
desc(`Test validation of group`).
params((u) => u.combine('attr', keysOf(kTests))).
fn((t) => {
  const code = `
${kTests[t.params.attr].src} @binding(1)
var<storage> a: i32;

@workgroup_size(1, 1, 1)
@compute fn main() {
  _ = a;
}`;
  t.expectCompileResult(kTests[t.params.attr].pass, code);
});

g.test('group_f16').
desc(`Test validation of group with f16`).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  const code = `
@group(1h) @binding(1)
var<storage> a: i32;

@workgroup_size(1, 1, 1)
@compute fn main() {
  _ = a;
}`;
  t.expectCompileResult(false, code);
});