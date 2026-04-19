// Copyright 2018 the V8 project authors, Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks handling of valid values for the granularity option to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    11. Let granularity be ? GetOption(options, "granularity", "string", « "grapheme", "word", "sentence" », "grapheme").
    12. Set segmenter.[[SegmenterGranularity]] to granularity.
features: [Intl.Segmenter]
---*/

const granularityOptions = ["grapheme", "word", "sentence"];
const combinations = [];

combinations.push([
  {},
  "grapheme",
  undefined,
]);

for (const granularity of granularityOptions) {
  combinations.push([
    { granularity },
    granularity,
    undefined,
  ]);
}

for (const [input, granularity, lineBreakStyle] of combinations) {
  const segmenter = new Intl.Segmenter([], input);
  const resolvedOptions = segmenter.resolvedOptions();
  assert.sameValue(resolvedOptions.granularity, granularity);
}
