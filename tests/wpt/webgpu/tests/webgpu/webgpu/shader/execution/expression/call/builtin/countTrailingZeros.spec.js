/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'countTrailingZeros' builtin function

S is i32 or u32
T is S or vecN<S>
@const fn countTrailingZeros(e: T ) -> T
The number of consecutive 0 bits starting from the least significant bit of e,
when T is a scalar type.
Component-wise when T is a vector.
Also known as "ctz" in some languages.
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
    await run(t, builtin('countTrailingZeros'), [TypeU32], TypeU32, cfg, [
      // Zero
      { input: u32Bits(0b00000000000000000000000000000000), expected: u32(32) },

      // High bit
      { input: u32Bits(0b10000000000000000000000000000000), expected: u32(31) },

      // 0's before trailing 1
      { input: u32Bits(0b00000000000000000000000000000001), expected: u32(0) },
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

      // 1's before trailing 1
      { input: u32Bits(0b11111111111111111111111111111111), expected: u32(0) },
      { input: u32Bits(0b11111111111111111111111111111110), expected: u32(1) },
      { input: u32Bits(0b11111111111111111111111111111100), expected: u32(2) },
      { input: u32Bits(0b11111111111111111111111111111000), expected: u32(3) },
      { input: u32Bits(0b11111111111111111111111111110000), expected: u32(4) },
      { input: u32Bits(0b11111111111111111111111111100000), expected: u32(5) },
      { input: u32Bits(0b11111111111111111111111111000000), expected: u32(6) },
      { input: u32Bits(0b11111111111111111111111110000000), expected: u32(7) },
      { input: u32Bits(0b11111111111111111111111100000000), expected: u32(8) },
      { input: u32Bits(0b11111111111111111111111000000000), expected: u32(9) },
      { input: u32Bits(0b11111111111111111111110000000000), expected: u32(10) },
      { input: u32Bits(0b11111111111111111111100000000000), expected: u32(11) },
      { input: u32Bits(0b11111111111111111111000000000000), expected: u32(12) },
      { input: u32Bits(0b11111111111111111110000000000000), expected: u32(13) },
      { input: u32Bits(0b11111111111111111100000000000000), expected: u32(14) },
      { input: u32Bits(0b11111111111111111000000000000000), expected: u32(15) },
      { input: u32Bits(0b11111111111111110000000000000000), expected: u32(16) },
      { input: u32Bits(0b11111111111111100000000000000000), expected: u32(17) },
      { input: u32Bits(0b11111111111111000000000000000000), expected: u32(18) },
      { input: u32Bits(0b11111111111110000000000000000000), expected: u32(19) },
      { input: u32Bits(0b11111111111100000000000000000000), expected: u32(20) },
      { input: u32Bits(0b11111111111000000000000000000000), expected: u32(21) },
      { input: u32Bits(0b11111111110000000000000000000000), expected: u32(22) },
      { input: u32Bits(0b11111111100000000000000000000000), expected: u32(23) },
      { input: u32Bits(0b11111111000000000000000000000000), expected: u32(24) },
      { input: u32Bits(0b11111110000000000000000000000000), expected: u32(25) },
      { input: u32Bits(0b11111100000000000000000000000000), expected: u32(26) },
      { input: u32Bits(0b11111000000000000000000000000000), expected: u32(27) },
      { input: u32Bits(0b11110000000000000000000000000000), expected: u32(28) },
      { input: u32Bits(0b11100000000000000000000000000000), expected: u32(29) },
      { input: u32Bits(0b11000000000000000000000000000000), expected: u32(30) },

      // random before trailing 1
      { input: u32Bits(0b11110000001111111101111010001111), expected: u32(0) },
      { input: u32Bits(0b11011110111111100101110011110010), expected: u32(1) },
      { input: u32Bits(0b11110111011011111111010000111100), expected: u32(2) },
      { input: u32Bits(0b11010011011101111111010011101000), expected: u32(3) },
      { input: u32Bits(0b11010111110111110001111110110000), expected: u32(4) },
      { input: u32Bits(0b11111101111101111110101111100000), expected: u32(5) },
      { input: u32Bits(0b11111001111011111001111011000000), expected: u32(6) },
      { input: u32Bits(0b11001110110111110111111010000000), expected: u32(7) },
      { input: u32Bits(0b11101111011111101110101100000000), expected: u32(8) },
      { input: u32Bits(0b11111101111011111111111000000000), expected: u32(9) },
      { input: u32Bits(0b10011111011101110110110000000000), expected: u32(10) },
      { input: u32Bits(0b11111111101101111011100000000000), expected: u32(11) },
      { input: u32Bits(0b11111011010110111011000000000000), expected: u32(12) },
      { input: u32Bits(0b00111101010000111010000000000000), expected: u32(13) },
      { input: u32Bits(0b11111011110001101100000000000000), expected: u32(14) },
      { input: u32Bits(0b10111111010111111000000000000000), expected: u32(15) },
      { input: u32Bits(0b11011101111010110000000000000000), expected: u32(16) },
      { input: u32Bits(0b01110100110110100000000000000000), expected: u32(17) },
      { input: u32Bits(0b11100111001011000000000000000000), expected: u32(18) },
      { input: u32Bits(0b11111001110110000000000000000000), expected: u32(19) },
      { input: u32Bits(0b00110100100100000000000000000000), expected: u32(20) },
      { input: u32Bits(0b11111010011000000000000000000000), expected: u32(21) },
      { input: u32Bits(0b00000010110000000000000000000000), expected: u32(22) },
      { input: u32Bits(0b11100111100000000000000000000000), expected: u32(23) },
      { input: u32Bits(0b00101101000000000000000000000000), expected: u32(24) },
      { input: u32Bits(0b11011010000000000000000000000000), expected: u32(25) },
      { input: u32Bits(0b11010100000000000000000000000000), expected: u32(26) },
      { input: u32Bits(0b10111000000000000000000000000000), expected: u32(27) },
      { input: u32Bits(0b01110000000000000000000000000000), expected: u32(28) },
      { input: u32Bits(0b10100000000000000000000000000000), expected: u32(29) },
    ]);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`i32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cfg = t.params;
    await run(t, builtin('countTrailingZeros'), [TypeI32], TypeI32, cfg, [
      // Zero
      { input: i32Bits(0b00000000000000000000000000000000), expected: i32(32) },

      // High bit
      { input: i32Bits(0b10000000000000000000000000000000), expected: i32(31) },

      // 0's before trailing 1
      { input: i32Bits(0b00000000000000000000000000000001), expected: i32(0) },
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

      // 1's before trailing 1
      { input: i32Bits(0b11111111111111111111111111111111), expected: i32(0) },
      { input: i32Bits(0b11111111111111111111111111111110), expected: i32(1) },
      { input: i32Bits(0b11111111111111111111111111111100), expected: i32(2) },
      { input: i32Bits(0b11111111111111111111111111111000), expected: i32(3) },
      { input: i32Bits(0b11111111111111111111111111110000), expected: i32(4) },
      { input: i32Bits(0b11111111111111111111111111100000), expected: i32(5) },
      { input: i32Bits(0b11111111111111111111111111000000), expected: i32(6) },
      { input: i32Bits(0b11111111111111111111111110000000), expected: i32(7) },
      { input: i32Bits(0b11111111111111111111111100000000), expected: i32(8) },
      { input: i32Bits(0b11111111111111111111111000000000), expected: i32(9) },
      { input: i32Bits(0b11111111111111111111110000000000), expected: i32(10) },
      { input: i32Bits(0b11111111111111111111100000000000), expected: i32(11) },
      { input: i32Bits(0b11111111111111111111000000000000), expected: i32(12) },
      { input: i32Bits(0b11111111111111111110000000000000), expected: i32(13) },
      { input: i32Bits(0b11111111111111111100000000000000), expected: i32(14) },
      { input: i32Bits(0b11111111111111111000000000000000), expected: i32(15) },
      { input: i32Bits(0b11111111111111110000000000000000), expected: i32(16) },
      { input: i32Bits(0b11111111111111100000000000000000), expected: i32(17) },
      { input: i32Bits(0b11111111111111000000000000000000), expected: i32(18) },
      { input: i32Bits(0b11111111111110000000000000000000), expected: i32(19) },
      { input: i32Bits(0b11111111111100000000000000000000), expected: i32(20) },
      { input: i32Bits(0b11111111111000000000000000000000), expected: i32(21) },
      { input: i32Bits(0b11111111110000000000000000000000), expected: i32(22) },
      { input: i32Bits(0b11111111100000000000000000000000), expected: i32(23) },
      { input: i32Bits(0b11111111000000000000000000000000), expected: i32(24) },
      { input: i32Bits(0b11111110000000000000000000000000), expected: i32(25) },
      { input: i32Bits(0b11111100000000000000000000000000), expected: i32(26) },
      { input: i32Bits(0b11111000000000000000000000000000), expected: i32(27) },
      { input: i32Bits(0b11110000000000000000000000000000), expected: i32(28) },
      { input: i32Bits(0b11100000000000000000000000000000), expected: i32(29) },
      { input: i32Bits(0b11000000000000000000000000000000), expected: i32(30) },

      // random before trailing 1
      { input: i32Bits(0b11110000001111111101111010001111), expected: i32(0) },
      { input: i32Bits(0b11011110111111100101110011110010), expected: i32(1) },
      { input: i32Bits(0b11110111011011111111010000111100), expected: i32(2) },
      { input: i32Bits(0b11010011011101111111010011101000), expected: i32(3) },
      { input: i32Bits(0b11010111110111110001111110110000), expected: i32(4) },
      { input: i32Bits(0b11111101111101111110101111100000), expected: i32(5) },
      { input: i32Bits(0b11111001111011111001111011000000), expected: i32(6) },
      { input: i32Bits(0b11001110110111110111111010000000), expected: i32(7) },
      { input: i32Bits(0b11101111011111101110101100000000), expected: i32(8) },
      { input: i32Bits(0b11111101111011111111111000000000), expected: i32(9) },
      { input: i32Bits(0b10011111011101110110110000000000), expected: i32(10) },
      { input: i32Bits(0b11111111101101111011100000000000), expected: i32(11) },
      { input: i32Bits(0b11111011010110111011000000000000), expected: i32(12) },
      { input: i32Bits(0b00111101010000111010000000000000), expected: i32(13) },
      { input: i32Bits(0b11111011110001101100000000000000), expected: i32(14) },
      { input: i32Bits(0b10111111010111111000000000000000), expected: i32(15) },
      { input: i32Bits(0b11011101111010110000000000000000), expected: i32(16) },
      { input: i32Bits(0b01110100110110100000000000000000), expected: i32(17) },
      { input: i32Bits(0b11100111001011000000000000000000), expected: i32(18) },
      { input: i32Bits(0b11111001110110000000000000000000), expected: i32(19) },
      { input: i32Bits(0b00110100100100000000000000000000), expected: i32(20) },
      { input: i32Bits(0b11111010011000000000000000000000), expected: i32(21) },
      { input: i32Bits(0b00000010110000000000000000000000), expected: i32(22) },
      { input: i32Bits(0b11100111100000000000000000000000), expected: i32(23) },
      { input: i32Bits(0b00101101000000000000000000000000), expected: i32(24) },
      { input: i32Bits(0b11011010000000000000000000000000), expected: i32(25) },
      { input: i32Bits(0b11010100000000000000000000000000), expected: i32(26) },
      { input: i32Bits(0b10111000000000000000000000000000), expected: i32(27) },
      { input: i32Bits(0b01110000000000000000000000000000), expected: i32(28) },
      { input: i32Bits(0b10100000000000000000000000000000), expected: i32(29) },
    ]);
  });
