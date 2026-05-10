// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.7
description: >
  Returns abrupt from ToString(this)
info: |
  21.1.3.7 String.prototype.includes ( searchString [ , position ] )

  1. Let O be RequireObjectCoercible(this value).
  2. Let S be ToString(O).
  3. ReturnIfAbrupt(S).
features: [String.prototype.includes]
---*/
var o = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  String.prototype.includes.call(o, '');
});
