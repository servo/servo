/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for AbstractFloat arithmetic unary expression operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeAbstractFloat } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { fullF64Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { onlyConstInputSource, run } from '../expression.js';

import { abstractUnary } from './unary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/af_arithmetic', {
  negation: () => {
    return FP.abstract.generateScalarToIntervalCases(
      fullF64Range({ neg_norm: 250, neg_sub: 20, pos_sub: 20, pos_norm: 250 }),
      'unfiltered',
      FP.abstract.negationInterval
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
  .params(u =>
    u.combine('inputSource', onlyConstInputSource).combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const cases = await d.get('negation');
    await run(t, abstractUnary('-'), [TypeAbstractFloat], TypeAbstractFloat, t.params, cases, 1);
  });
