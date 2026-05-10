// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
  Tests that HasProperty(O, Pk) is properly called within the constructor for
  Intl.NumberFormat
info: |
  9.2.1 CanonicalizeLocaleList ( locales )

  7.b. Let kPresent be ? HasProperty(O, Pk).
---*/

const locales = {
  length: 8,
  1: 'en-US',
  3: 'de-DE',
  5: 'en-IN',
  7: 'en-GB'
};

const actualLookups = [];

const handlers = {
  has(obj, prop) {
    actualLookups.push(prop);
    return Reflect.has(...arguments);
  }
};

const proxyLocales = new Proxy(locales, handlers);

const nf = new Intl.NumberFormat(proxyLocales);

assert.sameValue(actualLookups.length, locales.length);
for (let index in actualLookups) {
  assert.sameValue(actualLookups[index], String(index));
}
