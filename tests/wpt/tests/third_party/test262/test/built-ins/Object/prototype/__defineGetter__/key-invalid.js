// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Behavior when property key cannot be derived
info: |
    [...]
    4. Let key be ? ToPropertyKey(P).
features: [__getter__]
---*/

var noop = function() {};
var subject = {};
var key = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  subject.__defineGetter__(key, noop);
});
