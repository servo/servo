// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.big
es6id: B.2.3.3
description: Abrupt completion when coercing "this" value to string
info: |
    B.2.3.2.1 Runtime Semantics: CreateHTML

    1. Let str be ? RequireObjectCoercible(string).
    2. Let S be ? ToString(str).
---*/

var thisVal = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  String.prototype.big.call(thisVal);
});
