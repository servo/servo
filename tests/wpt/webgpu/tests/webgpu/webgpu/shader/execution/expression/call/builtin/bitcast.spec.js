/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'bitcast' builtin function

@const @must_use fn bitcast<T>(e: T ) -> T
T is concrete numeric scalar or concerete numeric vector
Identity function.

@const @must_use fn bitcast<T>(e: S ) -> T
@const @must_use fn bitcast<vecN<T>>(e: vecN<S> ) -> vecN<T>
S is i32, u32, f32
T is i32, u32, f32, and T is not S
Reinterpretation of bits.  Beware non-normal f32 values.

@const @must_use fn bitcast<T>(e: vec2<f16> ) -> T
@const @must_use fn bitcast<vec2<T>>(e: vec4<f16> ) -> vec2<T>
@const @must_use fn bitcast<vec2<f16>>(e: T ) -> vec2<f16>
@const @must_use fn bitcast<vec4<f16>>(e: vec2<T> ) -> vec4<f16>
T is i32, u32, f32
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF16, TypeF32, TypeI32, TypeU32, TypeVec } from '../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { d } from './bitcast.cache.js';
import { builtinWithPredeclaration } from './builtin.js';

export const g = makeTestGroup(GPUTest);

/**
 * @returns a ShaderBuilder that generates a call to bitcast,
 * using appropriate destination type, which optionally can be
 * a WGSL type alias.
 */
function bitcastBuilder(canonicalDestType, params) {
  const destType = params.vectorize ?
  `vec${params.vectorize}<${canonicalDestType}>` :
  canonicalDestType;

  return builtinWithPredeclaration(
    `bitcast<${destType}>`,
    params.alias ? `alias myalias = ${destType};` : ''
  );
}

// Identity cases
g.test('i32_to_i32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast i32 to i32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get('i32_to_i32');
  await run(t, bitcastBuilder('i32', t.params), [TypeI32], TypeI32, t.params, cases);
});

g.test('u32_to_u32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast u32 to u32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get('u32_to_u32');
  await run(t, bitcastBuilder('u32', t.params), [TypeU32], TypeU32, t.params, cases);
});

g.test('f32_to_f32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast f32 to f32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'f32_to_f32' : 'f32_inf_nan_to_f32'
  );
  await run(t, bitcastBuilder('f32', t.params), [TypeF32], TypeF32, t.params, cases);
});

// To i32 from u32, f32
g.test('u32_to_i32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast u32 to i32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get('u32_to_i32');
  await run(t, bitcastBuilder('i32', t.params), [TypeU32], TypeI32, t.params, cases);
});

g.test('f32_to_i32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast f32 to i32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'f32_to_i32' : 'f32_inf_nan_to_i32'
  );
  await run(t, bitcastBuilder('i32', t.params), [TypeF32], TypeI32, t.params, cases);
});

// To u32 from i32, f32
g.test('i32_to_u32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast i32 to u32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get('i32_to_u32');
  await run(t, bitcastBuilder('u32', t.params), [TypeI32], TypeU32, t.params, cases);
});

g.test('f32_to_u32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast f32 to i32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'f32_to_u32' : 'f32_inf_nan_to_u32'
  );
  await run(t, bitcastBuilder('u32', t.params), [TypeF32], TypeU32, t.params, cases);
});

// To f32 from i32, u32
g.test('i32_to_f32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast i32 to f32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'i32_to_f32' : 'i32_to_f32_inf_nan'
  );
  await run(t, bitcastBuilder('f32', t.params), [TypeI32], TypeF32, t.params, cases);
});

g.test('u32_to_f32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast u32 to f32 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'u32_to_f32' : 'u32_to_f32_inf_nan'
  );
  await run(t, bitcastBuilder('f32', t.params), [TypeU32], TypeF32, t.params, cases);
});

// 16 bit types

// f16 cases

// f16: Identity
g.test('f16_to_f16').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast f16 to f16 tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4]).
combine('alias', [false, true])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'f16_to_f16' : 'f16_inf_nan_to_f16'
  );
  await run(t, bitcastBuilder('f16', t.params), [TypeF16], TypeF16, t.params, cases);
});

// f16: 32-bit scalar numeric to vec2<f16>
g.test('i32_to_vec2h').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast i32 to vec2h tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'i32_to_vec2_f16' : 'i32_to_vec2_f16_inf_nan'
  );
  await run(
    t,
    bitcastBuilder('vec2<f16>', t.params),
    [TypeI32],
    TypeVec(2, TypeF16),
    t.params,
    cases
  );
});

