/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the boolean conversion operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { anyOf } from '../../../../util/compare.js';
import {
  bool,
  f32,
  i32,
  TypeBool,
  TypeF32,
  TypeI32,
  TypeU32,
  u32,
} from '../../../../util/conversion.js';
import {
  fullF32Range,
  fullI32Range,
  fullU32Range,
  isSubnormalNumberF32,
} from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/bool_conversion', {
  bool: () => {
    return [
      { input: bool(true), expected: bool(true) },
      { input: bool(false), expected: bool(false) },
    ];
  },
  u32: () => {
    return fullU32Range().map(u => {
      return { input: u32(u), expected: u === 0 ? bool(false) : bool(true) };
    });
  },
  i32: () => {
    return fullI32Range().map(i => {
      return { input: i32(i), expected: i === 0 ? bool(false) : bool(true) };
    });
  },
  f32: () => {
    return fullF32Range().map(f => {
      const expected = [];
      if (f !== 0) {
        expected.push(bool(true));
      }
      if (isSubnormalNumberF32(f)) {
        expected.push(bool(false));
      }
      return { input: f32(f), expected: anyOf(...expected) };
    });
  },
});

/** Generate expression builder based on how the test case is to be vectorized */
function vectorizeToExpression(vectorize) {
  return vectorize === undefined ? unary('bool') : unary(`vec${vectorize}<bool>`);
}

g.test('bool')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
bool(e), where e is a bool

Identity operation
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('bool');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeBool], TypeBool, t.params, cases);
  });

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#bool-builtin')
  .desc(
    `
bool(e), where e is a u32

Coercion to boolean.
The result is false if e is 0, and true otherwise.
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('u32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeU32], TypeBool, t.params, cases);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
bool(e), where e is a i32

Coercion to boolean.
The result is false if e is 0, and true otherwise.
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('i32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeI32], TypeBool, t.params, cases);
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
bool(e), where e is a f32

Coercion to boolean.
The result is false if e is 0.0 or -0.0, and true otherwise.
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeF32], TypeBool, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
bool(e), where e is a f16

Coercion to boolean.
The result is false if e is 0.0 or -0.0, and true otherwise.
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
