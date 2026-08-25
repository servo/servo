// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the cases which the value of index turn into out of bound.
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
    String | See grammar and conversion algorithm below.

features: [Intl.Segmenter]
---*/

const input = "a b c";
const granularities = [undefined, "grapheme", "word", "sentence"];
const index_to_out_of_bound = [
    input.length,
    input.length + 0.1,
    -1,
    -2,
    "-1",
    "-2",
    "-1.1",
    Infinity,
    -Infinity,
    "Infinity",
    "-Infinity",
    { toString(){ return "-1"; } },
    { valueOf(){ return input.length; } },
    { [Symbol.toPrimitive](){ return -1; } },
];

granularities.forEach(
    function(granularity) {
      const segmenter = new Intl.Segmenter(undefined, {granularity});
      const segment = segmenter.segment(input);
      index_to_out_of_bound.forEach(function(index) {
        const result = segment.containing(index);
        assert.sameValue(undefined, result);
      });
    });
