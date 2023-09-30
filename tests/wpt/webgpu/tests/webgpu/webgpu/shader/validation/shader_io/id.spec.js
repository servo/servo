/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for id`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  zero: {
    src: `@id(0)`,
    pass: true,
  },
  one: {
    src: `@id(1)`,
    pass: true,
  },
  hex: {
    src: `@id(0x1)`,
    pass: true,
  },
  trailing_comma: {
    src: `@id(1,)`,
    pass: true,
  },
  i32: {
    src: `@id(1i)`,
    pass: true,
  },
  ui32: {
    src: `@id(1u)`,
    pass: true,
  },
  largest: {
    src: `@id(65535)`,
    pass: true,
  },
  newline: {
    src: '@\nid(1)',
    pass: true,
  },
  comment: {
    src: `@/* comment */id(1)`,
    pass: true,
  },
  const_expr: {
    src: `const z = 5;
      const y = 2;
      @id(z + y)`,
    pass: true,
  },

  misspelling: {
    src: `@aid(1)`,
    pass: false,
  },
  empty: {
    src: `@id()`,
    pass: false,
  },
  missing_left_paren: {
    src: `@id 1)`,
    pass: false,
  },
  missing_right_paren: {
    src: `@id(1`,
    pass: false,
  },
  multi_value: {
    src: `@id(1, 2)`,
    pass: false,
  },
  overide_expr: {
    src: `override z = 5;
      override y = 2;
      @id(z + y)`,
    pass: false,
  },
  f32_literal: {
    src: `@id(1.0)`,
    pass: false,
  },
  f32: {
    src: `@id(1f)`,
    pass: false,
  },
  negative: {
    src: `@id(-1)`,
    pass: false,
  },
  too_large: {
    src: `@id(65536)`,
    pass: false,
  },
  no_params: {
    src: `@id`,
    pass: false,
  },
  duplicate: {
    src: `@id(1) @id(1)`,
    pass: false,
  },
};

g.test('id')
  .desc(`Test validation of id`)
  .params(u => u.combine('attr', keysOf(kTests)))
  .fn(t => {
    const code = `
${kTests[t.params.attr].src}
override a = 4;

@workgroup_size(1, 1, 1)
@compute fn main() {}`;
    t.expectCompileResult(kTests[t.params.attr].pass, code);
  });

g.test('id_fp16')
  .desc(`Test validation of id with fp16`)
  .params(u => u.combine('ext', ['', 'h']))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(t => {
    const code = `
@id(1${t.params.ext})
override a = 4;

@workgroup_size(1, 1, 1)
@compute fn main() {}`;
    t.expectCompileResult(t.params.ext === '', code);
  });

g.test('id_struct_member')
  .desc(`Test validation of id with struct member`)
  .params(u => u.combine('id', ['@id(1) override', '@id(1)', '']))
  .fn(t => {
    const code = `
struct S {
  ${t.params.id} a: i32,
}

@workgroup_size(1, 1, 1)
@compute fn main() {}`;
    t.expectCompileResult(t.params.id === '', code);
  });

g.test('id_non_override')
  .desc(`Test validation of id with non-override`)
  .params(u => u.combine('type', ['var', 'const', 'override']))
  .fn(t => {
    const code = `
@id(1) ${t.params['type']} a = 4;

@workgroup_size(1, 1, 1)
@compute fn main() {}`;
    t.expectCompileResult(t.params['type'] === 'override', code);
  });

g.test('id_in_function')
  .desc(`Test validation of id inside a function`)
  .params(u => u.combine('id', ['@id(1)', '']))
  .fn(t => {
    const code = `
@workgroup_size(1, 1, 1)
@compute fn main() {
  ${t.params['id']} var a = 4;
}`;
    t.expectCompileResult(t.params['id'] === '', code);
  });
