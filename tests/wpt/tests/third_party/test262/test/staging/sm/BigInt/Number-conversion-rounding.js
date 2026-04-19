// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/licenses/publicdomain/

/**
 * Edge-case behavior at Number.MAX_VALUE and beyond til overflow to Infinity.
 */
function maxMagnitudeTests(isNegative)
{
  var sign = isNegative ? -1 : +1;
  var signBigInt = isNegative ? -1n : 1n;

  const MAX_VALUE = isNegative ? -Number.MAX_VALUE : +Number.MAX_VALUE;

  // 2**971+2**972+...+2**1022+2**1023
  var maxMagnitudeNumber = 0;
  for (let i = 971; i < 1024; i++)
    maxMagnitudeNumber += 2**i;
  maxMagnitudeNumber *= sign;
  assert.sameValue(maxMagnitudeNumber, MAX_VALUE);

  // 2**971+2**972+...+2**1022+2**1023
  var maxMagnitudeNumberAsBigInt = 0n;
  for (let i = 971n; i < 1024n; i++)
    maxMagnitudeNumberAsBigInt += 2n**i;
  maxMagnitudeNumberAsBigInt *= signBigInt;
  var expectedMaxMagnitude = isNegative
                           ? -(2n**1024n) + 2n**971n
                           : 2n**1024n - 2n**971n;
  assert.sameValue(maxMagnitudeNumberAsBigInt, expectedMaxMagnitude);

  // Initial sanity tests.
  assert.sameValue(BigInt(maxMagnitudeNumber), maxMagnitudeNumberAsBigInt);
  assert.sameValue(maxMagnitudeNumber, Number(maxMagnitudeNumberAsBigInt));

  // Test conversion of BigInt values above Number.MAX_VALUE.
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 1n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 2n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 3n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 4n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 5n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 6n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 7n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 8n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 9n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 2n**20n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 2n**400n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 2n**800n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 2n**900n), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 2n**969n), MAX_VALUE);

  // For conversion purposes, rounding for values above Number.MAX_VALUE do
  // rounding with respect to Number.MAX_VALUE and 2**1024 (which is treated as
  // the "even" value -- so if the value to convert lies halfway between those two
  // values, 2**1024 is selected).  But if 2**1024 is the value that *would* have
  // been chosen by this process, Infinity is substituted.
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * (2n**970n - 1n)), MAX_VALUE);
  assert.sameValue(Number(maxMagnitudeNumberAsBigInt + signBigInt * 2n**970n), sign * Infinity);
}
maxMagnitudeTests(false);
maxMagnitudeTests(true);

/**
 * Simple single-Digit on x64, double-Digit on x86 tests.
 */

assert.sameValue(BigInt(Number(2n**53n - 2n)), 2n**53n - 2n);
assert.sameValue(BigInt(Number(2n**53n - 1n)), 2n**53n - 1n);
assert.sameValue(BigInt(Number(2n**53n)), 2n**53n);
assert.sameValue(BigInt(Number(2n**53n + 1n)), 2n**53n);
assert.sameValue(BigInt(Number(2n**53n + 2n)), 2n**53n + 2n);
assert.sameValue(BigInt(Number(2n**53n + 3n)), 2n**53n + 4n);
assert.sameValue(BigInt(Number(2n**53n + 4n)), 2n**53n + 4n);
assert.sameValue(BigInt(Number(2n**53n + 5n)), 2n**53n + 4n);
assert.sameValue(BigInt(Number(2n**53n + 6n)), 2n**53n + 6n);
assert.sameValue(BigInt(Number(2n**53n + 7n)), 2n**53n + 8n);
assert.sameValue(BigInt(Number(2n**53n + 8n)), 2n**53n + 8n);

