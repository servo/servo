// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: Return abrupt completion from GetOption granularity
info: |
    Intl.Segmenter ([ locales [ , options ]])

    13. Let granularity be ? GetOption(options, "granularity", "string", « "grapheme", "word", "sentence" », "grapheme").

    GetOption ( options, property, type, values, fallback )
    1. Let value be ? Get(options, property).
features: [Intl.Segmenter]
---*/


var options = {};
Object.defineProperty(options, 'granularity', {
  get() { throw new Test262Error(); },
});

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
});
