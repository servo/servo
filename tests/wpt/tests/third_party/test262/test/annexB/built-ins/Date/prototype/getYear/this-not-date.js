// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getyear
es6id: B.2.4.1
es5id: B.2.4
description: Behavior when `this` value has no [[DateValue]] internal slot
info: |
    1. Let t be ? thisTimeValue(this value).
---*/

var getYear = Date.prototype.getYear;

assert.sameValue(typeof getYear, 'function');

assert.throws(TypeError, function() {
  getYear.call({});
}, 'object');

assert.throws(TypeError, function() {
  getYear.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  getYear.call(null);
}, 'null');