assert.sameValue(BigInt(Number(2n**54n - 4n)), 2n**54n - 4n);
assert.sameValue(BigInt(Number(2n**54n - 3n)), 2n**54n - 4n);
assert.sameValue(BigInt(Number(2n**54n - 2n)), 2n**54n - 2n);
assert.sameValue(BigInt(Number(2n**54n - 1n)), 2n**54n);
assert.sameValue(BigInt(Number(2n**54n)), 2n**54n);
assert.sameValue(BigInt(Number(2n**54n + 1n)), 2n**54n);
assert.sameValue(BigInt(Number(2n**54n + 2n)), 2n**54n);
assert.sameValue(BigInt(Number(2n**54n + 3n)), 2n**54n + 4n);
assert.sameValue(BigInt(Number(2n**54n + 4n)), 2n**54n + 4n);
assert.sameValue(BigInt(Number(2n**54n + 5n)), 2n**54n + 4n);
assert.sameValue(BigInt(Number(2n**54n + 6n)), 2n**54n + 8n);
assert.sameValue(BigInt(Number(2n**54n + 7n)), 2n**54n + 8n);
assert.sameValue(BigInt(Number(2n**54n + 8n)), 2n**54n + 8n);

assert.sameValue(BigInt(Number(2n**55n - 8n)), 2n**55n - 8n);
assert.sameValue(BigInt(Number(2n**55n - 7n)), 2n**55n - 8n);
assert.sameValue(BigInt(Number(2n**55n - 6n)), 2n**55n - 8n);
assert.sameValue(BigInt(Number(2n**55n - 5n)), 2n**55n - 4n);
assert.sameValue(BigInt(Number(2n**55n - 4n)), 2n**55n - 4n);
assert.sameValue(BigInt(Number(2n**55n - 3n)), 2n**55n - 4n);
assert.sameValue(BigInt(Number(2n**55n - 2n)), 2n**55n);
assert.sameValue(BigInt(Number(2n**55n - 1n)), 2n**55n);
assert.sameValue(BigInt(Number(2n**55n)), 2n**55n);
assert.sameValue(BigInt(Number(2n**55n + 1n)), 2n**55n);
assert.sameValue(BigInt(Number(2n**55n + 2n)), 2n**55n);
assert.sameValue(BigInt(Number(2n**55n + 3n)), 2n**55n);
assert.sameValue(BigInt(Number(2n**55n + 4n)), 2n**55n);
assert.sameValue(BigInt(Number(2n**55n + 5n)), 2n**55n + 8n);
assert.sameValue(BigInt(Number(2n**55n + 6n)), 2n**55n + 8n);
assert.sameValue(BigInt(Number(2n**55n + 7n)), 2n**55n + 8n);
assert.sameValue(BigInt(Number(2n**55n + 8n)), 2n**55n + 8n);
assert.sameValue(BigInt(Number(2n**55n + 9n)), 2n**55n + 8n);
assert.sameValue(BigInt(Number(2n**55n + 10n)), 2n**55n + 8n);
assert.sameValue(BigInt(Number(2n**55n + 11n)), 2n**55n + 8n);
assert.sameValue(BigInt(Number(2n**55n + 12n)), 2n**55n + 16n);
assert.sameValue(BigInt(Number(2n**55n + 13n)), 2n**55n + 16n);
assert.sameValue(BigInt(Number(2n**55n + 14n)), 2n**55n + 16n);
assert.sameValue(BigInt(Number(2n**55n + 15n)), 2n**55n + 16n);
assert.sameValue(BigInt(Number(2n**55n + 16n)), 2n**55n + 16n);


/**
 * Simple double-Digit on x64, triple-Digit on x86 tests.
 */

// The tests below that aren't subtracting bits will have no bits in the
// ultimate significand from the most-significant digit (because of the implicit
// one being excluded).
assert.sameValue(BigInt(Number(2n**64n - 2n**11n)), 2n**64n - 2n**11n);
assert.sameValue(BigInt(Number(2n**64n - 2n**11n + 2n**10n - 1n)), 2n**64n - 2n**11n);
assert.sameValue(BigInt(Number(2n**64n - 2n**11n + 2n**10n)), 2n**64n);
assert.sameValue(BigInt(Number(2n**64n - 2n**10n)), 2n**64n);
assert.sameValue(BigInt(Number(2n**64n)), 2n**64n);
assert.sameValue(BigInt(Number(2n**64n + 1n)), 2n**64n);
assert.sameValue(BigInt(Number(2n**64n + 2n**5n)), 2n**64n);
assert.sameValue(BigInt(Number(2n**64n + 2n**10n)), 2n**64n);
assert.sameValue(BigInt(Number(2n**64n + 2n**11n)), 2n**64n);
assert.sameValue(BigInt(Number(2n**64n + 2n**11n + 1n)), 2n**64n + 2n**12n);
assert.sameValue(BigInt(Number(2n**64n + 2n**12n)), 2n**64n + 2n**12n);
assert.sameValue(BigInt(Number(2n**64n + 2n**12n + 1n)), 2n**64n + 2n**12n);
assert.sameValue(BigInt(Number(2n**64n + 2n**12n + 2n**5n)), 2n**64n + 2n**12n);
assert.sameValue(BigInt(Number(2n**64n + 2n**12n + 2n**10n)), 2n**64n + 2n**12n);
assert.sameValue(BigInt(Number(2n**64n + 2n**12n + 2n**11n - 1n)), 2n**64n + 2n**12n);
assert.sameValue(BigInt(Number(2n**64n + 2n**12n + 2n**11n)), 2n**64n + 2n**13n);
assert.sameValue(BigInt(Number(2n**64n + 2n**12n + 2n**11n + 1n)), 2n**64n + 2n**13n);