g.test('u32_to_vec2h').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast u32 to vec2h tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'u32_to_vec2_f16' : 'u32_to_vec2_f16_inf_nan'
  );
  await run(
    t,
    bitcastBuilder('vec2<f16>', t.params),
    [TypeU32],
    TypeVec(2, TypeF16),
    t.params,
    cases
  );
});

g.test('f32_to_vec2h').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast u32 to vec2h tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'f32_to_vec2_f16' : 'f32_inf_nan_to_vec2_f16_inf_nan'
  );
  await run(
    t,
    bitcastBuilder('vec2<f16>', t.params),
    [TypeF32],
    TypeVec(2, TypeF16),
    t.params,
    cases
  );
});

// f16: vec2<32-bit scalar numeric> to vec4<f16>
g.test('vec2i_to_vec4h').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec2i to vec4h tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'vec2_i32_to_vec4_f16' : 'vec2_i32_to_vec4_f16_inf_nan'
  );
  await run(
    t,
    bitcastBuilder('vec4<f16>', t.params),
    [TypeVec(2, TypeI32)],
    TypeVec(4, TypeF16),
    t.params,
    cases
  );
});

g.test('vec2u_to_vec4h').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec2u to vec4h tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'vec2_u32_to_vec4_f16' : 'vec2_u32_to_vec4_f16_inf_nan'
  );
  await run(
    t,
    bitcastBuilder('vec4<f16>', t.params),
    [TypeVec(2, TypeU32)],
    TypeVec(4, TypeF16),
    t.params,
    cases
  );
});

g.test('vec2f_to_vec4h').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec2f to vec2h tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ?
    'vec2_f32_to_vec4_f16' :
    'vec2_f32_inf_nan_to_vec4_f16_inf_nan'
  );
  await run(
    t,
    bitcastBuilder('vec4<f16>', t.params),
    [TypeVec(2, TypeF32)],
    TypeVec(4, TypeF16),
    t.params,
    cases
  );
});

// f16: vec2<f16> to 32-bit scalar numeric
g.test('vec2h_to_i32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec2h to i32 tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'vec2_f16_to_i32' : 'vec2_f16_inf_nan_to_i32'
  );
  await run(t, bitcastBuilder('i32', t.params), [TypeVec(2, TypeF16)], TypeI32, t.params, cases);
});

g.test('vec2h_to_u32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec2h to u32 tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'vec2_f16_to_u32' : 'vec2_f16_inf_nan_to_u32'
  );
  await run(t, bitcastBuilder('u32', t.params), [TypeVec(2, TypeF16)], TypeU32, t.params, cases);
});

g.test('vec2h_to_f32').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec2h to f32 tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'vec2_f16_to_f32_finite' : 'vec2_f16_inf_nan_to_f32'
  );
  await run(t, bitcastBuilder('f32', t.params), [TypeVec(2, TypeF16)], TypeF32, t.params, cases);
});

// f16: vec4<f16> to vec2<32-bit scalar numeric>
g.test('vec4h_to_vec2i').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec4h to vec2i tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'vec4_f16_to_vec2_i32' : 'vec4_f16_inf_nan_to_vec2_i32'
  );
  await run(
    t,
    bitcastBuilder('vec2<i32>', t.params),
    [TypeVec(4, TypeF16)],
    TypeVec(2, TypeI32),
    t.params,
    cases
  );
});

g.test('vec4h_to_vec2u').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec4h to vec2u tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ? 'vec4_f16_to_vec2_u32' : 'vec4_f16_inf_nan_to_vec2_u32'
  );
  await run(
    t,
    bitcastBuilder('vec2<u32>', t.params),
    [TypeVec(4, TypeF16)],
    TypeVec(2, TypeU32),
    t.params,
    cases
  );
});

g.test('vec4h_to_vec2f').
specURL('https://www.w3.org/TR/WGSL/#bitcast-builtin').
desc(`bitcast vec4h to vec2f tests`).
params((u) => u.combine('inputSource', allInputSources).combine('alias', [false, true])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    // Infinities and NaNs are errors in const-eval.
    t.params.inputSource === 'const' ?
    'vec4_f16_to_vec2_f32_finite' :
    'vec4_f16_inf_nan_to_vec2_f32'
  );
  await run(
    t,
    bitcastBuilder('vec2<f32>', t.params),
    [TypeVec(4, TypeF16)],
    TypeVec(2, TypeF32),
    t.params,
    cases
  );
});