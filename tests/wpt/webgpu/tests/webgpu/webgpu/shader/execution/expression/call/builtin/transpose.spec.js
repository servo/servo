/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'transpose' builtin function

T is AbstractFloat, f32, or f16
@const transpose(e: matRxC<T> ) -> matCxR<T>
Returns the transpose of e.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeMat } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseMatrixF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('transpose', {
  f32_mat2x2_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat2x2_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat2x3_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(2, 3),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat2x3_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(2, 3),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat2x4_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(2, 4),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat2x4_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(2, 4),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat3x2_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(3, 2),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat3x2_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(3, 2),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat3x3_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(3, 3),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat3x3_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(3, 3),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat3x4_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(3, 4),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat3x4_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(3, 4),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat4x2_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(4, 2),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat4x2_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(4, 2),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat4x3_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(4, 3),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat4x3_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(4, 3),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
  f32_mat4x4_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(4, 4),
      'finite',
      FP.f32.transposeInterval
    );
  },
  f32_mat4x4_non_const: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(4, 4),
      'unfiltered',
      FP.f32.transposeInterval
    );
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`abstract float tests`)
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`f32 tests`)
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
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
      builtin('transpose'),
      [TypeMat(cols, rows, TypeF32)],
      TypeMat(rows, cols, TypeF32),
      t.params,
      cases
    );
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`f16 tests`)
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
  .unimplemented();
