/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'frexp' builtin function

S is f32 or f16
T is S or vecN<S>

@const fn frexp(e: T) -> result_struct

Splits e into a significand and exponent of the form significand * 2^exponent.
Returns the result_struct for the appropriate overload.


The magnitude of the significand is in the range of [0.5, 1.0) or 0.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import {

  allInputSources,
  basicExpressionBuilder,
  run,
  abstractFloatShaderBuilder,
  abstractIntShaderBuilder,
  onlyConstInputSource } from
'../../expression.js';

import { d } from './frexp.cache.js';

export const g = makeTestGroup(GPUTest);

/* @returns an ShaderBuilder that evaluates frexp and returns .fract from the result structure */
function fractBuilder() {
  return basicExpressionBuilder((value) => `frexp(${value}).fract`);
}

/* @returns an ShaderBuilder that evaluates frexp and returns .exp from the result structure */
function expBuilder() {
  return basicExpressionBuilder((value) => `frexp(${value}).exp`);
}

/* @returns an ShaderBuilder that evaluates frexp and returns .fract from the result structure, for abstract inputs */
function abstractFractBuilder() {
  return abstractFloatShaderBuilder((value) => `frexp(${value}).fract`);
}

/* @returns an ShaderBuilder that evaluates frexp and returns .exp from the result structure, for abstract inputs */
function abstractExpBuilder() {
  return abstractIntShaderBuilder((value) => `frexp(${value}).exp`);
}

g.test('abstract_float_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is AbstractFloat

struct __frexp_result_abstract {
  fract : AbstractFloat, // fract part
  exp : AbstractInt  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_fract');
  await run(t, abstractFractBuilder(), [Type.abstractFloat], Type.abstractFloat, t.params, cases);
});

g.test('abstract_float_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is AbstractFloat

struct __frexp_result_abstract {
  fract : AbstractFloat, // fract part
  exp : AbstractInt  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_exp');
  await run(t, abstractExpBuilder(), [Type.abstractFloat], Type.abstractInt, t.params, cases);
});

g.test('abstract_float_vec2_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<AbstractFloat>

struct __frexp_result_vec2_abstract {
  fract : vec2<AbstractFloat>, // fract part
  exp : vec2<AbstractInt>  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec2_fract');
  await run(t, abstractFractBuilder(), [Type.vec2af], Type.vec2af, t.params, cases);
});

g.test('abstract_float_vec2_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<AbstractFloat>

struct __frexp_result_vec2_abstract {
  fract : vec2<AbstractFloat>, // fractional part
  exp : vec2<AbstractInt>  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec2_exp');
  await run(t, abstractExpBuilder(), [Type.vec2af], Type.vec2ai, t.params, cases);
});

g.test('abstract_float_vec3_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<AbstractFloat>

struct __frexp_result_vec3_abstract {
  fract : vec3<AbstractFloat>, // fractional part
  exp : vec3<AbstractInt>  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec3_fract');
  await run(t, abstractFractBuilder(), [Type.vec3af], Type.vec3af, t.params, cases);
});

g.test('abstract_float_vec3_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<AbstractFloat>

struct __frexp_result_vec3_abstract {
  fract : vec3<AbstractFloat>, // fractional part
  exp : vec3<AbstractInt>  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec3_exp');
  await run(t, abstractExpBuilder(), [Type.vec3af], Type.vec3ai, t.params, cases);
});

g.test('abstract_float_vec4_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<AbstractFloat>

struct __frexp_result_vec4_abstract {
  fract : vec4<AbstractFloat>, // fractional part
  exp : vec4<AbstractInt>  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec4_fract');
  await run(t, abstractFractBuilder(), [Type.vec4af], Type.vec4af, t.params, cases);
});

g.test('abstract_float_vec4_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<AbstractFloat>

struct __frexp_result_vec4_abstract {
  fract : vec4<AbstractFloat>, // fractional part
  exp : vec4<AbstractInt>  // exponent part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec4_exp');
  await run(t, abstractExpBuilder(), [Type.vec4af], Type.vec4ai, t.params, cases);
});

