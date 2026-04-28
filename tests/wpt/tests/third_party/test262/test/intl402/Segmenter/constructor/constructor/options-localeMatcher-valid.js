// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: >
  Valid options for localeMatcher
info: |
  Intl.Segmenter ([ locales [ , options ]])

  1. If NewTarget is undefined, throw a TypeError exception.
  3. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%Segmenter.prototype%", internalSlotsList).
  ...
  8. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
  ...

  GetOption ( options, property, type, values, fallback )

  1. Let value be ? Get(options, property).
  ...
features: [Intl.Segmenter]
locale: [en]
---*/

// results for option values verified in the tests for resolvedOptions

const localeMatchers = [
  undefined,
  'lookup',
  'best fit'
];

localeMatchers.forEach(localeMatcher => {
  const obj = new Intl.Segmenter(undefined, { localeMatcher });

  assert(obj instanceof Intl.Segmenter, `instanceof check - ${localeMatcher}`);
  assert.sameValue(Object.getPrototypeOf(obj), Intl.Segmenter.prototype, `proto check - ${localeMatcher}`);
});
