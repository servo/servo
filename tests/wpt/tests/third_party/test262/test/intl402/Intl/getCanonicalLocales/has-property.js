// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Test Intl.getCanonicalLocales.name for step 7.b. 
info: |
  9.2.1 CanonicalizeLocaleList (locales)
    7. Repeat, while k < len.
      b. Let kPresent be HasProperty(O, Pk).
features: [Proxy]
---*/

var locales = {
  '0': 'en-US',
  '1': 'pt-BR',
  length: 2
};

var p = new Proxy(locales, {
  has: function(_, prop) {
    if (prop === '0') {
      throw new Test262Error();
    }
  }
});

assert.throws(Test262Error, function() {
  Intl.getCanonicalLocales(p);
});
