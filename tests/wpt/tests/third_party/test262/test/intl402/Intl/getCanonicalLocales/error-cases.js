// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Tests the getCanonicalLocales function for error tags.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
  1. Let ll be ? CanonicalizeLocaleList(locales).
  2. Return CreateArrayFromList(ll).
features: [Symbol]
---*/

var rangeErrorCases =
  [
   "en-us-",
   "-en-us",
   "en-us-en-us",
   "--",
   "-",
   "",
   "-e-"
  ];

rangeErrorCases.forEach(function(re) {
  assert.throws(RangeError, function() {
    Intl.getCanonicalLocales(re);
  });
});

var typeErrorCases =
  [
    null,
    [null],
    [undefined],
    [true],
    [NaN],
    [2],
    [Symbol('foo')]
  ];

typeErrorCases.forEach(function(te) {
  assert.throws(TypeError, function() {
    Intl.getCanonicalLocales(te);
  });
});
