// Copyright 2018 Igalia, S.L. All rights reserved.
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

const validOptions = [
  [undefined, "grapheme"],
  ["grapheme", "grapheme"],
  ["word", "word"],
  ["sentence", "sentence"],
  [{ toString() { return "word"; } }, "word"],
];

for (const [granularity, expected] of validOptions) {
  const segmenter = new Intl.Segmenter([], { granularity });
  const resolvedOptions = segmenter.resolvedOptions();
  assert.sameValue(resolvedOptions.granularity, expected);
}

assert.throws(RangeError, () => new Intl.Segmenter([], {granularity: "line"}));
