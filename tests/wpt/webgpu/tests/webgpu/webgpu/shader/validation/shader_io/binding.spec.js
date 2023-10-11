/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for binding`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  const_expr: {
    src: `const z = 5;
    const y = 2;
    @binding(z + y)`,
    pass: true,
  },
  override_expr: {
    src: `override z = 5;
    @binding(z)`,
    pass: false,
  },

  zero: {
    src: `@binding(0)`,
    pass: true,
  },
  one: {
    src: `@binding(1)`,
    pass: true,
  },
  comment: {
    src: `@/* comment */binding(1)`,
    pass: true,
  },
  split_line: {
    src: '@ \n binding(1)',
    pass: true,
  },
  trailing_comma: {
    src: `@binding(1,)`,
    pass: true,
  },
  int_literal: {
    src: `@binding(1i)`,
    pass: true,
  },
  uint_literal: {
    src: `@binding(1u)`,
    pass: true,
  },
  hex_literal: {
    src: `@binding(0x1)`,
    pass: true,
  },

  negative: {
    src: `@binding(-1)`,
    pass: false,
  },
  missing_value: {
    src: `@binding()`,
    pass: false,
  },
  missing_left_paren: {
    src: `@binding 1)`,
    pass: false,
  },
  missing_right_paren: {
    src: `@binding(1`,
    pass: false,
  },
  multiple_values: {
    src: `@binding(1,2)`,
    pass: false,
  },
  f32_val_literal: {
    src: `@binding(1.0)`,
    pass: false,
  },
  f32_val: {
    src: `@binding(1f)`,
    pass: false,
  },
  no_params: {
    src: `@binding`,
    pass: false,
  },
  misspelling: {
    src: `@abinding(1)`,
    pass: false,
  },
  multi_binding: {
    src: `@binding(1) @binding(1)`,
    pass: false,
  },
};
g.test('binding')
  .desc(`Test validation of binding`)
  .params(u => u.combine('attr', keysOf(kTests)))
  .fn(t => {
    const code = `
${kTests[t.params.attr].src} @group(1)
var<storage> a: i32;

@workgroup_size(1, 1, 1)
@compute fn main() {
  _ = a;
}`;
    t.expectCompileResult(kTests[t.params.attr].pass, code);
  });

g.test('binding_f16')
  .desc(`Test validation of binding with f16`)
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(t => {
    const code = `
@group(1) @binding(1h)
var<storage> a: i32;

@workgroup_size(1, 1, 1)
@compute fn main() {
  _ = a;
}`;
    t.expectCompileResult(false, code);
  });

g.test('binding_without_group')
  .desc(`Test validation of binding without group`)
  .fn(t => {
    const code = `
@binding(1)
var<storage> a: i32;

@workgroup_size(1, 1, 1)
@compute fn main() {
  _ = a;
}`;
    t.expectCompileResult(false, code);
  });
