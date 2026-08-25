// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the cases which the input is unbreakable.
info: |
    %Segments.prototype%.containing ( index )

    8. Let startIndex be ! FindBoundary(segmenter, string, n, before).
    9. Let endIndex be ! FindBoundary(segmenter, string, n, after).

features: [Intl.Segmenter]
---*/

const granularities = [undefined, "grapheme", "word", "sentence"];
// The following all contains only one segment in any granularity.
const inputs = [
    "a",
    " ",
    "\ud800\udc00", // surrogate
    "\ud800", // only leading surrogate
    "\udc00", // only trailing surrogate
    "台", // a Han character
    "\u0301", // a modifier
    "a\u0301", // ASCII + a modifier
    "ซิ่", // a Thai cluster
    "𐂰",  // a Surrogate pair
    "\uD83D\uDC4B\uD83C\uDFFB", // Emoji short sequence: waving_hand_light_skin_tone
    "\uD83D\uDC68\uD83C\uDFFB\u200D\uD83E\uDDB0", // Emoji long sequence: man_light_skin_tone_red_hair
    "\u1102",  // Jamo L
    "\u1162",  // Jamo V
    "\u11A9",  // Jamo T
    "\u1102\u1162",  // Jamo LV
    "\u1102\u1162\u11A9",  // Jamo LVT
    "\u1102\u1102",  // Jamo L L
    "\u1102\u1102\u1162",  // Jamo L L V
    "\u1102\u1102\u1162\u11A9",  // Jamo L L V T
    "\u1162\u1162",  // Jamo V V
    "\u1162\u11A9",  // Jamo V T
    "\u1102\u1162\u1162",  // Jamo V V
    "\u11A9\u11A9",  // Jamo T T
    "\u1102\u1162\u11A9\u11A9",  // Jamo LVT T
];

granularities.forEach(
    function(granularity) {
      const segmenter = new Intl.Segmenter(undefined, {granularity});
      inputs.forEach(function(input) {
        const segment = segmenter.segment(input);
        for (let index = 0; index < input.length; index++) {
          const result = segment.containing(index);
          assert.sameValue(0, result.index);
          assert.sameValue(input, result.input);
          assert.sameValue(input, result.segment);
        }
      });
    });
