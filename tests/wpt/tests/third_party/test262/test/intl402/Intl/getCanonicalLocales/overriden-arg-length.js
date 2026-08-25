// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Test Intl.getCanonicalLocales for step 5. 
info: |
  9.2.1 CanonicalizeLocaleList (locales)
    5. Let len be ? ToLength(? Get(O, "length")).
includes: [compareArray.js]
features: [Symbol]
---*/

var locales = {
  '0': 'en-US',
};

Object.defineProperty(locales, "length", {
  get: function() { throw new Test262Error() }
});

assert.throws(Test262Error, function() {
  Intl.getCanonicalLocales(locales);
}, "should throw if locales.length throws");

var locales = {
  '0': 'en-US',
  '1': 'pt-BR',
};

Object.defineProperty(locales, "length", {
  get: function() { return "1" }
});

assert.compareArray(
  Intl.getCanonicalLocales(locales),
  ['en-US'],
  "should return one element if locales.length is '1'"
);

var locales = {
  '0': 'en-US',
  '1': 'pt-BR',
};

Object.defineProperty(locales, "length", {
  get: function() { return 1.3 }
});

assert.compareArray(
  Intl.getCanonicalLocales(locales),
  ['en-US'],
  "should return one element if locales.length is 1.3"
);

var locales = {
  '0': 'en-US',
  '1': 'pt-BR',
};

Object.defineProperty(locales, "length", {
  get: function() { return Symbol("1.8") }
});

assert.throws(TypeError, function() {
  Intl.getCanonicalLocales(locales);
}, "should throw if locales.length is a Symbol");

var locales = {
  '0': 'en-US',
  '1': 'pt-BR',
};

Object.defineProperty(locales, "length", {
  get: function() { return -Infinity }
});

assert.compareArray(
  Intl.getCanonicalLocales(locales),
  [],
  "should return empty array if locales.length is -Infinity"
);

var locales = {
  length: -Math.pow(2, 32) + 1
};

Object.defineProperty(locales, "0", {
  get: function() { throw new Error("must not be gotten!"); }
})

assert.compareArray(
  Intl.getCanonicalLocales(locales),
  [],
  "should return empty array if locales.length is a negative value"
);

var count = 0;
var locs = { get length() { if (count++ > 0) throw 42; return 0; } };
var locales = Intl.getCanonicalLocales(locs); // shouldn't throw 42
assert.sameValue(locales.length, 0);
