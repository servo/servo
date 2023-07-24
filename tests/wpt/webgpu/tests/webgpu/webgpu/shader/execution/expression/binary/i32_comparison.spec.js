/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the i32 comparison expressions
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { i32, bool, TypeBool, TypeI32 } from '../../../../util/conversion.js';
import { vectorI32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { binary } from './binary.js';

export const g = makeTestGroup(GPUTest);

/**
 * @returns a test case for the provided left hand & right hand values and
 * expected boolean result.
 */
function makeCase(lhs, rhs, expected_answer) {
  return { input: [i32(lhs), i32(rhs)], expected: bool(expected_answer) };
}

export const d = makeCaseCache('binary/i32_comparison', {
  equals: () => vectorI32Range(2).map(v => makeCase(v[0], v[1], v[0] === v[1])),
  not_equals: () => vectorI32Range(2).map(v => makeCase(v[0], v[1], v[0] !== v[1])),
  less_than: () => vectorI32Range(2).map(v => makeCase(v[0], v[1], v[0] < v[1])),
  less_equal: () => vectorI32Range(2).map(v => makeCase(v[0], v[1], v[0] <= v[1])),
  greater_than: () => vectorI32Range(2).map(v => makeCase(v[0], v[1], v[0] > v[1])),
  greater_equal: () => vectorI32Range(2).map(v => makeCase(v[0], v[1], v[0] >= v[1])),
});

g.test('equals')
  .specURL('https://www.w3.org/TR/WGSL/#comparison-expr')
  .desc(
    `
Expression: x == y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('equals');
    await run(t, binary('=='), [TypeI32, TypeI32], TypeBool, t.params, cases);
  });

g.test('not_equals')
  .specURL('https://www.w3.org/TR/WGSL/#comparison-expr')
  .desc(
    `
Expression: x != y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('not_equals');
    await run(t, binary('!='), [TypeI32, TypeI32], TypeBool, t.params, cases);
  });

g.test('less_than')
  .specURL('https://www.w3.org/TR/WGSL/#comparison-expr')
  .desc(
    `
Expression: x < y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('less_than');
    await run(t, binary('<'), [TypeI32, TypeI32], TypeBool, t.params, cases);
  });

g.test('less_equals')
  .specURL('https://www.w3.org/TR/WGSL/#comparison-expr')
  .desc(
    `
Expression: x <= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('less_equal');
    await run(t, binary('<='), [TypeI32, TypeI32], TypeBool, t.params, cases);
  });

g.test('greater_than')
  .specURL('https://www.w3.org/TR/WGSL/#comparison-expr')
  .desc(
    `
Expression: x > y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('greater_than');
    await run(t, binary('>'), [TypeI32, TypeI32], TypeBool, t.params, cases);
  });

g.test('greater_equals')
  .specURL('https://www.w3.org/TR/WGSL/#comparison-expr')
  .desc(
    `
Expression: x >= y
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('greater_equal');
    await run(t, binary('>='), [TypeI32, TypeI32], TypeBool, t.params, cases);
  });
