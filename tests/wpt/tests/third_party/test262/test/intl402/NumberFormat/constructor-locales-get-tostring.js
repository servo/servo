// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
  Tests that Get(O, P) and ToString(arg) are properly called within the
  constructor for Intl.NumberFormat
info: |
  9.2.1 CanonicalizeLocaleList ( locales )

  5. Let len be ? ToLength(? Get(O, "length")).

  7.a. Let Pk be ToString(k).

  7.c.i. Let kValue be ? Get(O, Pk).
---*/

const locales = {
  length: 8,
  1: 'en-US',
  3: 'de-DE',
  5: 'en-IN',
  7: 'en-GB'
};

const actualLookups = [];
const expectedLookups = Object.keys(locales);

const handlers = {
  get(obj, prop) {
    actualLookups.push(prop);
    return Reflect.get(...arguments);
  }
};

const proxyLocales = new Proxy(locales, handlers);

const nf = new Intl.NumberFormat(proxyLocales);

expectedLookups.forEach(lookup => assert(actualLookups.indexOf(lookup) != -1));
