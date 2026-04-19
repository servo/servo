// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.substr
description: >
  Test String.prototype.substr with number inputs for start and length.
---*/

function ToIntegerOrInfinity(arg) {
  assert.sameValue(typeof arg, "number");

  return Number.isNaN(arg) ? 0 : Math.trunc(arg);
}

// Basic reference implementation. Expects all inputs have the correct type.
function StringSubstr(string, start, length) {
  // Steps 1-2.
  assert.sameValue(typeof string, "string");

  // Step 3.
  let size = string.length;

  // Step 4.
  let intStart = ToIntegerOrInfinity(start);

  // Steps 5-7.
  if (intStart === -Infinity) {
    intStart = 0;
  } else if (intStart < 0) {
    intStart = Math.max(size + intStart, 0);
  } else {
    intStart = Math.min(intStart, size)
  }

  // |0 <= intStart <= size| now holds.
  assert(0 <= intStart && intStart <= size);

  // Step 8.
  let intLength = length === undefined ? size : ToIntegerOrInfinity(length);

  // Step 9.
  intLength = Math.min(Math.max(intLength, 0), size);

  // |0 <= intLength <= size| now holds.
  assert(0 <= intLength && intLength <= size);

  // Step 10.
  let intEnd = Math.min(intStart + intLength, size);

  // |intStart <= intEnd <= size| now holds.
  assert(intStart <= intEnd && intEnd <= size);

  // Step 11.
  //
  // Call `substring` and check the result is correct.
  let result = string.substring(intStart, intEnd);

  assert.sameValue(result.length, intEnd - intStart);

  for (let i = 0; i < result.length; ++i) {
    assert.sameValue(result[i], string[intStart + i]);
  }

  return result;
}

const positiveIntegers = [
  0, 1, 2, 3, 4, 5, 10, 100,
];

const integers = [
  ...positiveIntegers,
  ...positiveIntegers.map(v => -v),
];

const numbers = [
  ...integers,
  ...integers.map(v => v + 0.5),
  -Infinity, Infinity, NaN,
];

for (let string of ["", "a", "ab", "abc"]) {
  for (let start of numbers) {
    for (let length of [...numbers, undefined]) {
      let actual = string.substr(start, length);
      let expected = StringSubstr(string, start, length);

      assert.sameValue(actual, expected, `"${string}".substr(${start}, ${length})`);
    }
  }
}
