// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializenumberformat
description: Rejects invalid values for trailingZeroDisplay option.
features: [Intl.NumberFormat-v3]
---*/

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {trailingZeroDisplay: ''});
}, 'empty string');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {trailingZeroDisplay: 'Auto'});
}, 'Auto');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {trailingZeroDisplay: 'StripIfInteger'});
}, 'StripIfInteger');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {trailingZeroDisplay: 'stripifinteger'});
}, 'stripifinteger');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {trailingZeroDisplay: ' auto'});
}, '" auto" (with leading space)');

assert.throws(RangeError, function() {
  new Intl.NumberFormat([], {trailingZeroDisplay: 'auto '});
}, '"auto " (with trailing space)');
