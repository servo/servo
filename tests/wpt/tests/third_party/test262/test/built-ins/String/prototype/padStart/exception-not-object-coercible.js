// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padstart
description: >
  String#padStart should fail if given a null or undefined value,
  or an object not coercible to a string.
author: Jordan Harband
---*/

assert.throws(TypeError, function() {
  String.prototype.padStart.call(null);
});

assert.throws(TypeError, function() {
  String.prototype.padStart.call(undefined);
});

var notCoercible = {
  toString: function() {
    throw new Test262Error('attempted toString');
  }
};

assert.throws(Test262Error, function() {
  String.prototype.padStart.call(notCoercible);
});
