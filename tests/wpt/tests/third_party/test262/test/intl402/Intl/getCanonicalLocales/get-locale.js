// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Test Intl.getCanonicalLocales for step 7.c.i. 
info: |
  9.2.1 CanonicalizeLocaleList (locales)
    7. Repeat, while k < len.
      c. If kPresent is true, then
        i. Let kValue be ? Get(O, Pk).
---*/

var locales = {
  '0': 'en-US',
  length: 2
};

Object.defineProperty(locales, "1", {
  get: function() { throw new Test262Error() }
});

assert.throws(Test262Error, function() {
  Intl.getCanonicalLocales(locales);
});
