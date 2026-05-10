// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the cases which the value of index which throws.
info: |
    %Segments.prototype%.containing ( index )

    6. Let n be ? ToInteger(index).
    7. If n < 0 or n ≥ len, return undefined.
    8. Let startIndex be ! FindBoundary(segmenter, string, n, before).

    ToInteger ( argument )
    1. Let number be ? ToNumber(argument).

    ToNumber ( argument )
    Symbol | Throw a TypeError exception.
    BigInt | Throw a TypeError exception.

features: [Intl.Segmenter]
---*/

const input = "a b c";
const granularities = [undefined, "grapheme", "word", "sentence"];
const index_throws = [
    // Symbol
    Symbol(),
    // BigInt
    0n,
    -1n,
    1n,
    BigInt(0),
    BigInt(1),
    BigInt(-1),
    BigInt(input.length),
];

granularities.forEach(
    function(granularity) {
      const segmenter = new Intl.Segmenter(undefined, {granularity});
      const segment = segmenter.segment(input);
      index_throws.forEach(function(index) {
        assert.throws(TypeError, () => {segment.containing(index);})
      });
    });
