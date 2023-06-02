/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the bitwise binary expression operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { i32, scalarType, u32 } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

function makeBitwiseOrCases(inputType) {
  const V = inputType === 'i32' ? i32 : u32;
  const cases = [
    // Static patterns
    {
      input: [V(0b00000000000000000000000000000000), V(0b00000000000000000000000000000000)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b00000000000000000000000000000000)],
      expected: V(0b11111111111111111111111111111111),
    },
    {
      input: [V(0b00000000000000000000000000000000), V(0b11111111111111111111111111111111)],
      expected: V(0b11111111111111111111111111111111),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b11111111111111111111111111111111)],
      expected: V(0b11111111111111111111111111111111),
    },
    {
      input: [V(0b10100100010010100100010010100100), V(0b00000000000000000000000000000000)],
      expected: V(0b10100100010010100100010010100100),
    },
    {
      input: [V(0b00000000000000000000000000000000), V(0b10100100010010100100010010100100)],
      expected: V(0b10100100010010100100010010100100),
    },
    {
      input: [V(0b01010010001001010010001001010010), V(0b10100100010010100100010010100100)],
      expected: V(0b11110110011011110110011011110110),
    },
  ];

  // Permute all combinations of a single bit being set for the LHS and RHS
  for (let i = 0; i < 32; i++) {
    const lhs = 1 << i;
    for (let j = 0; j < 32; j++) {
      const rhs = 1 << j;
      cases.push({
        input: [V(lhs), V(rhs)],
        expected: V(lhs | rhs),
      });
    }
  }
  return cases;
}

g.test('bitwise_or')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 | e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-or. Component-wise when T is a vector.
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeBitwiseOrCases(t.params.type);

    await run(t, binary('|'), [type, type], type, t.params, cases);
  });

g.test('bitwise_or_compound')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 |= e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-or. Component-wise when T is a vector.
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeBitwiseOrCases(t.params.type);

    await run(t, compoundBinary('|='), [type, type], type, t.params, cases);
  });

function makeBitwiseAndCases(inputType) {
  const V = inputType === 'i32' ? i32 : u32;
  const cases = [
    // Static patterns
    {
      input: [V(0b00000000000000000000000000000000), V(0b00000000000000000000000000000000)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b00000000000000000000000000000000)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b00000000000000000000000000000000), V(0b11111111111111111111111111111111)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b11111111111111111111111111111111)],
      expected: V(0b11111111111111111111111111111111),
    },
    {
      input: [V(0b10100100010010100100010010100100), V(0b00000000000000000000000000000000)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b10100100010010100100010010100100), V(0b11111111111111111111111111111111)],
      expected: V(0b10100100010010100100010010100100),
    },
    {
      input: [V(0b00000000000000000000000000000000), V(0b10100100010010100100010010100100)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b10100100010010100100010010100100)],
      expected: V(0b10100100010010100100010010100100),
    },
    {
      input: [V(0b01010010001001010010001001010010), V(0b01011011101101011011101101011011)],
      expected: V(0b01010010001001010010001001010010),
    },
  ];

  // Permute all combinations of a single bit being set for the LHS and all but one bit set for the RHS
  for (let i = 0; i < 32; i++) {
    const lhs = 1 << i;
    for (let j = 0; j < 32; j++) {
      const rhs = 0xffffffff ^ (1 << j);
      cases.push({
        input: [V(lhs), V(rhs)],
        expected: V(lhs & rhs),
      });
    }
  }
  return cases;
}

g.test('bitwise_and')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 & e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-and. Component-wise when T is a vector.
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeBitwiseAndCases(t.params.type);
    await run(t, binary('&'), [type, type], type, t.params, cases);
  });

g.test('bitwise_and_compound')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 &= e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-and. Component-wise when T is a vector.
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeBitwiseAndCases(t.params.type);
    await run(t, compoundBinary('&='), [type, type], type, t.params, cases);
  });

function makeBitwiseExcluseOrCases(inputType) {
  const V = inputType === 'i32' ? i32 : u32;
  const cases = [
    // Static patterns
    {
      input: [V(0b00000000000000000000000000000000), V(0b00000000000000000000000000000000)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b00000000000000000000000000000000)],
      expected: V(0b11111111111111111111111111111111),
    },
    {
      input: [V(0b00000000000000000000000000000000), V(0b11111111111111111111111111111111)],
      expected: V(0b11111111111111111111111111111111),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b11111111111111111111111111111111)],
      expected: V(0b00000000000000000000000000000000),
    },
    {
      input: [V(0b10100100010010100100010010100100), V(0b00000000000000000000000000000000)],
      expected: V(0b10100100010010100100010010100100),
    },
    {
      input: [V(0b10100100010010100100010010100100), V(0b11111111111111111111111111111111)],
      expected: V(0b01011011101101011011101101011011),
    },
    {
      input: [V(0b00000000000000000000000000000000), V(0b10100100010010100100010010100100)],
      expected: V(0b10100100010010100100010010100100),
    },
    {
      input: [V(0b11111111111111111111111111111111), V(0b10100100010010100100010010100100)],
      expected: V(0b01011011101101011011101101011011),
    },
    {
      input: [V(0b01010010001001010010001001010010), V(0b01011011101101011011101101011011)],
      expected: V(0b00001001100100001001100100001001),
    },
  ];

  // Permute all combinations of a single bit being set for the LHS and all but one bit set for the RHS
  for (let i = 0; i < 32; i++) {
    const lhs = 1 << i;
    for (let j = 0; j < 32; j++) {
      const rhs = 0xffffffff ^ (1 << j);
      cases.push({
        input: [V(lhs), V(rhs)],
        expected: V(lhs ^ rhs),
      });
    }
  }
  return cases;
}

g.test('bitwise_exclusive_or')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 ^ e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-exclusive-or. Component-wise when T is a vector.
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeBitwiseExcluseOrCases(t.params.type);
    await run(t, binary('^'), [type, type], type, t.params, cases);
  });

g.test('bitwise_exclusive_or_compound')
  .specURL('https://www.w3.org/TR/WGSL/#bit-expr')
  .desc(
    `
e1 ^= e2: T
T is i32, u32, vecN<i32>, or vecN<u32>

Bitwise-exclusive-or. Component-wise when T is a vector.
`
  )
  .params(u =>
    u
      .combine('type', ['i32', 'u32'])
      .combine('inputSource', allInputSources)
      .combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const type = scalarType(t.params.type);
    const cases = makeBitwiseExcluseOrCases(t.params.type);
    await run(t, compoundBinary('^='), [type, type], type, t.params, cases);
  });