// These tests *will* have a bit from the most-significant digit in the ultimate
// significand.
assert.sameValue(BigInt(Number(2n**65n - 2n**12n)), 2n**65n - 2n**12n);
assert.sameValue(BigInt(Number(2n**65n - 2n**12n + 2n**11n - 1n)), 2n**65n - 2n**12n);
assert.sameValue(BigInt(Number(2n**65n - 2n**12n + 2n**11n)), 2n**65n);
assert.sameValue(BigInt(Number(2n**65n - 2n**11n)), 2n**65n);
assert.sameValue(BigInt(Number(2n**65n)), 2n**65n);
assert.sameValue(BigInt(Number(2n**65n + 1n)), 2n**65n);
assert.sameValue(BigInt(Number(2n**65n + 2n**5n)), 2n**65n);
assert.sameValue(BigInt(Number(2n**65n + 2n**11n)), 2n**65n);
assert.sameValue(BigInt(Number(2n**65n + 2n**12n)), 2n**65n);
assert.sameValue(BigInt(Number(2n**65n + 2n**12n + 1n)), 2n**65n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**13n)), 2n**65n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**13n + 1n)), 2n**65n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**13n + 2n**5n)), 2n**65n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**13n + 2n**11n)), 2n**65n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**13n + 2n**12n - 1n)), 2n**65n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**13n + 2n**12n)), 2n**65n + 2n**14n);
assert.sameValue(BigInt(Number(2n**65n + 2n**13n + 2n**12n + 1n)), 2n**65n + 2n**14n);

// ...and in these tests, the contributed bit from the most-significant digit
// is additionally nonzero.
assert.sameValue(BigInt(Number(2n**65n + 2n**64n)), 2n**65n + 2n**64n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 1n)), 2n**65n + 2n**64n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**5n)), 2n**65n + 2n**64n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**11n)), 2n**65n + 2n**64n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**12n)), 2n**65n + 2n**64n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**12n + 1n)), 2n**65n + 2n**64n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**13n)), 2n**65n + 2n**64n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**13n + 1n)), 2n**65n + 2n**64n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**13n + 2n**5n)), 2n**65n + 2n**64n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**13n + 2n**11n)), 2n**65n + 2n**64n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**13n + 2n**12n - 1n)), 2n**65n + 2n**64n + 2n**13n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**13n + 2n**12n)), 2n**65n + 2n**64n + 2n**14n);
assert.sameValue(BigInt(Number(2n**65n + 2n**64n + 2n**13n + 2n**12n + 1n)), 2n**65n + 2n**64n + 2n**14n);

/**
 * Versions of the testing above with all the high-order bits massively bumped
 * upward to test that super-low bits, not just bits in high digits, are
 * properly accounted for in rounding.
 */