g.test('f32_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f32

struct __frexp_result_f32 {
  fract : f32, // fract part
  exp : i32  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_fract');
  await run(t, fractBuilder(), [Type.f32], Type.f32, t.params, cases);
});

g.test('f32_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f32

struct __frexp_result_f32 {
  fract : f32, // fract part
  exp : i32  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_exp');
  await run(t, expBuilder(), [Type.f32], Type.i32, t.params, cases);
});

g.test('f32_vec2_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f32>

struct __frexp_result_vec2_f32 {
  fract : vec2<f32>, // fract part
  exp : vec2<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec2_fract');
  await run(t, fractBuilder(), [Type.vec2f], Type.vec2f, t.params, cases);
});

g.test('f32_vec2_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f32>

struct __frexp_result_vec2_f32 {
  fract : vec2<f32>, // fractional part
  exp : vec2<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec2_exp');
  await run(t, expBuilder(), [Type.vec2f], Type.vec2i, t.params, cases);
});

g.test('f32_vec3_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f32>

struct __frexp_result_vec3_f32 {
  fract : vec3<f32>, // fractional part
  exp : vec3<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec3_fract');
  await run(t, fractBuilder(), [Type.vec3f], Type.vec3f, t.params, cases);
});

g.test('f32_vec3_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f32>

struct __frexp_result_vec3_f32 {
  fract : vec3<f32>, // fractional part
  exp : vec3<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec3_exp');
  await run(t, expBuilder(), [Type.vec3f], Type.vec3i, t.params, cases);
});

g.test('f32_vec4_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f32>

struct __frexp_result_vec4_f32 {
  fract : vec4<f32>, // fractional part
  exp : vec4<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec4_fract');
  await run(t, fractBuilder(), [Type.vec4f], Type.vec4f, t.params, cases);
});

g.test('f32_vec4_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f32>

struct __frexp_result_vec4_f32 {
  fract : vec4<f32>, // fractional part
  exp : vec4<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32_vec4_exp');
  await run(t, expBuilder(), [Type.vec4f], Type.vec4i, t.params, cases);
});

g.test('f16_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f16

struct __frexp_result_f16 {
  fract : f16, // fract part
  exp : i32  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_fract');
  await run(t, fractBuilder(), [Type.f16], Type.f16, t.params, cases);
});

g.test('f16_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is f16

struct __frexp_result_f16 {
  fract : f16, // fract part
  exp : i32  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_exp');
  await run(t, expBuilder(), [Type.f16], Type.i32, t.params, cases);
});

g.test('f16_vec2_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f16>

struct __frexp_result_vec2_f16 {
  fract : vec2<f16>, // fract part
  exp : vec2<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_vec2_fract');
  await run(t, fractBuilder(), [Type.vec2h], Type.vec2h, t.params, cases);
});

g.test('f16_vec2_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec2<f16>

struct __frexp_result_vec2_f16 {
  fract : vec2<f16>, // fractional part
  exp : vec2<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_vec2_exp');
  await run(t, expBuilder(), [Type.vec2h], Type.vec2i, t.params, cases);
});

g.test('f16_vec3_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f16>

struct __frexp_result_vec3_f16 {
  fract : vec3<f16>, // fractional part
  exp : vec3<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_vec3_fract');
  await run(t, fractBuilder(), [Type.vec3h], Type.vec3h, t.params, cases);
});

g.test('f16_vec3_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec3<f16>

struct __frexp_result_vec3_f16 {
  fract : vec3<f16>, // fractional part
  exp : vec3<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_vec3_exp');
  await run(t, expBuilder(), [Type.vec3h], Type.vec3i, t.params, cases);
});

g.test('f16_vec4_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f16>

struct __frexp_result_vec4_f16 {
  fract : vec4<f16>, // fractional part
  exp : vec4<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_vec4_fract');
  await run(t, fractBuilder(), [Type.vec4h], Type.vec4h, t.params, cases);
});

g.test('f16_vec4_exp').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is vec4<f16>

struct __frexp_result_vec4_f16 {
  fract : vec4<f16>, // fractional part
  exp : vec4<i32>  // exponent part
}
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16_vec4_exp');
  await run(t, expBuilder(), [Type.vec4h], Type.vec4i, t.params, cases);
});