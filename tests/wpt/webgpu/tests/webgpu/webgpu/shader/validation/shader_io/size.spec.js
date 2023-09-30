/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for size`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kSizeTests = {
  valid: {
    src: `@size(4)`,
    pass: true,
  },
  non_align_size: {
    src: `@size(5)`,
    pass: true,
  },
  i32: {
    src: `@size(4i)`,
    pass: true,
  },
  u32: {
    src: `@size(4u)`,
    pass: true,
  },
  constant: {
    src: `@size(z)`,
    pass: true,
  },
  trailing_comma: {
    src: `@size(4,)`,
    pass: true,
  },
  hex: {
    src: `@size(0x4)`,
    pass: true,
  },
  whitespace: {
    src: '@\nsize(4)',
    pass: true,
  },
  comment: {
    src: `@/* comment */size(4)`,
    pass: true,
  },
  large: {
    src: `@size(2147483647)`,
    pass: true,
  },

  misspelling: {
    src: `@msize(4)`,
    pass: false,
  },
  no_value: {
    src: `@size()`,
    pass: false,
  },
  missing_left_paren: {
    src: `@size 4)`,
    pass: false,
  },
  missing_right_paren: {
    src: `@size(4`,
    pass: false,
  },
  missing_parens: {
    src: `@size`,
    pass: false,
  },
  multiple_values: {
    src: `@size(4, 8)`,
    pass: false,
  },
  override: {
    src: `@size(over)`,
    pass: false,
  },
  zero: {
    src: `@size(0)`,
    pass: false,
  },
  negative: {
    src: `@size(-4)`,
    pass: false,
  },
  f32_literal: {
    src: `@size(4.0)`,
    pass: false,
  },
  f32: {
    src: `@size(4f)`,
    pass: false,
  },
  duplicate: {
    src: `@size(4) @size(8)`,
    pass: false,
  },
  too_small: {
    src: `@size(1)`,
    pass: false,
  },
};

g.test('size')
  .desc(`Test validation of ize`)
  .params(u => u.combine('attr', keysOf(kSizeTests)))
  .fn(t => {
    const code = `
override over: i32 = 4;
const z: i32 = 4;

struct S {
  ${kSizeTests[t.params.attr].src} a: f32,
};
@group(0) @binding(0)
var<storage> a: S;

@workgroup_size(1)
@compute fn main() {
  _ = a;
}`;
    t.expectCompileResult(kSizeTests[t.params.attr].pass, code);
  });

g.test('size_fp16')
  .desc(`Test validation of size with fp16`)
  .params(u => u.combine('ext', ['', 'h']))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(t => {
    const code = `
struct S {
  @size(4${t.params.ext}) a: f32,
}
@group(0) @binding(0)
var<storage> a: S;

@workgroup_size(1)
@compute fn main() {
  _ = a;
}`;
    t.expectCompileResult(t.params.ext === '', code);
  });

const kNonStructTests = {
  control: {
    mod_src: ``,
    func_src: ``,
    size: 0,
    pass: true,
  },
  struct: {
    mod_src: `struct S { a: f32 }`,
    func_src: ``,
    size: 4,
    pass: false,
  },
  constant: {
    mod_src: `const a: f32 = 4.0;`,
    func_src: ``,
    size: 4,
    pass: false,
  },
  vec: {
    mod_src: ``,
    func_src: `vec4<f32>`,
    size: 16,
    pass: false,
  },
  mat: {
    mod_src: ``,
    func_src: `mat4x4<f32>`,
    size: 64,
    pass: false,
  },
  array: {
    mod_src: ``,
    func_src: `array<f32, 4>`,
    size: 16,
    pass: false,
  },
  scalar: {
    mod_src: ``,
    func_src: `f32`,
    size: 4,
    pass: false,
  },
};

g.test('size_non_struct')
  .desc(`Test validation of size outside of a struct`)
  .params(u => u.combine('attr', keysOf(kNonStructTests)))
  .fn(t => {
    const data = kNonStructTests[t.params.attr];
    let code = '';
    if (data.mod_src !== '') {
      code += `@size(${data.size}) ${data.mod_src}`;
    }

    code += `
@workgroup_size(1)
@compute fn main() {
`;
    if (data.func_src !== '') {
      code += `@size(${data.size}) var a: ${data.func_src};`;
    }
    code += '}';

    t.expectCompileResult(data.pass, code);
  });