// The tests below that aren't subtracting bits will have no bits in the
// ultimate significand from the most-significant digit (because of the implicit
// one being excluded).
assert.sameValue(BigInt(Number(2n**940n - 2n**887n + 1n)), 2n**940n - 2n**887n);
assert.sameValue(BigInt(Number(2n**940n - 2n**887n + 2n**886n - 1n)), 2n**940n - 2n**887n);
assert.sameValue(BigInt(Number(2n**940n - 2n**887n + 2n**886n)), 2n**940n);
assert.sameValue(BigInt(Number(2n**940n - 2n**886n)), 2n**940n);
assert.sameValue(BigInt(Number(2n**940n)), 2n**940n);
assert.sameValue(BigInt(Number(2n**940n + 1n)), 2n**940n);
assert.sameValue(BigInt(Number(2n**940n + 2n**880n)), 2n**940n);
assert.sameValue(BigInt(Number(2n**940n + 2n**885n)), 2n**940n);
assert.sameValue(BigInt(Number(2n**940n + 2n**887n)), 2n**940n);
assert.sameValue(BigInt(Number(2n**940n + 2n**887n + 1n)), 2n**940n + 2n**888n);
assert.sameValue(BigInt(Number(2n**940n + 2n**888n)), 2n**940n + 2n**888n);
assert.sameValue(BigInt(Number(2n**940n + 2n**888n + 1n)), 2n**940n + 2n**888n);
assert.sameValue(BigInt(Number(2n**940n + 2n**888n + 2n**5n)), 2n**940n + 2n**888n);
assert.sameValue(BigInt(Number(2n**940n + 2n**888n + 2n**12n)), 2n**940n + 2n**888n);
assert.sameValue(BigInt(Number(2n**940n + 2n**888n + 2n**887n - 1n)), 2n**940n + 2n**888n);
assert.sameValue(BigInt(Number(2n**940n + 2n**888n + 2n**887n)), 2n**940n + 2n**889n);
assert.sameValue(BigInt(Number(2n**940n + 2n**888n + 2n**887n + 1n)), 2n**940n + 2n**889n);

// These tests *will* have a bit from the most-significant digit in the ultimate
// significand.
assert.sameValue(BigInt(Number(2n**941n - 2n**888n)), 2n**941n - 2n**888n);
assert.sameValue(BigInt(Number(2n**941n - 2n**888n + 2n**887n - 1n)), 2n**941n - 2n**888n);
assert.sameValue(BigInt(Number(2n**941n - 2n**888n + 2n**887n)), 2n**941n);
assert.sameValue(BigInt(Number(2n**941n - 2n**887n)), 2n**941n);
assert.sameValue(BigInt(Number(2n**941n)), 2n**941n);
assert.sameValue(BigInt(Number(2n**941n + 1n)), 2n**941n);
assert.sameValue(BigInt(Number(2n**941n + 2n**881n)), 2n**941n);
assert.sameValue(BigInt(Number(2n**941n + 2n**886n)), 2n**941n);
assert.sameValue(BigInt(Number(2n**941n + 2n**888n)), 2n**941n);
assert.sameValue(BigInt(Number(2n**941n + 2n**888n + 1n)), 2n**941n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**889n)), 2n**941n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**889n + 1n)), 2n**941n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**889n + 2n**5n)), 2n**941n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**889n + 2n**12n)), 2n**941n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**889n + 2n**888n - 1n)), 2n**941n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**889n + 2n**888n)), 2n**941n + 2n**890n);
assert.sameValue(BigInt(Number(2n**941n + 2n**889n + 2n**888n + 1n)), 2n**941n + 2n**890n);

// ...and in these tests, the contributed bit from the most-significant digit
// is additionally nonzero.
assert.sameValue(BigInt(Number(2n**941n + 2n**940n)), 2n**941n + 2n**940n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 1n)), 2n**941n + 2n**940n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**881n)), 2n**941n + 2n**940n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**886n)), 2n**941n + 2n**940n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**888n)), 2n**941n + 2n**940n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**888n + 1n)), 2n**941n + 2n**940n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**889n)), 2n**941n + 2n**940n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**889n + 1n)), 2n**941n + 2n**940n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**889n + 2n**5n)), 2n**941n + 2n**940n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**889n + 2n**12n)), 2n**941n + 2n**940n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**889n + 2n**888n - 1n)), 2n**941n + 2n**940n + 2n**889n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**889n + 2n**888n)), 2n**941n + 2n**940n + 2n**890n);
assert.sameValue(BigInt(Number(2n**941n + 2n**940n + 2n**889n + 2n**888n + 1n)), 2n**941n + 2n**940n + 2n**890n);
