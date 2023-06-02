/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for non-matrix f32 addition expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF32, TypeVec } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseF32Range, sparseVectorF32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

const additionVectorScalarInterval = (v, s) => {
  return FP.f32.toVector(v.map(e => FP.f32.additionInterval(e, s)));
};

const additionScalarVectorInterval = (s, v) => {
  return FP.f32.toVector(v.map(e => FP.f32.additionInterval(s, e)));
};

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('binary/f32_addition', {
  scalar_const: () => {
    return FP.f32.generateScalarPairToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      'finite',
      FP.f32.additionInterval
    );
  },
  scalar_non_const: () => {
    return FP.f32.generateScalarPairToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      'unfiltered',
      FP.f32.additionInterval
    );
  },
  vec2_scalar_const: () => {
    return FP.f32.generateVectorScalarToVectorCases(
      sparseVectorF32Range(2),
      sparseF32Range(),
      'finite',
      additionVectorScalarInterval
    );
  },
  vec2_scalar_non_const: () => {
    return FP.f32.generateVectorScalarToVectorCases(
      sparseVectorF32Range(2),
      sparseF32Range(),
      'unfiltered',
      additionVectorScalarInterval
    );
  },
  vec3_scalar_const: () => {
    return FP.f32.generateVectorScalarToVectorCases(
      sparseVectorF32Range(3),
      sparseF32Range(),
      'finite',
      additionVectorScalarInterval
    );
  },
  vec3_scalar_non_const: () => {
    return FP.f32.generateVectorScalarToVectorCases(
      sparseVectorF32Range(3),
      sparseF32Range(),
      'unfiltered',
      additionVectorScalarInterval
    );
  },
  vec4_scalar_const: () => {
    return FP.f32.generateVectorScalarToVectorCases(
      sparseVectorF32Range(4),
      sparseF32Range(),
      'finite',
      additionVectorScalarInterval
    );
  },
  vec4_scalar_non_const: () => {
    return FP.f32.generateVectorScalarToVectorCases(
      sparseVectorF32Range(4),
      sparseF32Range(),
      'unfiltered',
      additionVectorScalarInterval
    );
  },
  scalar_vec2_const: () => {
    return FP.f32.generateScalarVectorToVectorCases(
      sparseF32Range(),
      sparseVectorF32Range(2),
      'finite',
      additionScalarVectorInterval
    );
  },
  scalar_vec2_non_const: () => {
    return FP.f32.generateScalarVectorToVectorCases(
      sparseF32Range(),
      sparseVectorF32Range(2),
      'unfiltered',
      additionScalarVectorInterval
    );
  },
  scalar_vec3_const: () => {
    return FP.f32.generateScalarVectorToVectorCases(
      sparseF32Range(),
      sparseVectorF32Range(3),
      'finite',
      additionScalarVectorInterval
    );
  },
  scalar_vec3_non_const: () => {
    return FP.f32.generateScalarVectorToVectorCases(
      sparseF32Range(),
      sparseVectorF32Range(3),
      'unfiltered',
      additionScalarVectorInterval
    );
  },
  scalar_vec4_const: () => {
    return FP.f32.generateScalarVectorToVectorCases(
      sparseF32Range(),
      sparseVectorF32Range(4),
      'finite',
      additionScalarVectorInterval
    );
  },
  scalar_vec4_non_const: () => {
    return FP.f32.generateScalarVectorToVectorCases(
      sparseF32Range(),
      sparseVectorF32Range(4),
      'unfiltered',
      additionScalarVectorInterval
    );
  },
  subtraction_const: () => {
    return FP.f32.generateScalarPairToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      'finite',
      FP.f32.subtractionInterval
    );
  },
});

g.test('scalar')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x + y
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const'
    );

    await run(t, binary('+'), [TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x += y
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const'
    );

    await run(t, compoundBinary('+='), [TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('vector_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x + y, where x is a vector and y is a scalar
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
      binary('+'),
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
Expression: x += y, where x is a vector and y is a scalar
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
      compoundBinary('+='),
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
Expression: x + y, where x is a scalar and y is a vector
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
      binary('+'),
      [TypeF32, TypeVec(dim, TypeF32)],
      TypeVec(dim, TypeF32),
      t.params,
      cases
    );
  });
