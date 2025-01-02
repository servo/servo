/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the bitwise binary expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { assert } from '../../../../../common/util/util.js';
import { GPUTest } from '../../../../gpu_test.js';
import {
  abstractIntBits,
  i32Bits,

  scalarType,
  u32Bits } from
'../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../expression.js';

import { abstractIntBinary, binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

/**
 * Collection of functions and values required to implement bitwise tests for a
 * specific scalar type
 */







const kScalarImpls = {
  i32: {
    builder: (bits) => {
      assert(typeof bits === 'number');
      return i32Bits(bits);
    },
    size: 32
  },
  u32: {
    builder: (bits) => {
      assert(typeof bits === 'number');
      return u32Bits(bits);
    },
    size: 32
  },
  'abstract-int': {
    builder: (bits) => {
      assert(typeof bits === 'bigint');
      return abstractIntBits(bits);
    },
    size: 64
  }
};

/** Wrapper for converting from input type strings to the appropriate implementation */
function scalarImplForInputType(inputType) {
  assert(inputType === 'i32' || inputType === 'u32' || inputType === 'abstract-int');
  return kScalarImpls[inputType];
}

/** Manually calculated bitwise-or cases used a check that the CTS test is correct */
const kBitwiseOrStaticPatterns = {
  32: [
  {
    input: [0b00000000000000000000000000000000, 0b00000000000000000000000000000000],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b11111111111111111111111111111111, 0b00000000000000000000000000000000],
    expected: 0b11111111111111111111111111111111
  },
  {
    input: [0b00000000000000000000000000000000, 0b11111111111111111111111111111111],
    expected: 0b11111111111111111111111111111111
  },
  {
    input: [0b11111111111111111111111111111111, 0b11111111111111111111111111111111],
    expected: 0b11111111111111111111111111111111
  },
  {
    input: [0b10100100010010100100010010100100, 0b00000000000000000000000000000000],
    expected: 0b10100100010010100100010010100100
  },
  {
    input: [0b00000000000000000000000000000000, 0b10100100010010100100010010100100],
    expected: 0b10100100010010100100010010100100
  },
  {
    input: [0b01010010001001010010001001010010, 0b10100100010010100100010010100100],
    expected: 0b11110110011011110110011011110110
  }],

  64: [
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b1111111111111111111111111111111111111111111111111111111111111111n
  },
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b1111111111111111111111111111111111111111111111111111111111111111n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b1111111111111111111111111111111111111111111111111111111111111111n
  },
  {
    input: [
    0b1010010001001010010001001010010010100100010010100100010010100100n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b1010010001001010010001001010010010100100010010100100010010100100n
  },
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b1010010001001010010001001010010010100100010010100100010010100100n],

    expected: 0b1010010001001010010001001010010010100100010010100100010010100100n
  },
  {
    input: [
    0b0101001000100101001000100101001010100100010010100100010010100100n,
    0b1010010001001010010001001010010010100100010010100100010010100100n],

    expected: 0b1111011001101111011001101111011010100100010010100100010010100100n
  }]

};

/** @returns a set of bitwise-or cases for the specific input type */
function makeBitwiseOrCases(inputType) {
  const impl = scalarImplForInputType(inputType);
  const indices =
  impl.size === 64 ? [...Array(impl.size).keys()].map(BigInt) : [...Array(impl.size).keys()];

  return [
  ...kBitwiseOrStaticPatterns[impl.size].map((c) => {
    return {
      input: c.input.map(impl.builder),
      expected: impl.builder(c.expected)
    };
  }),
  // Permute all combinations of a single bit being set for the LHS and RHS
  ...indices.flatMap((i) => {
    const lhs = typeof i === 'bigint' ? 1n << i : 1 << i;
    return indices.map((j) => {
      const rhs = typeof j === 'bigint' ? 1n << j : 1 << j;
      assert(typeof lhs === typeof rhs);
      const result = typeof lhs === 'bigint' ? lhs | rhs : lhs | rhs;
      return { input: [impl.builder(lhs), impl.builder(rhs)], expected: impl.builder(result) };
    });
  })];

}

g.test('bitwise_or').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 | e2: T
T is i32, u32, abstractInt, vecN<i32>, vecN<u32>, or vecN<abstractInt>

Bitwise-or. Component-wise when T is a vector.
`
).
params((u) =>
u.
combine('type', ['i32', 'u32', 'abstract-int']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  t.skipIf(
    t.params.type === 'abstract-int' && !onlyConstInputSource.includes(t.params.inputSource)
  );
  const type = scalarType(t.params.type);
  const cases = makeBitwiseOrCases(t.params.type);
  const builder = t.params.type === 'abstract-int' ? abstractIntBinary('|') : binary('|');
  await run(t, builder, [type, type], type, t.params, cases);
});

g.test('bitwise_or_compound').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 |= e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-or. Component-wise when T is a vector.
`
).
params((u) =>
u.
combine('type', ['i32', 'u32']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const type = scalarType(t.params.type);
  const cases = makeBitwiseOrCases(t.params.type);

  await run(t, compoundBinary('|='), [type, type], type, t.params, cases);
});

