// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-torawprecision
description: >
  Rounding to significant digits when the rounded value reaches the next order
  of magnitude
info: |
  ToRawPrecision ( x, minPrecision, maxPrecision, unsignedRoundingMode )

  1. Let p be maxPrecision.
  ...
  3. Else,
    a. Let n1 and e1 each be an integer and r1 a mathematical value, with
       r1 = ToRawPrecisionFn(n1, e1, p), such that r1 ≤ x and r1 is maximized.
    b. Let n2 and e2 each be an integer and r2 a mathematical value, with
       r2 = ToRawPrecisionFn(n2, e2, p), such that r2 ≥ x and r2 is minimized.
    c. Let xFinal be ApplyUnsignedRoundingMode(x, r1, r2, unsignedRoundingMode).
    d. If xFinal is r1, then
      i. Let n be n1.
      ii. Let e be e1.
    e. Else,
      i. Let n be n2.
      ii. Let e be e2.
    f. Let m be the String consisting of the digits of the decimal
       representation of n (in order, with no leading zeroes).
  4. If e ≥ (p - 1), then
    a. Set m to the string-concatenation of m and e - p + 1 occurrences of the
       code unit 0x0030 (DIGIT ZERO).
    b. Let int be e + 1.
  5. Else if e ≥ 0, then
    a. Set m to the string-concatenation of the first e + 1 code units of m,
       the code unit 0x002E (FULL STOP), and the remaining p - (e + 1) code
       units of m.
    b. Let int be e + 1.
  6. Else,
    a. Assert: e < 0.
    b. Set m to the string-concatenation of "0.", -(e + 1) occurrences of the
       code unit 0x0030 (DIGIT ZERO), and m.
    c. Let int be 1.
  7. If m contains the code unit 0x002E (FULL STOP) and
     maxPrecision > minPrecision, then
    a. Let cut be maxPrecision - minPrecision.
    b. Repeat, while cut > 0 and the last code unit of m is 0x0030
       (DIGIT ZERO),
      i. Remove the last code unit from m.
      ii. Set cut to cut - 1.
    c. If the last code unit of m is 0x002E (FULL STOP), then
      i. Remove the last code unit from m.
includes: [testIntl.js]
---*/

var locales = [
  new Intl.NumberFormat().resolvedOptions().locale,
  "ar", "de", "th", "ja"
];
var numberingSystems = ["latn", "arab", "thai", "hanidec"];

// minimumSignificantDigits = maximumSignificantDigits, so no trailing zeros
// are trimmed and the digit count of the result is visible directly.
testNumberFormat(locales, numberingSystems,
  { useGrouping: false, minimumSignificantDigits: 3, maximumSignificantDigits: 3 },
  {
    // step 6: e = -3 → e = -2 (still e < 0)
    "0.0009999": "0.00100",
    "-0.0009999": "-0.00100",
    // step 6: e = -2 → e = -1
    "0.09999": "0.100",
    "-0.09999": "-0.100",
    // step 6 (e = -1) → step 5 (e = 0)
    "0.9999": "1.00",
    "-0.9999": "-1.00",
    // step 5: e = 0 → e = 1
    "9.999": "10.0",
    "-9.999": "-10.0",
    // step 5 (e = 1) → step 4 (e = 2 = p - 1)
    "99.99": "100",
    "-99.99": "-100",
    // step 4: e = 2 → e = 3
    "999.9": "1000",
    "-999.9": "-1000",
    // step 4: e = 5 → e = 6
    "999999.9": "1000000",
    "-999999.9": "-1000000",
    // control values that round without changing magnitude
    "9.994": "9.99",
    "99.94": "99.9",
    "0.9994": "0.999",
  });

// Default minimumSignificantDigits of 1: rollover still happens, and the
// zeros that appear after the FULL STOP are trimmed by step 7.
testNumberFormat(locales, numberingSystems,
  { useGrouping: false, maximumSignificantDigits: 3 },
  {
    "0.0009999": "0.001",
    "0.09999": "0.1",
    "0.9999": "1",
    "-0.9999": "-1",
    "9.999": "10",
    "-9.999": "-10",
    "99.99": "100",
    "-99.99": "-100",
    "999.9": "1000",
    "999999.9": "1000000",
  });

// Lower minimum: step 7 trims at most maxPrecision - minPrecision zeros.
testNumberFormat(locales, numberingSystems,
  { useGrouping: false, minimumSignificantDigits: 2, maximumSignificantDigits: 3 },
  {
    "0.9999": "1.0",
    "9.999": "10",
    "99.99": "100",
  });

// Two significant digits
testNumberFormat(locales, numberingSystems,
  { useGrouping: false, minimumSignificantDigits: 2, maximumSignificantDigits: 2 },
  {
    "0.996": "1.0",
    "9.96": "10",
    "99.6": "100",
    "996": "1000",
  });

// Larger precision
testNumberFormat(locales, numberingSystems,
  { useGrouping: false, minimumSignificantDigits: 5, maximumSignificantDigits: 5 },
  {
    "9.99996": "10.000",
    "99999.6": "100000",
    "0.999996": "1.0000",
  });

// The rounded value is always compared against x; a value just below the
// rounding boundary must not roll over.
testNumberFormat(locales, numberingSystems,
  { useGrouping: false, minimumSignificantDigits: 3, maximumSignificantDigits: 3 },
  {
    "9.9949": "9.99",
    "99.949": "99.9",
    "999.49": "999",
  });
