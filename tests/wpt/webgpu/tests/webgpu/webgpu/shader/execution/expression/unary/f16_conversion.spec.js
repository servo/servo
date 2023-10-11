/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the f32 conversion operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import {
  bool,
  f16,
  i32,
  TypeBool,
  TypeF32,
  TypeF16,
  TypeI32,
  TypeMat,
  TypeU32,
  u32,
} from '../../../../util/conversion.js';
import { FP, FPInterval } from '../../../../util/floating_point.js';
import {
  fullF32Range,
  fullF16Range,
  fullI32Range,
  fullU32Range,
  sparseMatrixF32Range,
  sparseMatrixF16Range,
} from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

const f16FiniteRangeInterval = new FPInterval(
  'f32',
  FP.f16.constants().negative.min,
  FP.f16.constants().positive.max
);

// Cases: f32_matCxR_[non_]const
// Note that f32 values may be not exactly representable in f16 and/or out of range.
const f32_mat_cases = [2, 3, 4]
  .flatMap(cols =>
    [2, 3, 4].flatMap(rows =>
      [true, false].map(nonConst => ({
        [`f32_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
          return FP.f32.generateMatrixToMatrixCases(
            sparseMatrixF32Range(cols, rows),
            nonConst ? 'unfiltered' : 'finite',
            FP.f16.correctlyRoundedMatrix
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

// Cases: f16_matCxR_[non_]const
const f16_mat_cases = [2, 3, 4]
  .flatMap(cols =>
    [2, 3, 4].flatMap(rows =>
      [true, false].map(nonConst => ({
        [`f16_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
          // Input matrix is of f16 types, use f16.generateMatrixToMatrixCases.
          return FP.f16.generateMatrixToMatrixCases(
            sparseMatrixF16Range(cols, rows),
            nonConst ? 'unfiltered' : 'finite',
            FP.f16.correctlyRoundedMatrix
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('unary/f16_conversion', {
  bool: () => {
    return [
      { input: bool(true), expected: f16(1.0) },
      { input: bool(false), expected: f16(0.0) },
    ];
  },
  u32_non_const: () => {
    return [...fullU32Range(), 65504].map(u => {
      return { input: u32(u), expected: FP.f16.correctlyRoundedInterval(u) };
    });
  },
  u32_const: () => {
    return [...fullU32Range(), 65504]
      .filter(v => f16FiniteRangeInterval.contains(v))
      .map(u => {
        return { input: u32(u), expected: FP.f16.correctlyRoundedInterval(u) };
      });
  },
  i32_non_const: () => {
    return [...fullI32Range(), 65504, -65504].map(i => {
      return { input: i32(i), expected: FP.f16.correctlyRoundedInterval(i) };
    });
  },
  i32_const: () => {
    return [...fullI32Range(), 65504, -65504]
      .filter(v => f16FiniteRangeInterval.contains(v))
      .map(i => {
        return { input: i32(i), expected: FP.f16.correctlyRoundedInterval(i) };
      });
  },
  // Note that f32 values may be not exactly representable in f16 and/or out of range.
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [...fullF32Range(), 65535.996, -65535.996],
      'unfiltered',
      FP.f16.correctlyRoundedInterval
    );
  },
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [...fullF32Range(), 65535.996, -65535.996],
      'finite',
      FP.f16.correctlyRoundedInterval
    );
  },
  // All f16 values are exactly representable in f16.
  f16: () => {
    return fullF16Range().map(f => {
      return { input: f16(f), expected: FP.f16.correctlyRoundedInterval(f) };
    });
  },
  ...f32_mat_cases,
  ...f16_mat_cases,
});

/** Generate a ShaderBuilder based on how the test case is to be vectorized */
function vectorizeToExpression(vectorize) {
  return vectorize === undefined ? unary('f16') : unary(`vec${vectorize}<f16>`);
}

/** Generate a ShaderBuilder for a matrix of the provided dimensions */
function matrixExperession(cols, rows) {
  return unary(`mat${cols}x${rows}<f16>`);
}

g.test('bool')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
f16(e), where e is a bool

The result is 1.0 if e is true and 0.0 otherwise
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get('bool');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeBool], TypeF16, t.params, cases);
  });

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#bool-builtin')
  .desc(
    `
f16(e), where e is a u32

Converted to f16, +/-Inf if out of range
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'u32_const' : 'u32_non_const');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeU32], TypeF16, t.params, cases);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
f16(e), where e is a i32

Converted to f16, +/-Inf if out of range
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'i32_const' : 'i32_non_const');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeI32], TypeF16, t.params, cases);
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
f16(e), where e is a f32

Correctly rounded to f16
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeF32], TypeF16, t.params, cases);
  });

g.test('f32_mat')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`f32 matrix to f16 matrix tests`)
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `f32_mat${cols}x${rows}_const`
        : `f32_mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      matrixExperession(cols, rows),
      [TypeMat(cols, rows, TypeF32)],
      TypeMat(cols, rows, TypeF16),
      t.params,
      cases
    );
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
  f16(e), where e is a f16

  Identical.
  `
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeF16], TypeF16, t.params, cases);
  });

g.test('f16_mat')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`f16 matrix to f16 matrix tests, expected identical`)
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `f16_mat${cols}x${rows}_const`
        : `f16_mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      matrixExperession(cols, rows),
      [TypeMat(cols, rows, TypeF16)],
      TypeMat(cols, rows, TypeF16),
      t.params,
      cases
    );
  });
