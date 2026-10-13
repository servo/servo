// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-torawprecision
description: >
  Grouping and decimal parts reflect the magnitude of the rounded value when
  rounding to significant digits carries into a new leading digit
info: |
  ToRawPrecision ( x, minPrecision, maxPrecision, unsignedRoundingMode )

  ...
  4. If e ≥ (p - 1), then
    a. Set m to the string-concatenation of m and e - p + 1 occurrences of the
       code unit 0x0030 (DIGIT ZERO).
    b. Let int be e + 1.
  5. Else if e ≥ 0, then
    a. Set m to the string-concatenation of the first e + 1 code units of m,
       the code unit 0x002E (FULL STOP), and the remaining p - (e + 1) code
       units of m.
    b. Let int be e + 1.
  ...
  8. Return the Record { [[FormattedString]]: m, [[RoundedNumber]]: xFinal,
     [[IntegerDigitsCount]]: int, [[RoundingMagnitude]]: e–p+1 }.
locale: [en-US]
includes: [deepEqual.js]
---*/

const nf = new Intl.NumberFormat("en-US", { maximumSignificantDigits: 3 });

// 999.9 rounds to 1000: four integer digits, so a group separator appears.
assert.deepEqual(nf.formatToParts(999.9), [
  { type: "integer", value: "1" },
  { type: "group", value: "," },
  { type: "integer", value: "000" },
], "999.9 with maximumSignificantDigits 3 is '1,000'");

assert.deepEqual(nf.formatToParts(-999.9), [
  { type: "minusSign", value: "-" },
  { type: "integer", value: "1" },
  { type: "group", value: "," },
  { type: "integer", value: "000" },
], "-999.9 with maximumSignificantDigits 3 is '-1,000'");

// 999999.9 rounds to 1000000: three groups.
assert.deepEqual(nf.formatToParts(999999.9), [
  { type: "integer", value: "1" },
  { type: "group", value: "," },
  { type: "integer", value: "000" },
  { type: "group", value: "," },
  { type: "integer", value: "000" },
], "999999.9 with maximumSignificantDigits 3 is '1,000,000'");

// 9.999 rounds to 10: two integer digits and, with the default minimum of one
// significant digit, no fraction.
assert.deepEqual(nf.formatToParts(9.999), [
  { type: "integer", value: "10" },
], "9.999 with maximumSignificantDigits 3 is '10'");

// 0.9999 rounds to 1: the fraction disappears entirely.
assert.deepEqual(nf.formatToParts(0.9999), [
  { type: "integer", value: "1" },
], "0.9999 with maximumSignificantDigits 3 is '1'");

const nfMin = new Intl.NumberFormat("en-US", {
  minimumSignificantDigits: 3,
  maximumSignificantDigits: 3,
});

// With a minimum of three significant digits the trailing zero is kept as a
// fraction digit after the carry.
assert.deepEqual(nfMin.formatToParts(9.999), [
  { type: "integer", value: "10" },
  { type: "decimal", value: "." },
  { type: "fraction", value: "0" },
], "9.999 with minimumSignificantDigits 3 is '10.0'");

assert.deepEqual(nfMin.formatToParts(0.9999), [
  { type: "integer", value: "1" },
  { type: "decimal", value: "." },
  { type: "fraction", value: "00" },
], "0.9999 with minimumSignificantDigits 3 is '1.00'");

assert.deepEqual(nfMin.formatToParts(99.99), [
  { type: "integer", value: "100" },
], "99.99 with minimumSignificantDigits 3 is '100'");

const nfOne = new Intl.NumberFormat("en-US", { maximumSignificantDigits: 1 });

// A single significant digit: 9.6 rounds to 10, 96 to 100, 996 to 1,000.
assert.deepEqual(nfOne.formatToParts(0.96), [
  { type: "integer", value: "1" },
], "0.96 with maximumSignificantDigits 1 is '1'");

assert.deepEqual(nfOne.formatToParts(9.6), [
  { type: "integer", value: "10" },
], "9.6 with maximumSignificantDigits 1 is '10'");

assert.deepEqual(nfOne.formatToParts(96), [
  { type: "integer", value: "100" },
], "96 with maximumSignificantDigits 1 is '100'");

assert.deepEqual(nfOne.formatToParts(996), [
  { type: "integer", value: "1" },
  { type: "group", value: "," },
  { type: "integer", value: "000" },
], "996 with maximumSignificantDigits 1 is '1,000'");

assert.deepEqual(nfOne.formatToParts(0.096), [
  { type: "integer", value: "0" },
  { type: "decimal", value: "." },
  { type: "fraction", value: "1" },
], "0.096 with maximumSignificantDigits 1 is '0.1'");
