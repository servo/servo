// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Throws a TypeError if this is not a DateTimeFormat object
features: [Intl.DateTimeFormat-formatRange]
---*/

const formatRange = Intl.DateTimeFormat.prototype.formatRange;

assert.throws(TypeError, function() {
  formatRange.call({});
}, "{}");

assert.throws(TypeError, function() {
  formatRange.call(new Date());
}, "new Date()");

assert.throws(TypeError, function() {
  formatRange.call(Intl.DateTimeFormat);
}, "Intl.DateTimeFormat");

assert.throws(TypeError, function() {
  formatRange.call(Intl.DateTimeFormat.prototype);
}, "Intl.DateTimeFormat.prototype");
