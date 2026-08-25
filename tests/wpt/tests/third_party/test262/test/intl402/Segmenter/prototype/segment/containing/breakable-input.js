// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the cases which the input is breakable.
info: |
    %Segments.prototype%.containing ( index )

    8. Let startIndex be ! FindBoundary(segmenter, string, n, before).
    9. Let endIndex be ! FindBoundary(segmenter, string, n, after).

features: [Intl.Segmenter]
---*/

// The inputs are breakable for "grapheme" and "word" but not for "sentence"
const granularities = [undefined, "grapheme", "word"];
// The following all contains more than one segments in either "grapheme" or "word"
// granularity.
const inputs = [
    "123 ",
    "a ",
    " a",
    " \ud800\udc00", // SPACE + surrogate
    "\ud800\udc00 ", // surrogate + SPACE
    "\udc00\ud800", // incorrect surrogate- tail + leading
    "\ud800 ", // only leading surrogate + SPACE
    "\udc00 ", // only trailing surrogate + SPACE
    " \ud800", // SPACE + only leading surrogate
    " \udc00", // SPACE + only trailing surrogate
    " 台", // SPACE + a Han character
    "台 ", // a Han character + SPACE
    "\u0301 ", // a modifier + SPACE
];

granularities.forEach(
    function(granularity) {
      const segmenter = new Intl.Segmenter(undefined, {granularity});
      inputs.forEach(function(input) {
        const segment = segmenter.segment(input);
        let msg = `granularity: ${granularity} input: ${input}`;
        const first = segment.containing(0);
        assert.sameValue(0, first.index, `${msg} containing(0) index`);
        assert.sameValue(input, first.input, `${msg} containing(0) input`);
        assert.sameValue(false, first.segment == input,
            `${msg} containing(0) segment`);
        const last = segment.containing(input.length - 1);
        msg += ` containing(${input.length - 1}) `
        assert.sameValue(true, last.index > 0, `${msg} index > 0`);
        assert.sameValue(true, last.index < input.length, `${msg} index`);
        assert.sameValue(input, last.input, `${msg} input`);
        assert.sameValue(false, last.segment == input, `${msg} segment`);
      });
    });
