// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  The desc argument should be an object or undefined
includes: [propertyHelper.js]
features: [Symbol]
---*/
var sample = { foo: 42 };

assert.throws(Test262Error, () => {
  verifyProperty(sample, "foo", 'configurable');
}, "string");

assert.throws(Test262Error, () => {
  verifyProperty(sample, 'foo', true);
}, "boolean");

assert.throws(Test262Error, () => {
  verifyProperty(sample, 'foo', 42);
}, "number");

assert.throws(Test262Error, () => {
  verifyProperty(sample, 'foo', null);
}, "null");

assert.throws(Test262Error, () => {
  verifyProperty(sample, 'foo', Symbol(1));
}, "symbol");
