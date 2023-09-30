/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Positive and negative validation tests for variable and const.

TODO: Find a better way to test arrays than using a single arbitrary size. [1]
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTestTypes = [
  'f32',
  'i32',
  'u32',
  'bool',
  'vec2<f32>',
  'vec2<i32>',
  'vec2<u32>',
  'vec2<bool>',
  'vec3<f32>',
  'vec3<i32>',
  'vec3<u32>',
  'vec3<bool>',
  'vec4<f32>',
  'vec4<i32>',
  'vec4<u32>',
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
  // [1]: 12 is a random number here. find a solution to replace it.
  'array<f32, 12>',
  'array<i32, 12>',
  'array<u32, 12>',
  'array<bool, 12>',
];

g.test('initializer_type')
  .desc(
    `
  If present, the initializer's type must match the store type of the variable.
  Testing scalars, vectors, and matrices of every dimension and type.
  TODO: add test for: structs - arrays of vectors and matrices - arrays of different length
`
  )
  .params(u =>
    u
      .combine('variableOrConstant', ['var', 'let'])
      .beginSubcases()
      .combine('lhsType', kTestTypes)
      .combine('rhsType', kTestTypes)
  )
  .fn(t => {
    const { variableOrConstant, lhsType, rhsType } = t.params;

    const code = `
      @fragment
      fn main() {
        ${variableOrConstant} a : ${lhsType} = ${rhsType}();
      }
    `;

    const expectation = lhsType === rhsType;
    t.expectCompileResult(expectation, code);
  });

g.test('var_access_mode_bad_other_template_contents')
  .desc(
    'A variable declaration with explicit access mode with varying other template list contents'
  )
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls')
  .params(u =>
    u
      .combine('accessMode', ['read', 'read_write'])
      .combine('prefix', ['storage,', '', ','])
      .combine('suffix', [',storage', ',read', ',', ''])
  )
  .fn(t => {
    const prog = `@group(0) @binding(0)
                  var<${t.params.prefix}${t.params.accessMode}${t.params.suffix}> x: i32;`;
    const ok = t.params.prefix === 'storage,' && t.params.suffix === '';
    t.expectCompileResult(ok, prog);
  });

g.test('var_access_mode_bad_template_delim')
  .desc('A variable declaration has explicit access mode with varying template list delimiters')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls')
  .params(u =>
    u
      .combine('accessMode', ['read', 'read_write'])
      .combine('prefix', ['', '<', '>', ','])
      .combine('suffix', ['', '<', '>', ','])
  )
  .fn(t => {
    const prog = `@group(0) @binding(0)
                  var ${t.params.prefix}storage,${t.params.accessMode}${t.params.suffix} x: i32;`;
    const ok = t.params.prefix === '<' && t.params.suffix === '>';
    t.expectCompileResult(ok, prog);
  });
