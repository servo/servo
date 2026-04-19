// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the cases which the value of index turn into 0.
info: |
    %Segments.prototype%.containing ( index )

    6. Let n be ? ToInteger(index).
    7. If n < 0 or n ≥ len, return undefined.
    8. Let startIndex be ! FindBoundary(segmenter, string, n, before).

    ToInteger ( argument )
    1. Let number be ? ToNumber(argument).
    2. If number is NaN, +0, or -0, return +0.
    4. Let integer be the Number value that is the same sign as number and whose magnitude is floor(abs(number)).
    5. If integer is -0, return +0.
    6. Return integer.

    ToNumber ( argument )
    Undefined | Return NaN.
    Null      | Return +0.
    Boolean   | If argument is true, return 1. If argument is false, return +0.

features: [Intl.Segmenter]
---*/

const input = "a b c";
const granularities = [undefined, "grapheme", "word", "sentence"];
const index_to_zeros = [
    0,
    -0,
    NaN,
    0.49,
    -0.49,
    null,
    undefined,
    false,
    "\ud800\udc00", // surrogate
    "\ud800", // only leading surrogate
    "\udc00", // only trailing surrogate
    "a",
    "g",
    "\u00DD",
    "0",
    "+0",
    "-0",
    "0.49",
    "+0.49",
    "-0.49",
    "4.9e-1",
    "-4.9e-1",
    "4.9E-1",
    "-4.9E-1",
    { toString(){ return "-0.1"; } },
    { valueOf(){ return 0.1; } },
    { [Symbol.toPrimitive](){ return -0.1; } },
];

granularities.forEach(
    function(granularity) {
      const segmenter = new Intl.Segmenter(undefined, {granularity});
      const segment = segmenter.segment(input);
      index_to_zeros.forEach(function(index) {
        const result = segment.containing(index);
        assert.sameValue(0, result.index);
        assert.sameValue(input, result.input);
      });
    });
