// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the cases which the value of index turn into 1.
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

const input = "a c";
const granularities = [undefined, "grapheme", "word"];
const index_to_one = [
    1,
    1.49,
    14.9E-1,
    14.9e-1,
    "1.49",
    "14.9E-1",
    "14.9e-1",
    true,
    { toString(){ return "1"; } },
    { valueOf(){ return 1; } },
    { [Symbol.toPrimitive](){ return 1; } },
];

// Except granularity: "sentence", check the result.segment is " ".
granularities.forEach(
    function(granularity) {
      const segmenter = new Intl.Segmenter(undefined, {granularity});
      const segment = segmenter.segment(input);
      index_to_one.forEach(function(index) {
        const result = segment.containing(index);
        const msg = "granularity: " + granularity + " index: " + index;
        assert.sameValue(1, result.index, msg + " index");
        assert.sameValue(" ", result.segment, msg + " segment");
        assert.sameValue(input, result.input, msg + " input");
      });
    });

// For granularity: "sentence", result.segment is input
const segmenter = new Intl.Segmenter(undefined, {granularity: "sentence"});
const segment = segmenter.segment(input);
index_to_one.forEach(function(index) {
  const result = segment.containing(index);
  const msg = "granularity: sentence index: " + index;
  assert.sameValue(0, result.index, msg + " index");
  assert.sameValue(input, result.segment, msg + " segment");
  assert.sameValue(input, result.input, msg + " input");
});
