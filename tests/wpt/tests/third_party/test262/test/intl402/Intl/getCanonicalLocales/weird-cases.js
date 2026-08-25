// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Tests the getCanonicalLocales function for weird tags.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
  1. Let ll be ? CanonicalizeLocaleList(locales).
  2. Return CreateArrayFromList(ll).
includes: [compareArray.js]
---*/

var weirdCases =
  [
   "en-x-u-foo",
   "en-a-bar-x-u-foo",
   "en-x-u-foo-a-bar",
   "en-a-bar-u-baz-x-u-foo",
  ];

weirdCases.forEach(function (weird) {
  assert.compareArray(Intl.getCanonicalLocales(weird), [weird]);
});
