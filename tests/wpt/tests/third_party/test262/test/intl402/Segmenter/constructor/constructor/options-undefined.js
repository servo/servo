// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks handling of non-object option arguments to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    4. If options is undefined, then
        a. Let options be ObjectCreate(null).
features: [Intl.Segmenter]
---*/

Object.defineProperties(Object.prototype, {
  "localeMatcher": {
    "get": function() {
      throw new Error("Should not call getter on Object.prototype: localeMatcher");
    },
  },

  "lineBreakStyle": {
    "get": function() {
      throw new Error("Should not call getter on Object.prototype: lineBreakStyle");
    },
  },

  "granularity": {
    "get": function() {
      throw new Error("Should not call getter on Object.prototype: granularity");
    },
  },
});

const optionsArguments = [
  [],
  [[]],
  [[], undefined],
];

for (const args of optionsArguments) {
  const segmenter = new Intl.Segmenter(...args);
  const resolvedOptions = segmenter.resolvedOptions();
  assert.sameValue(resolvedOptions.granularity, "grapheme",
    `Calling with ${args.length} empty arguments should yield the correct value for "granularity"`);
  assert(
    !Object.prototype.hasOwnProperty.call(resolvedOptions, "lineBreakStyle"),
    `Calling with ${args.length} empty arguments should yield the correct value for "lineBreakStyle"`);
}
