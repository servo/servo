/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'modf' builtin function

T is f32 or f16 or Type.abstractFloat
@const fn modf(e:T) -> result_struct
Splits |e| into fractional and whole number parts.
The whole part is (|e| % 1.0), and the fractional part is |e| minus the whole part.
Returns the result_struct for the given type.

S is f32 or f16 or Type.abstractFloat
T is vecN<S>
@const fn modf(e:T) -> result_struct
Splits the components of |e| into fractional and whole number parts.
The |i|'th component of the whole and fractional parts equal the whole and fractional parts of modf(e[i]).
Returns the result_struct for the given type.

`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
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
  await run(t, fractBuilder(), [Type.f32], Type.f32, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.f32], Type.f32, t.params, cases);
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
  await run(t, fractBuilder(), [Type.vec2f], Type.vec2f, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.vec2f], Type.vec2f, t.params, cases);
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
  await run(t, fractBuilder(), [Type.vec3f], Type.vec3f, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.vec3f], Type.vec3f, t.params, cases);
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
  await run(t, fractBuilder(), [Type.vec4f], Type.vec4f, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.vec4f], Type.vec4f, t.params, cases);
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
  await run(t, fractBuilder(), [Type.f16], Type.f16, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.f16], Type.f16, t.params, cases);
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
  await run(t, fractBuilder(), [Type.vec2h], Type.vec2h, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.vec2h], Type.vec2h, t.params, cases);
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
  await run(t, fractBuilder(), [Type.vec3h], Type.vec3h, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.vec3h], Type.vec3h, t.params, cases);
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
  await run(t, fractBuilder(), [Type.vec4h], Type.vec4h, t.params, cases);
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
  await run(t, wholeBuilder(), [Type.vec4h], Type.vec4h, t.params, cases);
});

g.test('abstract_fract').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is abstract-float

struct __modf_result_abstract {
  fract : Type.abstractFloat, // fractional part
  whole : Type.abstractFloat  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_fract');
  await run(t, abstractFractBuilder(), [Type.abstractFloat], Type.abstractFloat, t.params, cases);
});

g.test('abstract_whole').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(
  `
T is abstract-float

struct __modf_result_abstract {
  fract : Type.abstractFloat, // fractional part
  whole : Type.abstractFloat  // whole part
}
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_whole');
  await run(t, abstractWholeBuilder(), [Type.abstractFloat], Type.abstractFloat, t.params, cases);
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
    [Type.vec(2, Type.abstractFloat)],
    Type.vec(2, Type.abstractFloat),
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
    [Type.vec(2, Type.abstractFloat)],
    Type.vec(2, Type.abstractFloat),
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
    [Type.vec(3, Type.abstractFloat)],
    Type.vec(3, Type.abstractFloat),
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
    [Type.vec(3, Type.abstractFloat)],
    Type.vec(3, Type.abstractFloat),
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
    [Type.vec(4, Type.abstractFloat)],
    Type.vec(4, Type.abstractFloat),
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
    [Type.vec(4, Type.abstractFloat)],
    Type.vec(4, Type.abstractFloat),
    t.params,
    cases
  );
});