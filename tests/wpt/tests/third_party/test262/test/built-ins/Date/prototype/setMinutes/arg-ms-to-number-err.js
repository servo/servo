// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setminutes
description: Abrupt completion during type coercion of provided "ms"
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let m be ? ToNumber(min).
  3. If sec is not specified, let s be SecFromTime(t); otherwise, let s be ?
     ToNumber(sec).
  4. If ms is not specified, let milli be msFromTime(t); otherwise, let milli
     be ? ToNumber(ms).
---*/

var date = new Date(0);
var originalValue = date.getTime();
var obj = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  date.setMinutes(0, 0, obj);
});

assert.sameValue(date.getTime(), originalValue);
