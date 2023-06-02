/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'modf' builtin function

T is f32 or f16
@const fn modf(e:T) -> result_struct
Splits |e| into fractional and whole number parts.
The whole part is (|e| % 1.0), and the fractional part is |e| minus the whole part.
Returns the result_struct for the given type.

S is f32 or f16
T is vecN<S>
@const fn modf(e:T) -> result_struct
Splits the components of |e| into fractional and whole number parts.
The |i|'th component of the whole and fractional parts equal the whole and fractional parts of modf(e[i]).
Returns the result_struct for the given type.

`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { toVector, TypeF32, TypeVec } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range, vectorF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, basicExpressionBuilder, run } from '../../expression.js';

export const g = makeTestGroup(GPUTest);

/** @returns an ShaderBuilder that evaluates modf and returns .whole from the result structure */
function wholeBuilder() {
  return basicExpressionBuilder(value => `modf(${value}).whole`);
}

/** @returns an ShaderBuilder that evaluates modf and returns .fract from the result structure */
function fractBuilder() {
  return basicExpressionBuilder(value => `modf(${value}).fract`);
}

/** @returns a fract Case for a given vector input */
function makeVectorCaseFract(kind, v) {
  const fp = FP[kind];
  v = v.map(fp.quantize);
  const fs = v.map(e => {
    return fp.modfInterval(e).fract;
  });

  return { input: toVector(v, fp.scalarBuilder), expected: fs };
}

/** @returns a whole Case for a given vector input */
function makeVectorCaseWhole(kind, v) {
  const fp = FP[kind];
  v = v.map(fp.quantize);
  const ws = v.map(e => {
    return fp.modfInterval(e).whole;
  });

  return { input: toVector(v, fp.scalarBuilder), expected: ws };
}

export const d = makeCaseCache('modf', {
  f32_fract: () => {
    const makeCase = n => {
      n = FP.f32.quantize(n);
      return { input: FP.f32.scalarBuilder(n), expected: FP.f32.modfInterval(n).fract };
    };
    return fullF32Range().map(makeCase);
  },
  f32_whole: () => {
    const makeCase = n => {
      n = FP.f32.quantize(n);
      return { input: FP.f32.scalarBuilder(n), expected: FP.f32.modfInterval(n).whole };
    };
    return fullF32Range().map(makeCase);
  },
  f32_vec2_fract: () => {
    return vectorF32Range(2).map(makeVectorCaseFract.bind(null, 'f32'));
  },
  f32_vec2_whole: () => {
    return vectorF32Range(2).map(makeVectorCaseWhole.bind(null, 'f32'));
  },
  f32_vec3_fract: () => {
    return vectorF32Range(3).map(makeVectorCaseFract.bind(null, 'f32'));
  },
  f32_vec3_whole: () => {
    return vectorF32Range(3).map(makeVectorCaseWhole.bind(null, 'f32'));
  },
  f32_vec4_fract: () => {
    return vectorF32Range(4).map(makeVectorCaseFract.bind(null, 'f32'));
  },
  f32_vec4_whole: () => {
    return vectorF32Range(4).map(makeVectorCaseWhole.bind(null, 'f32'));
  },
});

g.test('f32_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is f32

struct __modf_result_f32 {
  fract : f32, // fractional part
  whole : f32  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_fract');
    await run(t, fractBuilder(), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f32_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is f32

struct __modf_result_f32 {
  fract : f32, // fractional part
  whole : f32  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_whole');
    await run(t, wholeBuilder(), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f32_vec2_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec2<f32>

struct __modf_result_vec2_f32 {
  fract : vec2<f32>, // fractional part
  whole : vec2<f32>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_vec2_fract');
    await run(t, fractBuilder(), [TypeVec(2, TypeF32)], TypeVec(2, TypeF32), t.params, cases);
  });

g.test('f32_vec2_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec2<f32>

struct __modf_result_vec2_f32 {
  fract : vec2<f32>, // fractional part
  whole : vec2<f32>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_vec2_whole');
    await run(t, wholeBuilder(), [TypeVec(2, TypeF32)], TypeVec(2, TypeF32), t.params, cases);
  });

g.test('f32_vec3_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec3<f32>

struct __modf_result_vec3_f32 {
  fract : vec3<f32>, // fractional part
  whole : vec3<f32>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_vec3_fract');
    await run(t, fractBuilder(), [TypeVec(3, TypeF32)], TypeVec(3, TypeF32), t.params, cases);
  });

g.test('f32_vec3_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec3<f32>

struct __modf_result_vec3_f32 {
  fract : vec3<f32>, // fractional part
  whole : vec3<f32>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_vec3_whole');
    await run(t, wholeBuilder(), [TypeVec(3, TypeF32)], TypeVec(3, TypeF32), t.params, cases);
  });

g.test('f32_vec4_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec4<f32>

struct __modf_result_vec4_f32 {
  fract : vec4<f32>, // fractional part
  whole : vec4<f32>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_vec4_fract');
    await run(t, fractBuilder(), [TypeVec(4, TypeF32)], TypeVec(4, TypeF32), t.params, cases);
  });

g.test('f32_vec4_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec4<f32>

struct __modf_result_vec4_f32 {
  fract : vec4<f32>, // fractional part
  whole : vec4<f32>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32_vec4_whole');
    await run(t, wholeBuilder(), [TypeVec(4, TypeF32)], TypeVec(4, TypeF32), t.params, cases);
  });

g.test('f16_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is f16

struct __modf_result_f16 {
  fract : f16, // fractional part
  whole : f16  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f16_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is f16

struct __modf_result_f16 {
  fract : f16, // fractional part
  whole : f16  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f16_vec2_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec2<f16>

struct __modf_result_vec2_f16 {
  fract : vec2<f16>, // fractional part
  whole : vec2<f16>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f16_vec2_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec2<f16>

struct __modf_result_vec2_f16 {
  fract : vec2<f16>, // fractional part
  whole : vec2<f16>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f16_vec3_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec3<f16>

struct __modf_result_vec3_f16 {
  fract : vec3<f16>, // fractional part
  whole : vec3<f16>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f16_vec3_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec3<f16>

struct __modf_result_vec3_f16 {
  fract : vec3<f16>, // fractional part
  whole : vec3<f16>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f16_vec4_fract')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec4<f16>

struct __modf_result_vec4_f16 {
  fract : vec4<f16>, // fractional part
  whole : vec4<f16>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f16_vec4_whole')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
T is vec4<f16>

struct __modf_result_vec4_f16 {
  fract : vec4<f16>, // fractional part
  whole : vec4<f16>  // whole part
}
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();
