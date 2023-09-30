/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation negative tests for bitcast builtins.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { kBit } from '../../../../../util/constants.js';
import { linearRange } from '../../../../../util/math.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// A VectorCase specifies the number of components a vector type has,
// and which component will have a bad value.
// Use width = 1 to indicate a scalar.

const kVectorCases = {
  v1_b0: { width: 1, badIndex: 0 },
  v2_b0: { width: 2, badIndex: 0 },
  v2_b1: { width: 2, badIndex: 1 },
  v3_b0: { width: 3, badIndex: 0 },
  v3_b1: { width: 3, badIndex: 1 },
  v3_b2: { width: 3, badIndex: 2 },
  v4_b0: { width: 4, badIndex: 0 },
  v4_b1: { width: 4, badIndex: 1 },
  v4_b2: { width: 4, badIndex: 2 },
  v4_b3: { width: 4, badIndex: 3 },
};

const numNaNs = 4;
const f32InfAndNaNInU32 = [
  // Cover NaNs evenly in integer space.
  // The positive NaN with the lowest integer representation is the integer
  // for infinity, plus one.
  // The positive NaN with the highest integer representation is i32.max (!)
  ...linearRange(kBit.f32.positive.infinity + 1, kBit.i32.positive.max, numNaNs),
  // The negative NaN with the lowest integer representation is the integer
  // for negative infinity, plus one.
  // The negative NaN with the highest integer representation is u32.max (!)
  ...linearRange(kBit.f32.negative.infinity + 1, kBit.u32.max, numNaNs),
  kBit.f32.positive.infinity,
  kBit.f32.negative.infinity,
];

g.test('bad_const_to_f32')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
It is a shader-creation error if any const-expression of floating-point type evaluates to NaN or infinity.
`
  )
  .params(u =>
    u
      .combine('fromScalarType', ['i32', 'u32'])
      .combine('vectorize', keysOf(kVectorCases))
      .beginSubcases()
      .combine('bitBadValue', [...f32InfAndNaNInU32])
  )
  .fn(t => {
    // For scalar cases, generate code like:
    //  const f = bitcast<f32>(i32(u32(0x7f800000)));
    // For vector cases, generate code where one component is bad. In this case
    // width=4 and badIndex=2
    //  const f = bitcast<vec4f>(vec4<32>(0,0,i32(u32(0x7f800000)),0));
    const vectorize = kVectorCases[t.params.vectorize];
    const width = vectorize.width;
    const badIndex = vectorize.badIndex;
    const badScalar = `${t.params.fromScalarType}(u32(${t.params.bitBadValue}))`;
    const destType = width === 1 ? 'f32' : `vec${width}f`;
    const srcType =
      width === 1 ? t.params.fromScalarType : `vec${width}<${t.params.fromScalarType}>`;
    const components = [...Array(width).keys()]
      .map(i => (i === badIndex ? badScalar : '0'))
      .join(',');
    const code = `const f = bitcast<${destType}>(${srcType}(${components}));`;
    t.expectCompileResult(false, code);
  });

const f32_matrix_types = [2, 3, 4]
  .map(i => [2, 3, 4].map(j => `mat${i}x${j}f`))
  .reduce((a, c) => a.concat(c), []);
const bool_types = ['bool', ...[2, 3, 4].map(i => `vec${i}<bool>`)];

g.test('bad_type_constructible')
  .specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin')
  .desc(
    `
Bitcast only applies to concrete numeric scalar or concrete numeric vector.
Test constructible types.
`
  )
  .params(u =>
    u
      .combine('type', [...f32_matrix_types, ...bool_types, 'array<i32,2>', 'S'])
      .combine('direction', ['to', 'from'])
  )
  .fn(t => {
    const T = t.params.type;
    const preamble = T === 'S' ? 'struct S { a:i32 } ' : '';
    // Create a value of type T using zero-construction: T().
    const srcVal = t.params.direction === 'to' ? '0' : `${T}()`;
    const destType = t.params.direction === 'to' ? T : 'i32';
    const code = preamble + `const x = bitcast<${destType}>(${srcVal});`;
    t.expectCompileResult(false, code);
  });

g.test('bad_type_nonconstructible')
  .specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin')
  .desc(
    `
