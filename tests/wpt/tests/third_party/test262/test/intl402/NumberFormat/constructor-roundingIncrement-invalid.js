// Copyright 2021 the V8 project authors. All rights reserved.
// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializenumberformat
description: Rejects invalid values for roundingIncrement option.
features: [Intl.NumberFormat-v3]
---*/

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 0});
}, '0');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 3});
}, '3');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 4});
}, '4');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 5000.1});
}, '5000.1');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 5001});
}, '5001');

assert.throws(TypeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 2, roundingPriority: 'morePrecision'});
}, '2, roundingType is "morePrecision"');

assert.throws(TypeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 2, roundingPriority: 'lessPrecision'});
}, '2, roundingType is "lessPrecision"');

assert.throws(TypeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 2, minimumSignificantDigits: 1});
}, '2, roundingType is "significantDigits"');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {roundingIncrement: 2, maximumFractionDigits:3 , minimumFractionDigits:2 });
}, '"maximumFractionDigits" is not equal to "minimumFractionDigits"');
