/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'min' builtin function

S is AbstractInt, i32, or u32
T is S or vecN<S>
@const fn min(e1: T ,e2: T) -> T
Returns e1 if e1 is less than e2, and e2 otherwise. Component-wise when T is a vector.

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn min(e1: T ,e2: T) -> T
Returns e2 if e2 is less than e1, and e1 otherwise.
If one operand is a NaN, the other is returned.
If both operands are NaNs, a NaN is returned.
Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import {
  i32,
  TypeF32,
  TypeF16,
  TypeI32,
  TypeU32,
  u32,
  TypeAbstractFloat,
} from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range, fullF16Range, sparseF64Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractBuiltin, builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('min', {
  f32: () => {
    return FP.f32.generateScalarPairToIntervalCases(
      fullF32Range(),
      fullF32Range(),
      'unfiltered',
      FP.f32.minInterval
    );
  },
  f16: () => {
    return FP.f16.generateScalarPairToIntervalCases(
      fullF16Range(),
      fullF16Range(),
      'unfiltered',
      FP.f16.minInterval
    );
  },
  abstract: () => {
    return FP.abstract.generateScalarPairToIntervalCases(
      sparseF64Range(),
      sparseF64Range(),
      'unfiltered',
      FP.abstract.minInterval
    );
  },
});

/** Generate set of min test cases from list of interesting values */
function generateTestCases(values, makeCase) {
  const cases = new Array();
  values.forEach(e => {
    values.forEach(f => {
      cases.push(makeCase(e, f));
    });
  });
  return cases;
}

g.test('abstract_int')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`abstract int tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`u32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const makeCase = (x, y) => {
      return { input: [u32(x), u32(y)], expected: u32(Math.min(x, y)) };
    };

    const test_values = [0, 1, 2, 0x70000000, 0x80000000, 0xffffffff];
    const cases = generateTestCases(test_values, makeCase);

    await run(t, builtin('min'), [TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`i32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const makeCase = (x, y) => {
      return { input: [i32(x), i32(y)], expected: i32(Math.min(x, y)) };
    };

    const test_values = [-0x70000000, -2, -1, 0, 1, 2, 0x70000000];
    const cases = generateTestCases(test_values, makeCase);

    await run(t, builtin('min'), [TypeI32, TypeI32], TypeI32, t.params, cases);
  });

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u =>
    u.combine('inputSource', onlyConstInputSource).combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const cases = await d.get('abstract');
    await run(
      t,
      abstractBuiltin('min'),
      [TypeAbstractFloat, TypeAbstractFloat],
      TypeAbstractFloat,
      t.params,
      cases
    );
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, builtin('min'), [TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, builtin('min'), [TypeF16, TypeF16], TypeF16, t.params, cases);
  });
