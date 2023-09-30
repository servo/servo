/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'step' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn step(edge: T ,x: T ) -> T
Returns 1.0 if edge ≤ x, and 0.0 otherwise. Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { anyOf } from '../../../../../util/compare.js';
import { TypeF32, TypeF16 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range, fullF16Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// stepInterval's return value can't always be interpreted as a single acceptance
// interval, valid result may be 0.0 or 1.0 or both of them, but will never be a
// value in interval (0.0, 1.0).
// See the comment block on stepInterval for more details
const makeCase = (trait, edge, x) => {
  const FPTrait = FP[trait];
  edge = FPTrait.quantize(edge);
  x = FPTrait.quantize(x);
  const expected = FPTrait.stepInterval(edge, x);

  // [0, 0], [1, 1], or [-∞, +∞] cases
  if (expected.isPoint() || !expected.isFinite()) {
    return { input: [FPTrait.scalarBuilder(edge), FPTrait.scalarBuilder(x)], expected };
  }

  // [0, 1] case, valid result is either 0.0 or 1.0.
  const zeroInterval = FPTrait.toInterval(0);
  const oneInterval = FPTrait.toInterval(1);
  return {
    input: [FPTrait.scalarBuilder(edge), FPTrait.scalarBuilder(x)],
    expected: anyOf(zeroInterval, oneInterval),
  };
};

export const d = makeCaseCache('step', {
  f32: () => {
    return fullF32Range().flatMap(edge => fullF32Range().map(x => makeCase('f32', edge, x)));
  },
  f16: () => {
    return fullF16Range().flatMap(edge => fullF16Range().map(x => makeCase('f16', edge, x)));
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, builtin('step'), [TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, builtin('step'), [TypeF16, TypeF16], TypeF16, t.params, cases);
  });
