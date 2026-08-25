// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: >
  Return abrupt completion from GetOption localeMatcher
info: |
  Intl.Segmenter ([ locales [ , options ]])
  1. If NewTarget is undefined, throw a TypeError exception.
  ...
  4. If options is undefined, then
    a. Let options be ObjectCreate(null).
  5. Else
    a. Let options be ? ToObject(options).
  ...
  8. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
  GetOption ( options, property, type, values, fallback )
  1. Let value be ? Get(options, property).
  ...
features: [Intl.Segmenter]
---*/

var options = {};
Object.defineProperty(options, 'localeMatcher', {
  get() { throw new Test262Error(); },
});

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
});