/** Manually calculated bitwise-and cases used a check that the CTS test is correct */
const kBitwiseAndStaticPatterns = {
  32: [
  {
    input: [0b00000000000000000000000000000000, 0b00000000000000000000000000000000],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b11111111111111111111111111111111, 0b00000000000000000000000000000000],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b00000000000000000000000000000000, 0b11111111111111111111111111111111],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b11111111111111111111111111111111, 0b11111111111111111111111111111111],
    expected: 0b11111111111111111111111111111111
  },
  {
    input: [0b10100100010010100100010010100100, 0b00000000000000000000000000000000],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b10100100010010100100010010100100, 0b11111111111111111111111111111111],
    expected: 0b10100100010010100100010010100100
  },
  {
    input: [0b00000000000000000000000000000000, 0b10100100010010100100010010100100],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b11111111111111111111111111111111, 0b10100100010010100100010010100100],
    expected: 0b10100100010010100100010010100100
  },
  {
    input: [0b01010010001001010010001001010010, 0b01011011101101011011101101011011],
    expected: 0b01010010001001010010001001010010
  }],

  64: [
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b1111111111111111111111111111111111111111111111111111111111111111n
  },
  {
    input: [
    0b1010010001001010010001001010010010100100010010100100010010100100n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b1010010001001010010001001010010010100100010010100100010010100100n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b1010010001001010010001001010010010100100010010100100010010100100n
  },
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b1010010001001010010001001010010010100100010010100100010010100100n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b1010010001001010010001001010010010100100010010100100010010100100n],

    expected: 0b1010010001001010010001001010010010100100010010100100010010100100n
  },
  {
    input: [
    0b0101001000100101001000100101001001010010001001010010001001010010n,
    0b0101101110110101101110110101101101011011101101011011101101011011n],

    expected: 0b0101001000100101001000100101001001010010001001010010001001010010n
  }]

};

/** @returns a set of bitwise-or cases for the specific input type */
function makeBitwiseAndCases(inputType) {
  const impl = scalarImplForInputType(inputType);
  const indices =
  impl.size === 64 ? [...Array(impl.size).keys()].map(BigInt) : [...Array(impl.size).keys()];

  return [
  ...kBitwiseAndStaticPatterns[impl.size].map((c) => {
    return {
      input: c.input.map(impl.builder),
      expected: impl.builder(c.expected)
    };
  }),
  // Permute all combinations of a single bit being set for the LHS and all but one bit set for the RHS
  ...indices.flatMap((i) => {
    const lhs = typeof i === 'bigint' ? 1n << i : 1 << i;
    return indices.map((j) => {
      const rhs = typeof j === 'bigint' ? 0xffffffffffffffffn ^ 1n << j : 0xffffffff ^ 1 << j;
      assert(typeof lhs === typeof rhs);
      const result = typeof lhs === 'bigint' ? lhs & rhs : lhs & rhs;
      return { input: [impl.builder(lhs), impl.builder(rhs)], expected: impl.builder(result) };
    });
  })];

}

g.test('bitwise_and').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 & e2: T
T is i32, u32, AbstractInt, vecN<i32>, vecN<u32>, or vecN<AbstractInt>

Bitwise-and. Component-wise when T is a vector.
`
).
params((u) =>
u.
combine('type', ['i32', 'u32', 'abstract-int']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  t.skipIf(
    t.params.type === 'abstract-int' && !onlyConstInputSource.includes(t.params.inputSource)
  );
  const type = scalarType(t.params.type);
  const cases = makeBitwiseAndCases(t.params.type);
  const builder = t.params.type === 'abstract-int' ? abstractIntBinary('&') : binary('&');
  await run(t, builder, [type, type], type, t.params, cases);
});

g.test('bitwise_and_compound').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 &= e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-and. Component-wise when T is a vector.
`
).
params((u) =>
u.
combine('type', ['i32', 'u32']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const type = scalarType(t.params.type);
  const cases = makeBitwiseAndCases(t.params.type);
  await run(t, compoundBinary('&='), [type, type], type, t.params, cases);
});

