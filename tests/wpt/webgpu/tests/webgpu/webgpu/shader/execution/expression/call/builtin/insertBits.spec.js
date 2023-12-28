/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'insertBits' builtin function

S is i32 or u32
T is S or vecN<S>
@const fn insertBits(e: T, newbits:T, offset: u32, count: u32) -> T  Sets bits in an integer.

When T is a scalar type, then:
  w is the bit width of T
  o = min(offset,w)
  c = min(count, w - o)

The result is e if c is 0.
Otherwise, bits o..o+c-1 of the result are copied from bits 0..c-1 of newbits.
Other bits of the result are copied from e.
Component-wise when T is a vector.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import {
  i32Bits,
  TypeI32,
  u32,
  TypeU32,
  u32Bits,
  vec2,
  vec3,
  vec4,
  TypeVec } from
'../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('integer').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`integer tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('signed', [false, true]).
combine('width', [1, 2, 3, 4])
).
fn(async (t) => {
  const cfg = t.params;
  const scalarType = t.params.signed ? TypeI32 : TypeU32;
  const T = t.params.width === 1 ? scalarType : TypeVec(t.params.width, scalarType);

  const V = (x, y, z, w) => {
    y = y === undefined ? x : y;
    z = z === undefined ? x : z;
    w = w === undefined ? x : w;

    if (t.params.signed) {
      switch (t.params.width) {
        case 1:
          return i32Bits(x);
        case 2:
          return vec2(i32Bits(x), i32Bits(y));
        case 3:
          return vec3(i32Bits(x), i32Bits(y), i32Bits(z));
        default:
          return vec4(i32Bits(x), i32Bits(y), i32Bits(z), i32Bits(w));
      }
    } else {
      switch (t.params.width) {
        case 1:
          return u32Bits(x);
        case 2:
          return vec2(u32Bits(x), u32Bits(y));
        case 3:
          return vec3(u32Bits(x), u32Bits(y), u32Bits(z));
        default:
          return vec4(u32Bits(x), u32Bits(y), u32Bits(z), u32Bits(w));
      }
    }
  };

  const all_1 = V(0b11111111111111111111111111111111);
  const all_0 = V(0b00000000000000000000000000000000);
  const low_1 = V(0b00000000000000000000000000000001);
  const low_0 = V(0b11111111111111111111111111111110);
  const high_1 = V(0b10000000000000000000000000000000);
  const high_0 = V(0b01111111111111111111111111111111);
  const pattern = V(
    0b10001001010100100010010100100010,
    0b11001110001100111000110011100011,
    0b10101010101010101010101010101010,
    0b01010101010101010101010101010101
  );

  const cases = [
  { input: [all_0, all_0, u32(0), u32(32)], expected: all_0 },
  { input: [all_0, all_0, u32(1), u32(10)], expected: all_0 },
  { input: [all_0, all_0, u32(2), u32(5)], expected: all_0 },
  { input: [all_0, all_0, u32(0), u32(1)], expected: all_0 },
  { input: [all_0, all_0, u32(31), u32(1)], expected: all_0 },

  { input: [all_0, all_1, u32(0), u32(32)], expected: all_1 },
  { input: [all_1, all_0, u32(0), u32(32)], expected: all_0 },
  { input: [all_0, all_1, u32(0), u32(1)], expected: low_1 },
  { input: [all_1, all_0, u32(0), u32(1)], expected: low_0 },
  { input: [all_0, all_1, u32(31), u32(1)], expected: high_1 },
  { input: [all_1, all_0, u32(31), u32(1)], expected: high_0 },
  { input: [all_0, all_1, u32(1), u32(10)], expected: V(0b00000000000000000000011111111110) },
  { input: [all_1, all_0, u32(1), u32(10)], expected: V(0b11111111111111111111100000000001) },
  { input: [all_0, all_1, u32(2), u32(5)], expected: V(0b00000000000000000000000001111100) },
  { input: [all_1, all_0, u32(2), u32(5)], expected: V(0b11111111111111111111111110000011) },

  // Patterns
  { input: [all_0, pattern, u32(0), u32(32)], expected: pattern },
  { input: [all_1, pattern, u32(0), u32(32)], expected: pattern },
  {
    input: [all_0, pattern, u32(1), u32(31)],
    expected: V(
      0b00010010101001000100101001000100,
      0b10011100011001110001100111000110,
      0b01010101010101010101010101010100,
      0b10101010101010101010101010101010
    )
  },
  {
    input: [all_1, pattern, u32(1), u32(31)],
    expected: V(
      0b00010010101001000100101001000101,
      0b10011100011001110001100111000111,
      0b01010101010101010101010101010101,
      0b10101010101010101010101010101011
    )
  },
  {
    input: [all_0, pattern, u32(14), u32(18)],
    expected: V(
      0b10001001010010001000000000000000,
      0b11100011001110001100000000000000,
      0b10101010101010101000000000000000,
      0b01010101010101010100000000000000
    )
  },
  {
    input: [all_1, pattern, u32(14), u32(18)],
    expected: V(
      0b10001001010010001011111111111111,
      0b11100011001110001111111111111111,
      0b10101010101010101011111111111111,
      0b01010101010101010111111111111111
    )
  },
  {
    input: [all_0, pattern, u32(14), u32(7)],
    expected: V(
      0b00000000000010001000000000000000,
      0b00000000000110001100000000000000,
      0b00000000000010101000000000000000,
      0b00000000000101010100000000000000
    )
  },
  {
    input: [all_1, pattern, u32(14), u32(7)],
    expected: V(
      0b11111111111010001011111111111111,
      0b11111111111110001111111111111111,
      0b11111111111010101011111111111111,
      0b11111111111101010111111111111111
    )
  },
  {
    input: [all_0, pattern, u32(14), u32(4)],
    expected: V(
      0b00000000000000001000000000000000,
      0b00000000000000001100000000000000,
      0b00000000000000101000000000000000,
      0b00000000000000010100000000000000
    )
  },
  {
    input: [all_1, pattern, u32(14), u32(4)],
    expected: V(
      0b11111111111111001011111111111111,
      0b11111111111111001111111111111111,
      0b11111111111111101011111111111111,
      0b11111111111111010111111111111111
    )
  },
  {
    input: [all_0, pattern, u32(14), u32(3)],
    expected: V(
      0b00000000000000001000000000000000,
      0b00000000000000001100000000000000,
      0b00000000000000001000000000000000,
      0b00000000000000010100000000000000
    )
  },
  {
    input: [all_1, pattern, u32(14), u32(3)],
    expected: V(
      0b11111111111111101011111111111111,
      0b11111111111111101111111111111111,
      0b11111111111111101011111111111111,
      0b11111111111111110111111111111111
    )
  },
  {
    input: [all_0, pattern, u32(18), u32(3)],
    expected: V(
      0b00000000000010000000000000000000,
      0b00000000000011000000000000000000,
      0b00000000000010000000000000000000,
      0b00000000000101000000000000000000
    )
  },
  {
    input: [all_1, pattern, u32(18), u32(3)],
    expected: V(
      0b11111111111010111111111111111111,
      0b11111111111011111111111111111111,
      0b11111111111010111111111111111111,
      0b11111111111101111111111111111111
    )
  },
  {
    input: [pattern, all_0, u32(1), u32(31)],
    expected: V(
      0b00000000000000000000000000000000,
      0b00000000000000000000000000000001,
      0b00000000000000000000000000000000,
      0b00000000000000000000000000000001
    )
  },
  {
    input: [pattern, all_1, u32(1), u32(31)],
    expected: V(
      0b11111111111111111111111111111110,
      0b11111111111111111111111111111111,
      0b11111111111111111111111111111110,
      0b11111111111111111111111111111111
    )
  },
  {
    input: [pattern, all_0, u32(14), u32(18)],
    expected: V(
      0b00000000000000000010010100100010,
      0b00000000000000000000110011100011,
      0b00000000000000000010101010101010,
      0b00000000000000000001010101010101
    )
  },
  {
    input: [pattern, all_1, u32(14), u32(18)],
    expected: V(
      0b11111111111111111110010100100010,
      0b11111111111111111100110011100011,
      0b11111111111111111110101010101010,
      0b11111111111111111101010101010101
    )
  },
  {
    input: [pattern, all_0, u32(14), u32(7)],
    expected: V(
      0b10001001010000000010010100100010,
      0b11001110001000000000110011100011,
      0b10101010101000000010101010101010,
      0b01010101010000000001010101010101
    )
  },
  {
    input: [pattern, all_1, u32(14), u32(7)],
    expected: V(
      0b10001001010111111110010100100010,
      0b11001110001111111100110011100011,
      0b10101010101111111110101010101010,
      0b01010101010111111101010101010101
    )
  },
  {
    input: [pattern, all_0, u32(14), u32(4)],
    expected: V(
      0b10001001010100000010010100100010,
      0b11001110001100000000110011100011,
      0b10101010101010000010101010101010,
      0b01010101010101000001010101010101
    )
  },
  {
    input: [pattern, all_1, u32(14), u32(4)],
    expected: V(
      0b10001001010100111110010100100010,
      0b11001110001100111100110011100011,
      0b10101010101010111110101010101010,
      0b01010101010101111101010101010101
    )
  },
  {
    input: [pattern, all_0, u32(14), u32(3)],
    expected: V(
      0b10001001010100100010010100100010,
      0b11001110001100100000110011100011,
      0b10101010101010100010101010101010,
      0b01010101010101000001010101010101
    )
  },
  {
    input: [pattern, all_1, u32(14), u32(3)],
    expected: V(
      0b10001001010100111110010100100010,
      0b11001110001100111100110011100011,
      0b10101010101010111110101010101010,
      0b01010101010101011101010101010101
    )
  },
  {
    input: [pattern, all_0, u32(18), u32(3)],
    expected: V(
      0b10001001010000100010010100100010,
      0b11001110001000111000110011100011,
      0b10101010101000101010101010101010,
      0b01010101010000010101010101010101
    )
  },
  {
    input: [pattern, all_1, u32(18), u32(3)],
    expected: V(
      0b10001001010111100010010100100010,
      0b11001110001111111000110011100011,
      0b10101010101111101010101010101010,
      0b01010101010111010101010101010101
    )
  },
  {
    input: [pattern, pattern, u32(18), u32(3)],
    expected: V(
      0b10001001010010100010010100100010,
      0b11001110001011111000110011100011,
      0b10101010101010101010101010101010,
      0b01010101010101010101010101010101
    )
  },
  {
    input: [pattern, pattern, u32(14), u32(7)],
    expected: V(
      0b10001001010010001010010100100010,
      0b11001110001110001100110011100011,
      0b10101010101010101010101010101010,
      0b01010101010101010101010101010101
    )
  },

  // Zero count
  { input: [pattern, all_1, u32(0), u32(0)], expected: pattern },
  { input: [pattern, all_1, u32(1), u32(0)], expected: pattern },
  { input: [pattern, all_1, u32(2), u32(0)], expected: pattern },
  { input: [pattern, all_1, u32(31), u32(0)], expected: pattern },
  { input: [pattern, all_1, u32(32), u32(0)], expected: pattern },
  { input: [pattern, all_1, u32(0), u32(0)], expected: pattern }];


  if (t.params.inputSource !== 'const') {
    cases.push(
      ...[
      // Start overflow
      { input: [all_0, pattern, u32(50), u32(3)], expected: all_0 },
      { input: [all_1, pattern, u32(50), u32(3)], expected: all_1 },
      { input: [pattern, pattern, u32(50), u32(3)], expected: pattern },

      // End overflow
      { input: [all_0, pattern, u32(0), u32(99)], expected: pattern },
      { input: [all_1, pattern, u32(0), u32(99)], expected: pattern },
      { input: [all_0, low_1, u32(31), u32(99)], expected: high_1 },
      {
        input: [pattern, pattern, u32(20), u32(99)],
        expected: V(
          0b01010010001000100010010100100010,
          0b11001110001100111000110011100011,
          0b10101010101010101010101010101010,
          0b01010101010101010101010101010101
        )
      }]

    );
  }

  await run(t, builtin('insertBits'), [T, T, TypeU32, TypeU32], T, cfg, cases);
});