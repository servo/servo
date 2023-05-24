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
import { TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('step', {
  f32: () => {
    const zeroInterval = FP.f32.toInterval(0);
    const oneInterval = FP.f32.toInterval(1);

    // stepInterval's return value isn't always interpreted as an acceptance
    // interval, so makeBinaryToF32IntervalCase cannot be used here.
    // See the comment block on stepInterval for more details
    const makeCase = (edge, x) => {
      edge = FP.f32.quantize(edge);
      x = FP.f32.quantize(x);
      const expected = FP.f32.stepInterval(edge, x);

      // [0, 0], [1, 1], or [-∞, +∞] cases
      if (expected.isPoint() || !expected.isFinite()) {
        return { input: [FP.f32.scalarBuilder(edge), FP.f32.scalarBuilder(x)], expected };
      }

      // [0, 1] case
      return {
        input: [FP.f32.scalarBuilder(edge), FP.f32.scalarBuilder(x)],
        expected: anyOf(zeroInterval, oneInterval),
      };
    };

    const range = fullF32Range();
    const cases = [];
    range.forEach(edge => {
      range.forEach(x => {
        cases.push(makeCase(edge, x));
      });
    });

    return cases;
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
  .unimplemented();