Bitcast only applies to concrete numeric scalar or concrete numeric vector.
Test non-constructible types.
`
  )
  .params(u => u.combine('var', ['s', 't', 'b', 'p']).combine('direction', ['to', 'from']))
  .fn(t => {
    const typeOf = {
      s: 'sampler',
      t: 'texture_depth_2d',
      b: 'array<i32>',
      p: 'ptr<private,i32>',
    };
    const srcVal = t.params.direction === 'to' ? '0' : t.params.var;
    const destType = t.params.direction === 'to' ? typeOf[t.params.var] : 'i32';
    const code = `
    @group(0) @binding(0) var s: sampler;
    @group(0) @binding(1) var t: texture_depth_2d;
    @group(0) @binding(2) var<storage> b: array<i32>;
    var<private> v: i32;
    @compute @workgroup_size(1)
    fn main() {
      let p = &v;
      let x = bitcast<${destType}>(${srcVal});
    }
    `;
    t.expectCompileResult(false, code);
  });

g.test('bad_to_vec3h')
  .specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin')
  .desc(
    `
Can't cast numeric type to vec3<f16> because it is 48 bits wide
and no other type is that size.
`
  )
  .params(u =>
    u
      .combine('other_type', [
        'bool',
        'u32',
        'i32',
        'f32',
        'vec2<bool>',
        'vec3<bool>',
        'vec4<bool>',
        'vec2u',
        'vec3u',
        'vec4u',
        'vec2i',
        'vec3i',
        'vec4i',
        'vec2f',
        'vec3f',
        'vec4f',
        'vec2h',
        'vec4h',
      ])
      .combine('direction', ['to', 'from'])
      .combine('type', ['vec3<f16>', 'vec3h'])
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(t => {
    const src_type = t.params.direction === 'to' ? t.params.type : t.params.other_type;
    const dst_type = t.params.direction === 'from' ? t.params.type : t.params.other_type;
    const code = `
enable f16;
@fragment
fn main() {
  var src : ${src_type};
  let dst = bitcast<${dst_type}>(src);
}`;
    t.expectCompileResult(false, code);
  });

g.test('bad_to_f16')
  .specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin')
  .desc(
    `
Can't cast non-16-bit types to f16 because it is 16 bits wide
and no other type is that size.
`
  )
  .params(u =>
    u
      .combine('other_type', [
        'bool',
        'u32',
        'i32',
        'f32',
        'vec2<bool>',
        'vec3<bool>',
        'vec4<bool>',
        'vec2u',
        'vec3u',
        'vec4u',
        'vec2i',
        'vec3i',
        'vec4i',
        'vec2f',
        'vec3f',
        'vec4f',
        'vec2h',
        'vec3h',
        'vec4h',
      ])
      .combine('direction', ['to', 'from'])
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(t => {
    const src_type = t.params.direction === 'to' ? 'f16' : t.params.other_type;
    const dst_type = t.params.direction === 'from' ? 'f16' : t.params.other_type;
    const code = `
enable f16;
@fragment
fn main() {
  var src : ${src_type};
  let dst = bitcast<${dst_type}>(src);
}`;
    t.expectCompileResult(false, code);
  });

g.test('valid_vec2h')
  .specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin')
  .desc(`Check valid vec2<f16> bitcasts`)
  .params(u =>
    u
      .combine('other_type', ['u32', 'i32', 'f32'])
      .combine('type', ['vec2<f16>', 'vec2h'])
      .combine('direction', ['to', 'from'])
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(t => {
    const src_type = t.params.direction === 'to' ? t.params.type : t.params.other_type;
    const dst_type = t.params.direction === 'from' ? t.params.type : t.params.other_type;
    const code = `
enable f16;
@fragment
fn main() {
  var src : ${src_type};
  let dst = bitcast<${dst_type}>(src);
}`;
    t.expectCompileResult(true, code);
  });

g.test('valid_vec4h')
  .specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin')
  .desc(`Check valid vec2<f16> bitcasts`)
  .params(u =>
    u
      .combine('other_type', ['vec2<u32>', 'vec2u', 'vec2<i32>', 'vec2i', 'vec2<f32>', 'vec2f'])
      .combine('type', ['vec4<f16>', 'vec4h'])
      .combine('direction', ['to', 'from'])
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(t => {
    const src_type = t.params.direction === 'to' ? t.params.type : t.params.other_type;
    const dst_type = t.params.direction === 'from' ? t.params.type : t.params.other_type;
    const code = `
enable f16;
@fragment
fn main() {
  var src : ${src_type};
  let dst = bitcast<${dst_type}>(src);
}`;
    t.expectCompileResult(true, code);
  });
