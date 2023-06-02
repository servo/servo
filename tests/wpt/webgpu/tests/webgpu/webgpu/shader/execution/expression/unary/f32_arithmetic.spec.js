/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the f32 arithmetic unary expression operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF32 } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { fullF32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/f32_arithmetic', {
  negation: () => {
    return FP.f32.generateScalarToIntervalCases(
      fullF32Range({ neg_norm: 250, neg_sub: 20, pos_sub: 20, pos_norm: 250 }),
      'unfiltered',
      FP.f32.negationInterval
    );
  },
});

g.test('negation')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: -x
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('negation');
    await run(t, unary('-'), [TypeF32], TypeF32, t.params, cases);
  });
