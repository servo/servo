// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setyear
es6id: B.2.4.2
es5id: B.2.5
description: >
    Behavior when calling ToNumber on year value returns an abrupt completion
info: |
    [...]
    3. Let y be ? ToNumber(year).
features: [Symbol]
---*/

var date = new Date(0);
var symbol = Symbol('');
var year = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  date.setYear(year);
});

assert.throws(TypeError, function() {
  date.setYear(symbol);
});
