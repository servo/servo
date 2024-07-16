/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'extractBits' builtin function

T is u32 or vecN<u32>
@const fn extractBits(e: T, offset: u32, count: u32) -> T
Reads bits from an integer, without sign extension.

When T is a scalar type, then:
  w is the bit width of T
  o = min(offset,w)
  c = min(count, w - o)

The result is 0 if c is 0.
Otherwise, bits 0..c-1 of the result are copied from bits o..o+c-1 of e.
Other bits of the result are 0.
Component-wise when T is a vector.


T is i32 or vecN<i32>
@const fn extractBits(e: T, offset: u32, count: u32) -> T
Reads bits from an integer, with sign extension.

When T is a scalar type, then:
  w is the bit width of T
  o = min(offset,w)
  c = min(count, w - o)

The result is 0 if c is 0.
Otherwise, bits 0..c-1 of the result are copied from bits o..o+c-1 of e.
Other bits of the result are the same as bit c-1 of the result.
Component-wise when T is a vector.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { i32Bits, Type, u32, u32Bits, vec2, vec3, vec4 } from '../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('u32').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`u32 tests`).
params((u) => u.combine('inputSource', allInputSources).combine('width', [1, 2, 3, 4])).
fn(async (t) => {
  const cfg = t.params;

  const T = t.params.width === 1 ? Type.u32 : Type.vec(t.params.width, Type.u32);

  const V = (x, y, z, w) => {
    y = y === undefined ? x : y;
    z = z === undefined ? x : z;
    w = w === undefined ? x : w;

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
  };

  const all_1 = V(0b11111111111111111111111111111111);
  const all_0 = V(0b00000000000000000000000000000000);
  const low_1 = V(0b00000000000000000000000000000001);
  const high_1 = V(0b10000000000000000000000000000000);
  const pattern = V(
    0b00000000000111011100000000000000,
    0b11111111111000000011111111111111,
    0b00000000010101010101000000000000,
    0b00000000001010101010100000000000
  );

  const cases = [
  { input: [all_0, u32(0), u32(32)], expected: all_0 },
  { input: [all_0, u32(1), u32(10)], expected: all_0 },
  { input: [all_0, u32(2), u32(5)], expected: all_0 },
  { input: [all_0, u32(0), u32(1)], expected: all_0 },
  { input: [all_0, u32(31), u32(1)], expected: all_0 },

  { input: [all_1, u32(0), u32(32)], expected: all_1 },
  {
    input: [all_1, u32(1), u32(10)],
    expected: V(0b00000000000000000000001111111111)
  },
  {
    input: [all_1, u32(2), u32(5)],
    expected: V(0b00000000000000000000000000011111)
  },
  { input: [all_1, u32(0), u32(1)], expected: low_1 },
  { input: [all_1, u32(31), u32(1)], expected: low_1 },

  // Patterns
  { input: [pattern, u32(0), u32(32)], expected: pattern },
  {
    input: [pattern, u32(1), u32(31)],
    expected: V(
      0b00000000000011101110000000000000,
      0b01111111111100000001111111111111,
      0b00000000001010101010100000000000,
      0b00000000000101010101010000000000
    )
  },
  {
    input: [pattern, u32(14), u32(18)],
    expected: V(
      0b00000000000000000000000001110111,
      0b00000000000000111111111110000000,
      0b00000000000000000000000101010101,
      0b00000000000000000000000010101010
    )
  },
  {
    input: [pattern, u32(14), u32(7)],
    expected: V(
      0b00000000000000000000000001110111,
      0b00000000000000000000000000000000,
      0b00000000000000000000000001010101,
      0b00000000000000000000000000101010
    )
  },
  {
    input: [pattern, u32(14), u32(4)],
    expected: V(
      0b00000000000000000000000000000111,
      0b00000000000000000000000000000000,
      0b00000000000000000000000000000101,
      0b00000000000000000000000000001010
    )
  },
  {
    input: [pattern, u32(14), u32(3)],
    expected: V(
      0b00000000000000000000000000000111,
      0b00000000000000000000000000000000,
      0b00000000000000000000000000000101,
      0b00000000000000000000000000000010
    )
  },
  {
    input: [pattern, u32(18), u32(3)],
    expected: V(
      0b00000000000000000000000000000111,
      0b00000000000000000000000000000000,
      0b00000000000000000000000000000101,
      0b00000000000000000000000000000010
    )
  },
  { input: [low_1, u32(0), u32(1)], expected: low_1 },
  { input: [high_1, u32(31), u32(1)], expected: low_1 },

  // Zero count
  { input: [all_1, u32(0), u32(0)], expected: all_0 },
  { input: [all_0, u32(0), u32(0)], expected: all_0 },
  { input: [low_1, u32(0), u32(0)], expected: all_0 },
  { input: [high_1, u32(31), u32(0)], expected: all_0 },
  { input: [pattern, u32(0), u32(0)], expected: all_0 }];


  if (t.params.inputSource !== 'const') {
    cases.push(
      ...[
      // End overflow
      { input: [low_1, u32(0), u32(99)], expected: low_1 },
      { input: [high_1, u32(31), u32(99)], expected: low_1 },
      { input: [pattern, u32(0), u32(99)], expected: pattern },
      {
        input: [pattern, u32(14), u32(99)],
        expected: V(
          0b00000000000000000000000001110111,
          0b00000000000000111111111110000000,
          0b00000000000000000000000101010101,
          0b00000000000000000000000010101010
        )
      }]

    );
  }

  await run(t, builtin('extractBits'), [T, Type.u32, Type.u32], T, cfg, cases);
});

