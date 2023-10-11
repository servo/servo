/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for non-matrix f32 division expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF32, TypeVec } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseF32Range, sparseVectorF32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

const divisionVectorScalarInterval = (v, s) => {
  return FP.f32.toVector(v.map(e => FP.f32.divisionInterval(e, s)));
};

const divisionScalarVectorInterval = (s, v) => {
  return FP.f32.toVector(v.map(e => FP.f32.divisionInterval(s, e)));
};

export const g = makeTestGroup(GPUTest);

const scalar_cases = [true, false]
  .map(nonConst => ({
    [`scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
      return FP.f32.generateScalarPairToIntervalCases(
        sparseF32Range(),
        sparseF32Range(),
        nonConst ? 'unfiltered' : 'finite',
        FP.f32.divisionInterval
      );
    },
  }))
  .reduce((a, b) => ({ ...a, ...b }), {});

const vector_scalar_cases = [2, 3, 4]
  .flatMap(dim =>
    [true, false].map(nonConst => ({
      [`vec${dim}_scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
        return FP.f32.generateVectorScalarToVectorCases(
          sparseVectorF32Range(dim),
          sparseF32Range(),
          nonConst ? 'unfiltered' : 'finite',
          divisionVectorScalarInterval
        );
      },
    }))
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

const scalar_vector_cases = [2, 3, 4]
  .flatMap(dim =>
    [true, false].map(nonConst => ({
      [`scalar_vec${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
        return FP.f32.generateScalarVectorToVectorCases(
          sparseF32Range(),
          sparseVectorF32Range(dim),
          nonConst ? 'unfiltered' : 'finite',
          divisionScalarVectorInterval
        );
      },
    }))
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f32_division', {
  ...scalar_cases,
  ...vector_scalar_cases,
  ...scalar_vector_cases,
});

g.test('scalar')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x / y, where x and y are scalars
Accuracy: 2.5 ULP for |y| in the range [2^-126, 2^126]
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const'
    );

    await run(t, binary('/'), [TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('vector')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x / y, where x and y are vectors
Accuracy: 2.5 ULP for |y| in the range [2^-126, 2^126]
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const' // Using vectorize to generate vector cases based on scalar cases
    );
    await run(t, binary('/'), [TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x /= y
Accuracy: 2.5 ULP for |y| in the range [2^-126, 2^126]
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const'
    );

    await run(t, compoundBinary('/='), [TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x / y, where x is a vector and y is a scalar
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4]))
  .fn(async t => {
    const dim = t.params.dim;
    const cases = await d.get(
      t.params.inputSource === 'const' ? `vec${dim}_scalar_const` : `vec${dim}_scalar_non_const`
    );

    await run(
      t,
      binary('/'),
      [TypeVec(dim, TypeF32), TypeF32],
      TypeVec(dim, TypeF32),
      t.params,
      cases
    );
  });

g.test('vector_scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x /= y, where x is a vector and y is a scalar
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4]))
  .fn(async t => {
    const dim = t.params.dim;
    const cases = await d.get(
      t.params.inputSource === 'const' ? `vec${dim}_scalar_const` : `vec${dim}_scalar_non_const`
    );

    await run(
      t,
      compoundBinary('/='),
      [TypeVec(dim, TypeF32), TypeF32],
      TypeVec(dim, TypeF32),
      t.params,
      cases
    );
  });

g.test('scalar_vector')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x / y, where x is a scalar and y is a vector
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4]))
  .fn(async t => {
    const dim = t.params.dim;
    const cases = await d.get(
      t.params.inputSource === 'const' ? `scalar_vec${dim}_const` : `scalar_vec${dim}_non_const`
    );

    await run(
      t,
      binary('/'),
      [TypeF32, TypeVec(dim, TypeF32)],
      TypeVec(dim, TypeF32),
      t.params,
      cases
    );
  });
