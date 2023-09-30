/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for non-matrix AbstractFloat addition expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeAbstractFloat, TypeVec } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseF64Range, sparseVectorF64Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { onlyConstInputSource, run } from '../expression.js';

import { abstractBinary } from './binary.js';

const additionVectorScalarInterval = (v, s) => {
  return FP.abstract.toVector(v.map(e => FP.abstract.additionInterval(e, s)));
};

const additionScalarVectorInterval = (s, v) => {
  return FP.abstract.toVector(v.map(e => FP.abstract.additionInterval(s, e)));
};

export const g = makeTestGroup(GPUTest);

const scalar_cases = {
  ['scalar']: () => {
    return FP.abstract.generateScalarPairToIntervalCases(
      sparseF64Range(),
      sparseF64Range(),
      'finite',
      FP.abstract.additionInterval
    );
  },
};

const vector_scalar_cases = [2, 3, 4]
  .map(dim => ({
    [`vec${dim}_scalar`]: () => {
      return FP.abstract.generateVectorScalarToVectorCases(
        sparseVectorF64Range(dim),
        sparseF64Range(),
        'finite',
        additionVectorScalarInterval
      );
    },
  }))
  .reduce((a, b) => ({ ...a, ...b }), {});

const scalar_vector_cases = [2, 3, 4]
  .map(dim => ({
    [`scalar_vec${dim}`]: () => {
      return FP.abstract.generateScalarVectorToVectorCases(
        sparseF64Range(),
        sparseVectorF64Range(dim),
        'finite',
        additionScalarVectorInterval
      );
    },
  }))
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/af_addition', {
  ...scalar_cases,
  ...vector_scalar_cases,
  ...scalar_vector_cases,
});

g.test('scalar')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x + y, where x and y are scalars
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', onlyConstInputSource))
  .fn(async t => {
    const cases = await d.get('scalar');
    await run(
      t,
      abstractBinary('+'),
      [TypeAbstractFloat, TypeAbstractFloat],
      TypeAbstractFloat,
      t.params,
      cases
    );
  });

g.test('vector')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x + y, where x and y are vectors
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', onlyConstInputSource).combine('vectorize', [2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('scalar'); // Using vectorize to generate vector cases based on scalar cases
    await run(
      t,
      abstractBinary('+'),
      [TypeAbstractFloat, TypeAbstractFloat],
      TypeAbstractFloat,
      t.params,
      cases
    );
  });

g.test('vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x + y, where x is a vector and y is a scalar
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', onlyConstInputSource).combine('dim', [2, 3, 4]))
  .fn(async t => {
    const dim = t.params.dim;
    const cases = await d.get(`vec${dim}_scalar`);
    await run(
      t,
      abstractBinary('+'),
      [TypeVec(dim, TypeAbstractFloat), TypeAbstractFloat],
      TypeVec(dim, TypeAbstractFloat),
      t.params,
      cases
    );
  });

g.test('scalar_vector')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x + y, where x is a scalar and y is a vector
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', onlyConstInputSource).combine('dim', [2, 3, 4]))
  .fn(async t => {
    const dim = t.params.dim;
    const cases = await d.get(`scalar_vec${dim}`);
    await run(
      t,
      abstractBinary('+'),
      [TypeAbstractFloat, TypeVec(dim, TypeAbstractFloat)],
      TypeVec(dim, TypeAbstractFloat),
      t.params,
      cases
    );
  });