g.test('i32').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`i32 tests`).
params((u) => u.combine('inputSource', allInputSources).combine('width', [1, 2, 3, 4])).
fn(async (t) => {
  const cfg = t.params;

  const T = t.params.width === 1 ? Type.i32 : Type.vec(t.params.width, Type.i32);

  const V = (x, y, z, w) => {
    y = y === undefined ? x : y;
    z = z === undefined ? x : z;
    w = w === undefined ? x : w;

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
  };

  const all_1 = V(0b11111111111111111111111111111111);
  const all_0 = V(0b00000000000000000000000000000000);
  const low_1 = V(0b00000000000000000000000000000001);
  const high_1 = V(0b10000000000000000000000000000000);
  const pattern = V(
    0b00000000000111011100000000000000,
    0b11111111111000000011111111111111,
    0b00000000010101010101000000000000,
    0b00000000001010101010100000000000
  );

  const cases = [
  { input: [all_0, u32(0), u32(32)], expected: all_0 },
  { input: [all_0, u32(1), u32(10)], expected: all_0 },
  { input: [all_0, u32(2), u32(5)], expected: all_0 },
  { input: [all_0, u32(0), u32(1)], expected: all_0 },
  { input: [all_0, u32(31), u32(1)], expected: all_0 },

  { input: [all_1, u32(0), u32(32)], expected: all_1 },
  { input: [all_1, u32(1), u32(10)], expected: all_1 },
  { input: [all_1, u32(2), u32(5)], expected: all_1 },
  { input: [all_1, u32(0), u32(1)], expected: all_1 },
  { input: [all_1, u32(31), u32(1)], expected: all_1 },

  // Patterns
  { input: [pattern, u32(0), u32(32)], expected: pattern },
  {
    input: [pattern, u32(1), u32(31)],
    expected: V(
      0b00000000000011101110000000000000,
      0b11111111111100000001111111111111,
      0b00000000001010101010100000000000,
      0b00000000000101010101010000000000
    )
  },
  {
    input: [pattern, u32(14), u32(18)],
    expected: V(
      0b00000000000000000000000001110111,
      0b11111111111111111111111110000000,
      0b00000000000000000000000101010101,
      0b00000000000000000000000010101010
    )
  },
  {
    input: [pattern, u32(14), u32(7)],
    expected: V(
      0b11111111111111111111111111110111,
      0b00000000000000000000000000000000,
      0b11111111111111111111111111010101,
      0b00000000000000000000000000101010
    )
  },
  {
    input: [pattern, u32(14), u32(4)],
    expected: V(
      0b00000000000000000000000000000111,
      0b00000000000000000000000000000000,
      0b00000000000000000000000000000101,
      0b11111111111111111111111111111010
    )
  },
  {
    input: [pattern, u32(14), u32(3)],
    expected: V(
      0b11111111111111111111111111111111,
      0b00000000000000000000000000000000,
      0b11111111111111111111111111111101,
      0b00000000000000000000000000000010
    )
  },
  {
    input: [pattern, u32(18), u32(3)],
    expected: V(
      0b11111111111111111111111111111111,
      0b00000000000000000000000000000000,
      0b11111111111111111111111111111101,
      0b00000000000000000000000000000010
    )
  },
  { input: [low_1, u32(0), u32(1)], expected: all_1 },
  { input: [high_1, u32(31), u32(1)], expected: all_1 },

  // Zero count
  { input: [all_1, u32(0), u32(0)], expected: all_0 },
  { input: [all_0, u32(0), u32(0)], expected: all_0 },
  { input: [low_1, u32(0), u32(0)], expected: all_0 },
  { input: [high_1, u32(31), u32(0)], expected: all_0 },
  { input: [pattern, u32(0), u32(0)], expected: all_0 }];


  if (t.params.inputSource !== 'const') {
    cases.push(
      ...[
      // End overflow
      { input: [low_1, u32(0), u32(99)], expected: low_1 },
      { input: [high_1, u32(31), u32(99)], expected: all_1 },
      { input: [pattern, u32(0), u32(99)], expected: pattern },
      {
        input: [pattern, u32(14), u32(99)],
        expected: V(
          0b00000000000000000000000001110111,
          0b11111111111111111111111110000000,
          0b00000000000000000000000101010101,
          0b00000000000000000000000010101010
        )
      }]

    );
  }

  await run(t, builtin('extractBits'), [T, Type.u32, Type.u32], T, cfg, cases);
});