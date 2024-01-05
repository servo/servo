/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'modf' builtin function

T is f32 or f16 or AbstractFloat
@const fn modf(e:T) -> result_struct
Splits |e| into fractional and whole number parts.
The whole part is (|e| % 1.0), and the fractional part is |e| minus the whole part.
Returns the result_struct for the given type.

S is f32 or f16 or AbstractFloat
T is vecN<S>
@const fn modf(e:T) -> result_struct
Splits the components of |e| into fractional and whole number parts.
The |i|'th component of the whole and fractional parts equal the whole and fractional parts of modf(e[i]).
Returns the result_struct for the given type.

`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeAbstractFloat, TypeF16, TypeF32, TypeVec } from '../../../../../util/conversion.js';
import {
  abstractFloatShaderBuilder,
  allInputSources,
  basicExpressionBuilder,
  onlyConstInputSource,
  run } from

'../../expression.js';

import { d } from './modf.cache.js';

export const g = makeTestGroup(GPUTest);

/** @returns an ShaderBuilder that evaluates modf and returns .whole from the result structure */
function wholeBuilder() {
  return basicExpressionBuilder((value) => `modf(${value}).whole`);
}

/** @returns an ShaderBuilder that evaluates modf and returns .fract from the result structure */
function fractBuilder() {
  return basicExpressionBuilder((value) => `modf(${value}).fract`);
}

/** @returns an ShaderBuilder that evaluates modf and returns .whole from the result structure for AbstractFloats */
function abstractWholeBuilder() {
  return abstractFloatShaderBuilder((value) => `modf(${value}).whole`);
}

/** @returns an ShaderBuilder that evaluates modf and returns .fract from the result structure for AbstractFloats */
function abstractFractBuilder() {
  return abstractFloatShaderBuilder((value) => `modf(${value}).fract`);
}

g.test('f32_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f32

struct __modf_result_f32 {
  fract : f32, // fractional part
  whole : f32  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_fract');
  await run(t, fractBuilder(), [TypeF32], TypeF32, t.params, cases);
});

g.test('f32_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f32

struct __modf_result_f32 {
  fract : f32, // fractional part
  whole : f32  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_whole');
  await run(t, wholeBuilder(), [TypeF32], TypeF32, t.params, cases);
});

g.test('f32_vec2_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f32>

struct __modf_result_vec2_f32 {
  fract : vec2<f32>, // fractional part
  whole : vec2<f32>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec2_fract');
  await run(t, fractBuilder(), [TypeVec(2, TypeF32)], TypeVec(2, TypeF32), t.params, cases);
});

g.test('f32_vec2_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f32>

struct __modf_result_vec2_f32 {
  fract : vec2<f32>, // fractional part
  whole : vec2<f32>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec2_whole');
  await run(t, wholeBuilder(), [TypeVec(2, TypeF32)], TypeVec(2, TypeF32), t.params, cases);
});

g.test('f32_vec3_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f32>

struct __modf_result_vec3_f32 {
  fract : vec3<f32>, // fractional part
  whole : vec3<f32>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec3_fract');
  await run(t, fractBuilder(), [TypeVec(3, TypeF32)], TypeVec(3, TypeF32), t.params, cases);
});

g.test('f32_vec3_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f32>

struct __modf_result_vec3_f32 {
  fract : vec3<f32>, // fractional part
  whole : vec3<f32>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec3_whole');
  await run(t, wholeBuilder(), [TypeVec(3, TypeF32)], TypeVec(3, TypeF32), t.params, cases);
});

g.test('f32_vec4_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f32>

struct __modf_result_vec4_f32 {
  fract : vec4<f32>, // fractional part
  whole : vec4<f32>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec4_fract');
  await run(t, fractBuilder(), [TypeVec(4, TypeF32)], TypeVec(4, TypeF32), t.params, cases);
});

g.test('f32_vec4_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f32>

struct __modf_result_vec4_f32 {
  fract : vec4<f32>, // fractional part
  whole : vec4<f32>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec4_whole');
  await run(t, wholeBuilder(), [TypeVec(4, TypeF32)], TypeVec(4, TypeF32), t.params, cases);
});

g.test('f16_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f16

struct __modf_result_f16 {
  fract : f16, // fractional part
  whole : f16  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_fract');
  await run(t, fractBuilder(), [TypeF16], TypeF16, t.params, cases);
});

g.test('f16_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f16

struct __modf_result_f16 {
  fract : f16, // fractional part
  whole : f16  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_whole');
  await run(t, wholeBuilder(), [TypeF16], TypeF16, t.params, cases);
});

g.test('f16_vec2_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f16>

struct __modf_result_vec2_f16 {
  fract : vec2<f16>, // fractional part
  whole : vec2<f16>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_vec2_fract');
  await run(t, fractBuilder(), [TypeVec(2, TypeF16)], TypeVec(2, TypeF16), t.params, cases);
});

g.test('f16_vec2_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f16>

struct __modf_result_vec2_f16 {
  fract : vec2<f16>, // fractional part
  whole : vec2<f16>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_vec2_whole');
  await run(t, wholeBuilder(), [TypeVec(2, TypeF16)], TypeVec(2, TypeF16), t.params, cases);
});

g.test('f16_vec3_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f16>

struct __modf_result_vec3_f16 {
  fract : vec3<f16>, // fractional part
  whole : vec3<f16>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_vec3_fract');
  await run(t, fractBuilder(), [TypeVec(3, TypeF16)], TypeVec(3, TypeF16), t.params, cases);
});

g.test('f16_vec3_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f16>

struct __modf_result_vec3_f16 {
  fract : vec3<f16>, // fractional part
  whole : vec3<f16>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_vec3_whole');
  await run(t, wholeBuilder(), [TypeVec(3, TypeF16)], TypeVec(3, TypeF16), t.params, cases);
});

g.test('f16_vec4_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f16>

struct __modf_result_vec4_f16 {
  fract : vec4<f16>, // fractional part
  whole : vec4<f16>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_vec4_fract');
  await run(t, fractBuilder(), [TypeVec(4, TypeF16)], TypeVec(4, TypeF16), t.params, cases);
});

g.test('f16_vec4_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f16>

struct __modf_result_vec4_f16 {
  fract : vec4<f16>, // fractional part
  whole : vec4<f16>  // whole part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16_vec4_whole');
  await run(t, wholeBuilder(), [TypeVec(4, TypeF16)], TypeVec(4, TypeF16), t.params, cases);
});

g.test('abstract_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is AbstractFloat

struct __modf_result_abstract {
  fract : AbstractFloat, // fractional part
  whole : AbstractFloat  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_fract');
  await run(t, abstractFractBuilder(), [TypeAbstractFloat], TypeAbstractFloat, t.params, cases);
});

g.test('abstract_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is AbstractFloat

struct __modf_result_abstract {
  fract : AbstractFloat, // fractional part
  whole : AbstractFloat  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_whole');
  await run(t, abstractWholeBuilder(), [TypeAbstractFloat], TypeAbstractFloat, t.params, cases);
});

g.test('abstract_vec2_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<abstract>

struct __modf_result_vec2_abstract {
  fract : vec2<abstract>, // fractional part
  whole : vec2<abstract>  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec2_fract');
  await run(
    t,
    abstractFractBuilder(),
    [TypeVec(2, TypeAbstractFloat)],
    TypeVec(2, TypeAbstractFloat),
    t.params,
    cases
  );
});

g.test('abstract_vec2_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<abstract>

struct __modf_result_vec2_abstract {
  fract : vec2<abstract>, // fractional part
  whole : vec2<abstract>  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec2_whole');
  await run(
    t,
    abstractWholeBuilder(),
    [TypeVec(2, TypeAbstractFloat)],
    TypeVec(2, TypeAbstractFloat),
    t.params,
    cases
  );
});

g.test('abstract_vec3_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<abstract>

struct __modf_result_vec3_abstract {
  fract : vec3<abstract>, // fractional part
  whole : vec3<abstract>  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec3_fract');
  await run(
    t,
    abstractFractBuilder(),
    [TypeVec(3, TypeAbstractFloat)],
    TypeVec(3, TypeAbstractFloat),
    t.params,
    cases
  );
});

g.test('abstract_vec3_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<abstract>

struct __modf_result_vec3_abstract {
  fract : vec3<abstract>, // fractional part
  whole : vec3<abstract>  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec3_whole');
  await run(
    t,
    abstractWholeBuilder(),
    [TypeVec(3, TypeAbstractFloat)],
    TypeVec(3, TypeAbstractFloat),
    t.params,
    cases
  );
});

g.test('abstract_vec4_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<abstract>

struct __modf_result_vec4_abstract {
  fract : vec4<abstract>, // fractional part
  whole : vec4<abstract>  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec4_fract');
  await run(
    t,
    abstractFractBuilder(),
    [TypeVec(4, TypeAbstractFloat)],
    TypeVec(4, TypeAbstractFloat),
    t.params,
    cases
  );
});

g.test('abstract_vec4_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<abstract>

struct __modf_result_vec4_abstract {
  fract : vec4<abstract>, // fractional part
  whole : vec4<abstract>  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec4_whole');
  await run(
    t,
    abstractWholeBuilder(),
    [TypeVec(4, TypeAbstractFloat)],
    TypeVec(4, TypeAbstractFloat),
    t.params,
    cases
  );
});