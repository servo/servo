// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Throws a TypeError if this is not a DateTimeFormat object
features: [Intl.DateTimeFormat-formatRange]
---*/

const formatRangeToParts = Intl.DateTimeFormat.prototype.formatRangeToParts;

assert.throws(TypeError, function() {
  formatRangeToParts.call({});
}, "{}");

assert.throws(TypeError, function() {
  formatRangeToParts.call(new Date());
}, "new Date()");

assert.throws(TypeError, function() {
  formatRangeToParts.call(Intl.DateTimeFormat);
}, "Intl.DateTimeFormat");

assert.throws(TypeError, function() {
  formatRangeToParts.call(Intl.DateTimeFormat.prototype);
}, "Intl.DateTimeFormat.prototype");
