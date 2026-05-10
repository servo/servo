// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks handling of invalid value for the style option to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    13. Let granularity be ? GetOption(options, "granularity", "string", « "grapheme", "word", "sentence" », "grapheme").
    14. Set segmenter.[[SegmenterGranularity]] to granularity.
features: [Intl.Segmenter]
---*/

const invalidOptions = [
  null,
  1,
  "",
  "standard",
  "Grapheme",
  "GRAPHEME",
  "grapheme\0",
  "Word",
  "WORD",
  "word\0",
  "Sentence",
  "SENTENCE",
  "sentence\0",
  "line",
  "Line",
  "LINE",
  "line\0",
];

for (const granularity of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.Segmenter([], { granularity });
  }, `${granularity} is an invalid style option value`);
}
