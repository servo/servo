/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the f16 arithmetic unary expression operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF16 } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { fullF16Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/f16_arithmetic', {
  negation: () => {
    return FP.f16.generateScalarToIntervalCases(
      fullF16Range({ neg_norm: 250, neg_sub: 20, pos_sub: 20, pos_norm: 250 }),
      'unfiltered',
      FP.f16.negationInterval
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
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get('negation');
    await run(t, unary('-'), [TypeF16], TypeF16, t.params, cases);
  });
