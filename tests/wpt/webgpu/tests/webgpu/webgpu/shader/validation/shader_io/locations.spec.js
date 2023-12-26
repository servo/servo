/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for entry point user-defined IO`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { generateShader } from './util.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValidLocationTypes = new Set([
'f16',
'f32',
'i32',
'u32',
'vec2<f32>',
'vec2<i32>',
'vec2<u32>',
'vec3<f32>',
'vec3<i32>',
'vec3<u32>',
'vec4<f32>',
'vec4<i32>',
'vec4<u32>',
'vec2h',
'vec2f',
'vec2i',
'vec2u',
'vec3h',
'vec3f',
'vec3i',
'vec3u',
'vec4h',
'vec4f',
'vec4i',
'vec4u',
'MyAlias']
);

const kInvalidLocationTypes = new Set([
'bool',
'vec2<bool>',
'vec3<bool>',
'vec4<bool>',
'mat2x2<f32>',
'mat2x3<f32>',
'mat2x4<f32>',
'mat3x2<f32>',
'mat3x3<f32>',
'mat3x4<f32>',
'mat4x2<f32>',
'mat4x3<f32>',
'mat4x4<f32>',
'mat2x2f',
'mat2x3f',
'mat2x4f',
'mat3x2f',
'mat3x3f',
'mat3x4f',
'mat4x2f',
'mat4x3f',
'mat4x4f',
'mat2x2h',
'mat2x3h',
'mat2x4h',
'mat3x2h',
'mat3x3h',
'mat3x4h',
'mat4x2h',
'mat4x3h',
'mat4x4h',
'array<f32, 12>',
'array<i32, 12>',
'array<u32, 12>',
'array<bool, 12>',
'atomic<i32>',
'atomic<u32>',
'MyStruct',
'texture_1d<i32>',
'texture_2d<f32>',
'texture_2d_array<i32>',
'texture_3d<f32>',
'texture_cube<u32>',
'texture_cube_array<i32>',
'texture_multisampled_2d<i32>',
'texture_external',
'texture_storage_1d<rgba8unorm, write>',
'texture_storage_2d<rg32float, write>',
'texture_storage_2d_array<r32float, write>',
'texture_storage_3d<r32float, write>',
'texture_depth_2d',
'texture_depth_2d_array',
'texture_depth_cube',
'texture_depth_cube_array',
'texture_depth_multisampled_2d',
'sampler',
'sampler_comparison']
);

g.test('stage_inout').
desc(`Test validation of user-defined IO stage and in/out usage`).
params((u) =>
u.
combine('use_struct', [true, false]).
combine('target_stage', ['vertex', 'fragment', 'compute']).
combine('target_io', ['in', 'out']).
beginSubcases()
).
fn((t) => {
  const code = generateShader({
    attribute: '@location(0)',
    type: 'f32',
    stage: t.params.target_stage,
    io: t.params.target_io,
    use_struct: t.params.use_struct
  });

  // Expect to fail for compute shaders or when used as a non-struct vertex output (since the
  // position built-in must also be specified).
  const expectation =
  t.params.target_stage === 'fragment' ||
  t.params.target_stage === 'vertex' && (t.params.target_io === 'in' || t.params.use_struct);
  t.expectCompileResult(expectation, code);
});

g.test('type').
desc(`Test validation of user-defined IO types`).
params((u) =>
u.
combine('use_struct', [true, false]).
combine('type', new Set([...kValidLocationTypes, ...kInvalidLocationTypes])).
beginSubcases()
).
beforeAllSubcases((t) => {
  if (
  t.params.type === 'f16' ||
  (t.params.type.startsWith('mat') || t.params.type.startsWith('vec')) &&
  t.params.type.endsWith('h'))
  {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  let code = '';

  if (
  t.params.type === 'f16' ||
  (t.params.type.startsWith('mat') || t.params.type.startsWith('vec')) &&
  t.params.type.endsWith('h'))
  {
    code += 'enable f16;\n';
  }

  if (t.params.type === 'MyStruct') {
    // Generate a struct that contains a valid type.
    code += `struct MyStruct {
                value : f32,
              }
              `;
  }
  if (t.params.type === 'MyAlias') {
    code += 'alias MyAlias = i32;\n';
  }

  code += generateShader({
    attribute: '@location(0) @interpolate(flat)',
    type: t.params.type,
    stage: 'fragment',
    io: 'in',
    use_struct: t.params.use_struct
  });

  t.expectCompileResult(kValidLocationTypes.has(t.params.type), code);
});

g.test('nesting').
desc(`Test validation of nested user-defined IO`).
params((u) =>
u.
combine('target_stage', ['vertex', 'fragment', '']).
combine('target_io', ['in', 'out']).
beginSubcases()
).
fn((t) => {
  let code = '';

  // Generate a struct that contains a valid type.
  code += `struct Inner {
               @location(0) value : f32,
             }
             struct Outer {
               inner : Inner,
             }
             `;

  code += generateShader({
    attribute: '',
    type: 'Outer',
    stage: t.params.target_stage,
    io: t.params.target_io,
    use_struct: false
  });

  // Expect to pass only if the struct is not used for entry point IO.
  t.expectCompileResult(t.params.target_stage === '', code);
});

g.test('duplicates').
desc(`Test that duplicated user-defined IO attributes are validated.`).
params((u) =>
u
// Place two @location(0) attributes onto the entry point function.
// The function:
// - has two non-struct parameters (`p1` and `p2`)
// - has two struct parameters each with two members (`s1{a,b}` and `s2{a,b}`)
// - returns a struct with two members (`ra` and `rb`)
// By default, all of these user-defined IO variables will have unique location attributes.
.combine('first', ['p1', 's1a', 's2a', 'ra']).
combine('second', ['p2', 's1b', 's2b', 'rb']).
beginSubcases()
).
fn((t) => {
  const p1 = t.params.first === 'p1' ? '0' : '1';
  const p2 = t.params.second === 'p2' ? '0' : '2';
  const s1a = t.params.first === 's1a' ? '0' : '3';
  const s1b = t.params.second === 's1b' ? '0' : '4';
  const s2a = t.params.first === 's2a' ? '0' : '5';
  const s2b = t.params.second === 's2b' ? '0' : '6';
  const ra = t.params.first === 'ra' ? '0' : '1';
  const rb = t.params.second === 'rb' ? '0' : '2';
  const code = `
    struct S1 {
      @location(${s1a}) a : f32,
      @location(${s1b}) b : f32,
    };
    struct S2 {
      @location(${s2a}) a : f32,
      @location(${s2b}) b : f32,
    };
    struct R {
      @location(${ra}) a : f32,
      @location(${rb}) b : f32,
    };
    @fragment
    fn main(@location(${p1}) p1 : f32,
            @location(${p2}) p2 : f32,
            s1 : S1,
            s2 : S2,
            ) -> R {
      return R();
    }
    `;

  // The test should fail if both @location(0) attributes are on the input parameters or
  // structures, or it they are both on the output struct. Otherwise it should pass.
  const firstIsRet = t.params.first === 'ra';
  const secondIsRet = t.params.second === 'rb';
  const expectation = firstIsRet !== secondIsRet;
  t.expectCompileResult(expectation, code);
});

const kValidationTests = {
  zero: {
    src: `@location(0)`,
    pass: true
  },
  one: {
    src: `@location(1)`,
    pass: true
  },
  extra_comma: {
    src: `@location(1,)`,
    pass: true
  },
  i32: {
    src: `@location(1i)`,
    pass: true
  },
  u32: {
    src: `@location(1u)`,
    pass: true
  },
  hex: {
    src: `@location(0x1)`,
    pass: true
  },
  const_expr: {
    src: `@location(a + b)`,
    pass: true
  },
  max: {
    src: `@location(2147483647)`,
    pass: true
  },
  newline: {
    src: '@\nlocation(1)',
    pass: true
  },
  comment: {
    src: `@/* comment */location(1)`,
    pass: true
  },

  misspelling: {
    src: `@mlocation(1)`,
    pass: false
  },
  no_parens: {
    src: `@location`,
    pass: false
  },
  empty_params: {
    src: `@location()`,
    pass: false
  },
  missing_left_paren: {
    src: `@location 1)`,
    pass: false
  },
  missing_right_paren: {
    src: `@location(1`,
    pass: false
  },
  extra_params: {
    src: `@location(1, 2)`,
    pass: false
  },
  f32: {
    src: `@location(1f)`,
    pass: false
  },
  f32_literal: {
    src: `@location(1.0)`,
    pass: false
  },
  negative: {
    src: `@location(-1)`,
    pass: false
  },
  override_expr: {
    src: `@location(z + y)`,
    pass: false
  },
  vec: {
    src: `@location(vec2(1,1))`,
    pass: false
  }
};
g.test('validation').
desc(`Test validation of location`).
params((u) => u.combine('attr', keysOf(kValidationTests))).
fn((t) => {
  const code = `
const a = 5;
const b = 6;
override z = 7;
override y = 8;

@vertex fn main(
  ${kValidationTests[t.params.attr].src} res: f32
) -> @builtin(position) vec4f {
  return vec4f(0);
}`;
  t.expectCompileResult(kValidationTests[t.params.attr].pass, code);
});

g.test('location_fp16').
desc(`Test validation of location with fp16`).
params((u) => u.combine('ext', ['', 'h'])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => {
  const code = `

@vertex fn main(
  @location(1${t.params.ext}) res: f32
) -> @builtin(position) vec4f {
  return vec4f();
}`;
  t.expectCompileResult(t.params.ext === '', code);
});