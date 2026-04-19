// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializenumberformat
description: Abrupt completion from invalid values for `roundingMode`
features: [Intl.NumberFormat-v3]
---*/

assert.throws(RangeError, function() {
  new Intl.NumberFormat('en', {roundingMode: null});
}, 'null');

assert.throws(RangeError, function() {
  new Intl.NumberFormat('en', {roundingMode: 3});
}, 'number');

assert.throws(RangeError, function() {
  new Intl.NumberFormat('en', {roundingMode: true});
}, 'boolean');

assert.throws(RangeError, function() {
  new Intl.NumberFormat('en', {roundingMode: 'HalfExpand'});
}, 'invalid string');

var symbol = Symbol('halfExpand');
assert.throws(TypeError, function() {
  new Intl.NumberFormat('en', {roundingMode: symbol});
}, 'Symbol');

var brokenToString = { toString: function() { throw new Test262Error(); } };
assert.throws(Test262Error, function() {
  new Intl.NumberFormat('en', {roundingMode: brokenToString});
}, 'broken `toString` implementation');
