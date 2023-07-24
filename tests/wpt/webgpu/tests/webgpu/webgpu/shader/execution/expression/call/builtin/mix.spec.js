/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'mix' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn mix(e1: T, e2: T, e3: T) -> T
Returns the linear blend of e1 and e2 (e.g. e1*(1-e3)+e2*e3). Component-wise when T is a vector.

T is AbstractFloat, f32, or f16
T2 is vecN<T>
@const fn mix(e1: T2, e2: T2, e3: T) -> T2
Returns the component-wise linear blend of e1 and e2, using scalar blending factor e3 for each component.
Same as mix(e1,e2,T2(e3)).

`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('mix', {
  f32_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'finite',
      ...FP.f32.mixIntervals
    );
  },
  f32_non_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'unfiltered',
      ...FP.f32.mixIntervals
    );
  },
});

g.test('matching_abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests with matching params`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('matching_f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 test with matching third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('mix'), [TypeF32, TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('matching_f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests with matching third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('nonmatching_abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests with vector params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [2, 3, 4]))
  .unimplemented();

g.test('nonmatching_f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests with vector params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [2, 3, 4]))
  .unimplemented();

g.test('monmatching_f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests with vector params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [2, 3, 4]))
  .unimplemented();