/** Manually calculated bitwise-or cases used a check that the CTS test is correct */
const kBitwiseExclusiveOrStaticPatterns = {
  32: [
  {
    input: [0b00000000000000000000000000000000, 0b00000000000000000000000000000000],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b11111111111111111111111111111111, 0b00000000000000000000000000000000],
    expected: 0b11111111111111111111111111111111
  },
  {
    input: [0b00000000000000000000000000000000, 0b11111111111111111111111111111111],
    expected: 0b11111111111111111111111111111111
  },
  {
    input: [0b11111111111111111111111111111111, 0b11111111111111111111111111111111],
    expected: 0b00000000000000000000000000000000
  },
  {
    input: [0b10100100010010100100010010100100, 0b00000000000000000000000000000000],
    expected: 0b10100100010010100100010010100100
  },
  {
    input: [0b10100100010010100100010010100100, 0b11111111111111111111111111111111],
    expected: 0b01011011101101011011101101011011
  },
  {
    input: [0b00000000000000000000000000000000, 0b10100100010010100100010010100100],
    expected: 0b10100100010010100100010010100100
  },
  {
    input: [0b11111111111111111111111111111111, 0b10100100010010100100010010100100],
    expected: 0b01011011101101011011101101011011
  },
  {
    input: [0b01010010001001010010001001010010, 0b01011011101101011011101101011011],
    expected: 0b00001001100100001001100100001001
  }],

  64: [
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b1111111111111111111111111111111111111111111111111111111111111111n
  },
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b1111111111111111111111111111111111111111111111111111111111111111n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b0000000000000000000000000000000000000000000000000000000000000000n
  },
  {
    input: [
    0b1010010001001010010001001010010010100100010010100100010010100100n,
    0b0000000000000000000000000000000000000000000000000000000000000000n],

    expected: 0b1010010001001010010001001010010010100100010010100100010010100100n
  },
  {
    input: [
    0b1010010001001010010001001010010010100100010010100100010010100100n,
    0b1111111111111111111111111111111111111111111111111111111111111111n],

    expected: 0b0101101110110101101110110101101101011011101101011011101101011011n
  },
  {
    input: [
    0b0000000000000000000000000000000000000000000000000000000000000000n,
    0b1010010001001010010001001010010010100100010010100100010010100100n],

    expected: 0b1010010001001010010001001010010010100100010010100100010010100100n
  },
  {
    input: [
    0b1111111111111111111111111111111111111111111111111111111111111111n,
    0b1010010001001010010001001010010010100100010010100100010010100100n],

    expected: 0b0101101110110101101110110101101101011011101101011011101101011011n
  },
  {
    input: [
    0b0101001000100101001000100101001001010010001001010010001001010010n,
    0b0101101110110101101110110101101101011011101101011011101101011011n],

    expected: 0b0000100110010000100110010000100100001001100100001001100100001001n
  }]

};

/** @returns a set of bitwise-xor cases for the specific input type */
function makeBitwiseExclusiveOrCases(inputType) {
  const impl = scalarImplForInputType(inputType);
  const indices =
  impl.size === 64 ? [...Array(impl.size).keys()].map(BigInt) : [...Array(impl.size).keys()];

  return [
  ...kBitwiseExclusiveOrStaticPatterns[impl.size].map((c) => {
    return {
      input: c.input.map(impl.builder),
      expected: impl.builder(c.expected)
    };
  }),
  // Permute all combinations of a single bit being set for the LHS and all but one bit set for the RHS
  ...indices.flatMap((i) => {
    const lhs = typeof i === 'bigint' ? 1n << i : 1 << i;
    return indices.map((j) => {
      const rhs = typeof j === 'bigint' ? 0xffffffffffffffffn ^ 1n << j : 0xffffffff ^ 1 << j;
      assert(typeof lhs === typeof rhs);
      const result = typeof lhs === 'bigint' ? lhs ^ rhs : lhs ^ rhs;
      return { input: [impl.builder(lhs), impl.builder(rhs)], expected: impl.builder(result) };
    });
  })];

}

g.test('bitwise_exclusive_or').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 ^ e2: T
T is i32, u32, abstractInt, vecN<i32>, vecN<u32>, or vecN<abstractInt>

Bitwise-exclusive-or. Component-wise when T is a vector.
`
).
params((u) =>
u.
combine('type', ['i32', 'u32', 'abstract-int']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  t.skipIf(
    t.params.type === 'abstract-int' && !onlyConstInputSource.includes(t.params.inputSource)
  );
  const type = scalarType(t.params.type);
  const cases = makeBitwiseExclusiveOrCases(t.params.type);
  const builder = t.params.type === 'abstract-int' ? abstractIntBinary('^') : binary('^');
  await run(t, builder, [type, type], type, t.params, cases);
});

g.test('bitwise_exclusive_or_compound').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
e1 ^= e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-exclusive-or. Component-wise when T is a vector.
`
).
params((u) =>
u.
combine('type', ['i32', 'u32']).
combine('inputSource', allInputSources).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const type = scalarType(t.params.type);
  const cases = makeBitwiseExclusiveOrCases(t.params.type);
  await run(t, compoundBinary('^='), [type, type], type, t.params, cases);
});