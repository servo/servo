/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'firstLeadingBit' builtin function

T is u32 or vecN<u32>
@const fn firstLeadingBit(e: T ) -> T
For scalar T, the result is: T(-1) if e is zero.
Otherwise the position of the most significant 1 bit in e.
Component-wise when T is a vector.

T is i32 or vecN<i32>
@const fn firstLeadingBit(e: T ) -> T
For scalar T, the result is: -1 if e is 0 or -1.
Otherwise the position of the most significant bit in e that is different from e’s sign bit.
Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { i32, i32Bits, TypeI32, u32, TypeU32, u32Bits } from '../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`u32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cfg = t.params;
    await run(t, builtin('firstLeadingBit'), [TypeU32], TypeU32, cfg, [
      // Zero
      { input: u32Bits(0b00000000000000000000000000000000), expected: u32(-1) },

      // One
      { input: u32Bits(0b00000000000000000000000000000001), expected: u32(0) },

      // 0's after leading 1
      { input: u32Bits(0b00000000000000000000000000000010), expected: u32(1) },
      { input: u32Bits(0b00000000000000000000000000000100), expected: u32(2) },
      { input: u32Bits(0b00000000000000000000000000001000), expected: u32(3) },
      { input: u32Bits(0b00000000000000000000000000010000), expected: u32(4) },
      { input: u32Bits(0b00000000000000000000000000100000), expected: u32(5) },
      { input: u32Bits(0b00000000000000000000000001000000), expected: u32(6) },
      { input: u32Bits(0b00000000000000000000000010000000), expected: u32(7) },
      { input: u32Bits(0b00000000000000000000000100000000), expected: u32(8) },
      { input: u32Bits(0b00000000000000000000001000000000), expected: u32(9) },
      { input: u32Bits(0b00000000000000000000010000000000), expected: u32(10) },
      { input: u32Bits(0b00000000000000000000100000000000), expected: u32(11) },
      { input: u32Bits(0b00000000000000000001000000000000), expected: u32(12) },
      { input: u32Bits(0b00000000000000000010000000000000), expected: u32(13) },
      { input: u32Bits(0b00000000000000000100000000000000), expected: u32(14) },
      { input: u32Bits(0b00000000000000001000000000000000), expected: u32(15) },
      { input: u32Bits(0b00000000000000010000000000000000), expected: u32(16) },
      { input: u32Bits(0b00000000000000100000000000000000), expected: u32(17) },
      { input: u32Bits(0b00000000000001000000000000000000), expected: u32(18) },
      { input: u32Bits(0b00000000000010000000000000000000), expected: u32(19) },
      { input: u32Bits(0b00000000000100000000000000000000), expected: u32(20) },
      { input: u32Bits(0b00000000001000000000000000000000), expected: u32(21) },
      { input: u32Bits(0b00000000010000000000000000000000), expected: u32(22) },
      { input: u32Bits(0b00000000100000000000000000000000), expected: u32(23) },
      { input: u32Bits(0b00000001000000000000000000000000), expected: u32(24) },
      { input: u32Bits(0b00000010000000000000000000000000), expected: u32(25) },
      { input: u32Bits(0b00000100000000000000000000000000), expected: u32(26) },
      { input: u32Bits(0b00001000000000000000000000000000), expected: u32(27) },
      { input: u32Bits(0b00010000000000000000000000000000), expected: u32(28) },
      { input: u32Bits(0b00100000000000000000000000000000), expected: u32(29) },
      { input: u32Bits(0b01000000000000000000000000000000), expected: u32(30) },
      { input: u32Bits(0b10000000000000000000000000000000), expected: u32(31) },

      // 1's after leading 1
      { input: u32Bits(0b00000000000000000000000000000011), expected: u32(1) },
      { input: u32Bits(0b00000000000000000000000000000111), expected: u32(2) },
      { input: u32Bits(0b00000000000000000000000000001111), expected: u32(3) },
      { input: u32Bits(0b00000000000000000000000000011111), expected: u32(4) },
      { input: u32Bits(0b00000000000000000000000000111111), expected: u32(5) },
      { input: u32Bits(0b00000000000000000000000001111111), expected: u32(6) },
      { input: u32Bits(0b00000000000000000000000011111111), expected: u32(7) },
      { input: u32Bits(0b00000000000000000000000111111111), expected: u32(8) },
      { input: u32Bits(0b00000000000000000000001111111111), expected: u32(9) },
      { input: u32Bits(0b00000000000000000000011111111111), expected: u32(10) },
      { input: u32Bits(0b00000000000000000000111111111111), expected: u32(11) },
      { input: u32Bits(0b00000000000000000001111111111111), expected: u32(12) },
      { input: u32Bits(0b00000000000000000011111111111111), expected: u32(13) },
      { input: u32Bits(0b00000000000000000111111111111111), expected: u32(14) },
      { input: u32Bits(0b00000000000000001111111111111111), expected: u32(15) },
      { input: u32Bits(0b00000000000000011111111111111111), expected: u32(16) },
      { input: u32Bits(0b00000000000000111111111111111111), expected: u32(17) },
      { input: u32Bits(0b00000000000001111111111111111111), expected: u32(18) },
      { input: u32Bits(0b00000000000011111111111111111111), expected: u32(19) },
      { input: u32Bits(0b00000000000111111111111111111111), expected: u32(20) },
      { input: u32Bits(0b00000000001111111111111111111111), expected: u32(21) },
      { input: u32Bits(0b00000000011111111111111111111111), expected: u32(22) },
      { input: u32Bits(0b00000000111111111111111111111111), expected: u32(23) },
      { input: u32Bits(0b00000001111111111111111111111111), expected: u32(24) },
      { input: u32Bits(0b00000011111111111111111111111111), expected: u32(25) },
      { input: u32Bits(0b00000111111111111111111111111111), expected: u32(26) },
      { input: u32Bits(0b00001111111111111111111111111111), expected: u32(27) },
      { input: u32Bits(0b00011111111111111111111111111111), expected: u32(28) },
      { input: u32Bits(0b00111111111111111111111111111111), expected: u32(29) },
      { input: u32Bits(0b01111111111111111111111111111111), expected: u32(30) },
      { input: u32Bits(0b11111111111111111111111111111111), expected: u32(31) },

      // random after leading 1
      { input: u32Bits(0b00000000000000000000000000000110), expected: u32(2) },
      { input: u32Bits(0b00000000000000000000000000001101), expected: u32(3) },
      { input: u32Bits(0b00000000000000000000000000011101), expected: u32(4) },
      { input: u32Bits(0b00000000000000000000000000111001), expected: u32(5) },
      { input: u32Bits(0b00000000000000000000000001101111), expected: u32(6) },
      { input: u32Bits(0b00000000000000000000000011111111), expected: u32(7) },
      { input: u32Bits(0b00000000000000000000000111101111), expected: u32(8) },
      { input: u32Bits(0b00000000000000000000001111111111), expected: u32(9) },
      { input: u32Bits(0b00000000000000000000011111110001), expected: u32(10) },
      { input: u32Bits(0b00000000000000000000111011011101), expected: u32(11) },
      { input: u32Bits(0b00000000000000000001101101111111), expected: u32(12) },
      { input: u32Bits(0b00000000000000000011111111011111), expected: u32(13) },
      { input: u32Bits(0b00000000000000000101111001110101), expected: u32(14) },
      { input: u32Bits(0b00000000000000001101111011110111), expected: u32(15) },
      { input: u32Bits(0b00000000000000011111111111110011), expected: u32(16) },
      { input: u32Bits(0b00000000000000111111111110111111), expected: u32(17) },
      { input: u32Bits(0b00000000000001111111011111111111), expected: u32(18) },
      { input: u32Bits(0b00000000000011111111111111111111), expected: u32(19) },
      { input: u32Bits(0b00000000000111110101011110111111), expected: u32(20) },
      { input: u32Bits(0b00000000001111101111111111110111), expected: u32(21) },
      { input: u32Bits(0b00000000011111111111010000101111), expected: u32(22) },
      { input: u32Bits(0b00000000111111111111001111111011), expected: u32(23) },
      { input: u32Bits(0b00000001111111011111101111111111), expected: u32(24) },
      { input: u32Bits(0b00000011101011111011110111111011), expected: u32(25) },
      { input: u32Bits(0b00000111111110111111111111111111), expected: u32(26) },
      { input: u32Bits(0b00001111000000011011011110111111), expected: u32(27) },
      { input: u32Bits(0b00011110101111011111111111111111), expected: u32(28) },
      { input: u32Bits(0b00110110111111100111111110111101), expected: u32(29) },
      { input: u32Bits(0b01010111111101111111011111011111), expected: u32(30) },
      { input: u32Bits(0b11100010011110101101101110101111), expected: u32(31) },
    ]);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`i32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cfg = t.params;
    await run(t, builtin('firstLeadingBit'), [TypeI32], TypeI32, cfg, [
      // Zero
      { input: i32Bits(0b00000000000000000000000000000000), expected: i32(-1) },

      // One
      { input: i32Bits(0b00000000000000000000000000000001), expected: i32(0) },

      // Positive: 0's after leading 1
      { input: i32Bits(0b00000000000000000000000000000010), expected: i32(1) },
      { input: i32Bits(0b00000000000000000000000000000100), expected: i32(2) },
      { input: i32Bits(0b00000000000000000000000000001000), expected: i32(3) },
      { input: i32Bits(0b00000000000000000000000000010000), expected: i32(4) },
      { input: i32Bits(0b00000000000000000000000000100000), expected: i32(5) },
      { input: i32Bits(0b00000000000000000000000001000000), expected: i32(6) },
      { input: i32Bits(0b00000000000000000000000010000000), expected: i32(7) },
      { input: i32Bits(0b00000000000000000000000100000000), expected: i32(8) },
      { input: i32Bits(0b00000000000000000000001000000000), expected: i32(9) },
      { input: i32Bits(0b00000000000000000000010000000000), expected: i32(10) },
      { input: i32Bits(0b00000000000000000000100000000000), expected: i32(11) },
      { input: i32Bits(0b00000000000000000001000000000000), expected: i32(12) },
      { input: i32Bits(0b00000000000000000010000000000000), expected: i32(13) },
      { input: i32Bits(0b00000000000000000100000000000000), expected: i32(14) },
      { input: i32Bits(0b00000000000000001000000000000000), expected: i32(15) },
      { input: i32Bits(0b00000000000000010000000000000000), expected: i32(16) },
      { input: i32Bits(0b00000000000000100000000000000000), expected: i32(17) },
      { input: i32Bits(0b00000000000001000000000000000000), expected: i32(18) },
      { input: i32Bits(0b00000000000010000000000000000000), expected: i32(19) },
      { input: i32Bits(0b00000000000100000000000000000000), expected: i32(20) },
      { input: i32Bits(0b00000000001000000000000000000000), expected: i32(21) },
      { input: i32Bits(0b00000000010000000000000000000000), expected: i32(22) },
      { input: i32Bits(0b00000000100000000000000000000000), expected: i32(23) },
      { input: i32Bits(0b00000001000000000000000000000000), expected: i32(24) },
      { input: i32Bits(0b00000010000000000000000000000000), expected: i32(25) },
      { input: i32Bits(0b00000100000000000000000000000000), expected: i32(26) },
      { input: i32Bits(0b00001000000000000000000000000000), expected: i32(27) },
      { input: i32Bits(0b00010000000000000000000000000000), expected: i32(28) },
      { input: i32Bits(0b00100000000000000000000000000000), expected: i32(29) },
      { input: i32Bits(0b01000000000000000000000000000000), expected: i32(30) },

      // Negative: 0's after leading 0
      { input: i32Bits(0b11111111111111111111111111111110), expected: i32(0) },
      { input: i32Bits(0b11111111111111111111111111111100), expected: i32(1) },
      { input: i32Bits(0b11111111111111111111111111111000), expected: i32(2) },
      { input: i32Bits(0b11111111111111111111111111110000), expected: i32(3) },
      { input: i32Bits(0b11111111111111111111111111100000), expected: i32(4) },
      { input: i32Bits(0b11111111111111111111111111000000), expected: i32(5) },
      { input: i32Bits(0b11111111111111111111111110000000), expected: i32(6) },
      { input: i32Bits(0b11111111111111111111111100000000), expected: i32(7) },
      { input: i32Bits(0b11111111111111111111111000000000), expected: i32(8) },
      { input: i32Bits(0b11111111111111111111110000000000), expected: i32(9) },
      { input: i32Bits(0b11111111111111111111100000000000), expected: i32(10) },
      { input: i32Bits(0b11111111111111111111000000000000), expected: i32(11) },
      { input: i32Bits(0b11111111111111111110000000000000), expected: i32(12) },
      { input: i32Bits(0b11111111111111111100000000000000), expected: i32(13) },
      { input: i32Bits(0b11111111111111111000000000000000), expected: i32(14) },
      { input: i32Bits(0b11111111111111110000000000000000), expected: i32(15) },
      { input: i32Bits(0b11111111111111100000000000000000), expected: i32(16) },
      { input: i32Bits(0b11111111111111000000000000000000), expected: i32(17) },
      { input: i32Bits(0b11111111111110000000000000000000), expected: i32(18) },
      { input: i32Bits(0b11111111111100000000000000000000), expected: i32(19) },
      { input: i32Bits(0b11111111111000000000000000000000), expected: i32(20) },
      { input: i32Bits(0b11111111110000000000000000000000), expected: i32(21) },
      { input: i32Bits(0b11111111100000000000000000000000), expected: i32(22) },
      { input: i32Bits(0b11111111000000000000000000000000), expected: i32(23) },
      { input: i32Bits(0b11111110000000000000000000000000), expected: i32(24) },
      { input: i32Bits(0b11111100000000000000000000000000), expected: i32(25) },
      { input: i32Bits(0b11111000000000000000000000000000), expected: i32(26) },
      { input: i32Bits(0b11110000000000000000000000000000), expected: i32(27) },
      { input: i32Bits(0b11100000000000000000000000000000), expected: i32(28) },
      { input: i32Bits(0b11000000000000000000000000000000), expected: i32(29) },
      { input: i32Bits(0b10000000000000000000000000000000), expected: i32(30) },

      // Positive: 1's after leading 1
      { input: i32Bits(0b00000000000000000000000000000011), expected: i32(1) },
      { input: i32Bits(0b00000000000000000000000000000111), expected: i32(2) },
      { input: i32Bits(0b00000000000000000000000000001111), expected: i32(3) },
      { input: i32Bits(0b00000000000000000000000000011111), expected: i32(4) },
      { input: i32Bits(0b00000000000000000000000000111111), expected: i32(5) },
      { input: i32Bits(0b00000000000000000000000001111111), expected: i32(6) },
      { input: i32Bits(0b00000000000000000000000011111111), expected: i32(7) },
      { input: i32Bits(0b00000000000000000000000111111111), expected: i32(8) },
      { input: i32Bits(0b00000000000000000000001111111111), expected: i32(9) },
      { input: i32Bits(0b00000000000000000000011111111111), expected: i32(10) },
      { input: i32Bits(0b00000000000000000000111111111111), expected: i32(11) },
      { input: i32Bits(0b00000000000000000001111111111111), expected: i32(12) },
      { input: i32Bits(0b00000000000000000011111111111111), expected: i32(13) },
      { input: i32Bits(0b00000000000000000111111111111111), expected: i32(14) },
      { input: i32Bits(0b00000000000000001111111111111111), expected: i32(15) },
      { input: i32Bits(0b00000000000000011111111111111111), expected: i32(16) },
      { input: i32Bits(0b00000000000000111111111111111111), expected: i32(17) },
      { input: i32Bits(0b00000000000001111111111111111111), expected: i32(18) },
      { input: i32Bits(0b00000000000011111111111111111111), expected: i32(19) },
      { input: i32Bits(0b00000000000111111111111111111111), expected: i32(20) },
      { input: i32Bits(0b00000000001111111111111111111111), expected: i32(21) },
      { input: i32Bits(0b00000000011111111111111111111111), expected: i32(22) },
      { input: i32Bits(0b00000000111111111111111111111111), expected: i32(23) },
      { input: i32Bits(0b00000001111111111111111111111111), expected: i32(24) },
      { input: i32Bits(0b00000011111111111111111111111111), expected: i32(25) },
      { input: i32Bits(0b00000111111111111111111111111111), expected: i32(26) },
      { input: i32Bits(0b00001111111111111111111111111111), expected: i32(27) },
      { input: i32Bits(0b00011111111111111111111111111111), expected: i32(28) },
      { input: i32Bits(0b00111111111111111111111111111111), expected: i32(29) },
      { input: i32Bits(0b01111111111111111111111111111111), expected: i32(30) },

      // Negative: 1's after leading 0
      { input: i32Bits(0b11111111111111111111111111111101), expected: i32(1) },
      { input: i32Bits(0b11111111111111111111111111111011), expected: i32(2) },
      { input: i32Bits(0b11111111111111111111111111110111), expected: i32(3) },
      { input: i32Bits(0b11111111111111111111111111101111), expected: i32(4) },
      { input: i32Bits(0b11111111111111111111111111011111), expected: i32(5) },
      { input: i32Bits(0b11111111111111111111111110111111), expected: i32(6) },
      { input: i32Bits(0b11111111111111111111111101111111), expected: i32(7) },
      { input: i32Bits(0b11111111111111111111111011111111), expected: i32(8) },
      { input: i32Bits(0b11111111111111111111110111111111), expected: i32(9) },
      { input: i32Bits(0b11111111111111111111101111111111), expected: i32(10) },
      { input: i32Bits(0b11111111111111111111011111111111), expected: i32(11) },
      { input: i32Bits(0b11111111111111111110111111111111), expected: i32(12) },
      { input: i32Bits(0b11111111111111111101111111111111), expected: i32(13) },
      { input: i32Bits(0b11111111111111111011111111111111), expected: i32(14) },
      { input: i32Bits(0b11111111111111110111111111111111), expected: i32(15) },
      { input: i32Bits(0b11111111111111101111111111111111), expected: i32(16) },
      { input: i32Bits(0b11111111111111011111111111111111), expected: i32(17) },
      { input: i32Bits(0b11111111111110111111111111111111), expected: i32(18) },
      { input: i32Bits(0b11111111111101111111111111111111), expected: i32(19) },
      { input: i32Bits(0b11111111111011111111111111111111), expected: i32(20) },
      { input: i32Bits(0b11111111110111111111111111111111), expected: i32(21) },
      { input: i32Bits(0b11111111101111111111111111111111), expected: i32(22) },
      { input: i32Bits(0b11111111011111111111111111111111), expected: i32(23) },
      { input: i32Bits(0b11111110111111111111111111111111), expected: i32(24) },
      { input: i32Bits(0b11111101111111111111111111111111), expected: i32(25) },
      { input: i32Bits(0b11111011111111111111111111111111), expected: i32(26) },
      { input: i32Bits(0b11110111111111111111111111111111), expected: i32(27) },
      { input: i32Bits(0b11101111111111111111111111111111), expected: i32(28) },
      { input: i32Bits(0b11011111111111111111111111111111), expected: i32(29) },
      { input: i32Bits(0b10111111111111111111111111111111), expected: i32(30) },

      // Positive: random after leading 1
      { input: i32Bits(0b00000000000000000000000000000110), expected: i32(2) },
      { input: i32Bits(0b00000000000000000000000000001101), expected: i32(3) },
      { input: i32Bits(0b00000000000000000000000000011101), expected: i32(4) },
      { input: i32Bits(0b00000000000000000000000000111001), expected: i32(5) },
      { input: i32Bits(0b00000000000000000000000001101111), expected: i32(6) },
      { input: i32Bits(0b00000000000000000000000011111111), expected: i32(7) },
      { input: i32Bits(0b00000000000000000000000111101111), expected: i32(8) },
      { input: i32Bits(0b00000000000000000000001111111111), expected: i32(9) },
      { input: i32Bits(0b00000000000000000000011111110001), expected: i32(10) },
      { input: i32Bits(0b00000000000000000000111011011101), expected: i32(11) },
      { input: i32Bits(0b00000000000000000001101101111111), expected: i32(12) },
      { input: i32Bits(0b00000000000000000011111111011111), expected: i32(13) },
      { input: i32Bits(0b00000000000000000101111001110101), expected: i32(14) },
      { input: i32Bits(0b00000000000000001101111011110111), expected: i32(15) },
      { input: i32Bits(0b00000000000000011111111111110011), expected: i32(16) },
      { input: i32Bits(0b00000000000000111111111110111111), expected: i32(17) },
      { input: i32Bits(0b00000000000001111111011111111111), expected: i32(18) },
      { input: i32Bits(0b00000000000011111111111111111111), expected: i32(19) },
      { input: i32Bits(0b00000000000111110101011110111111), expected: i32(20) },
      { input: i32Bits(0b00000000001111101111111111110111), expected: i32(21) },
      { input: i32Bits(0b00000000011111111111010000101111), expected: i32(22) },
      { input: i32Bits(0b00000000111111111111001111111011), expected: i32(23) },
      { input: i32Bits(0b00000001111111011111101111111111), expected: i32(24) },
      { input: i32Bits(0b00000011101011111011110111111011), expected: i32(25) },
      { input: i32Bits(0b00000111111110111111111111111111), expected: i32(26) },
      { input: i32Bits(0b00001111000000011011011110111111), expected: i32(27) },
      { input: i32Bits(0b00011110101111011111111111111111), expected: i32(28) },
      { input: i32Bits(0b00110110111111100111111110111101), expected: i32(29) },
      { input: i32Bits(0b01010111111101111111011111011111), expected: i32(30) },

      // Negative: random after leading 0
      { input: i32Bits(0b11111111111111111111111111111010), expected: i32(2) },
      { input: i32Bits(0b11111111111111111111111111110110), expected: i32(3) },
      { input: i32Bits(0b11111111111111111111111111101101), expected: i32(4) },
      { input: i32Bits(0b11111111111111111111111111011101), expected: i32(5) },
      { input: i32Bits(0b11111111111111111111111110111001), expected: i32(6) },
      { input: i32Bits(0b11111111111111111111111101101111), expected: i32(7) },
      { input: i32Bits(0b11111111111111111111111011111111), expected: i32(8) },
      { input: i32Bits(0b11111111111111111111110111101111), expected: i32(9) },
      { input: i32Bits(0b11111111111111111111101111111111), expected: i32(10) },
      { input: i32Bits(0b11111111111111111111011111110001), expected: i32(11) },
      { input: i32Bits(0b11111111111111111110111011011101), expected: i32(12) },
      { input: i32Bits(0b11111111111111111101101101111111), expected: i32(13) },
      { input: i32Bits(0b11111111111111111011111111011111), expected: i32(14) },
      { input: i32Bits(0b11111111111111110101111001110101), expected: i32(15) },
      { input: i32Bits(0b11111111111111101101111011110111), expected: i32(16) },
      { input: i32Bits(0b11111111111111011111111111110011), expected: i32(17) },
      { input: i32Bits(0b11111111111110111111111110111111), expected: i32(18) },
      { input: i32Bits(0b11111111111101111111011111111111), expected: i32(19) },
      { input: i32Bits(0b11111111111011111111111111111111), expected: i32(20) },
      { input: i32Bits(0b11111111110111110101011110111111), expected: i32(21) },
      { input: i32Bits(0b11111111101111101111111111110111), expected: i32(22) },
      { input: i32Bits(0b11111111011111111111010000101111), expected: i32(23) },
      { input: i32Bits(0b11111110111111111111001111111011), expected: i32(24) },
      { input: i32Bits(0b11111101111111011111101111111111), expected: i32(25) },
      { input: i32Bits(0b11111011101011111011110111111011), expected: i32(26) },
      { input: i32Bits(0b11110111111110111111111111111111), expected: i32(27) },
      { input: i32Bits(0b11101111000000011011011110111111), expected: i32(28) },
      { input: i32Bits(0b11011110101111011111111111111111), expected: i32(29) },
      { input: i32Bits(0b10110110111111100111111110111101), expected: i32(30) },
    ]);
  });
