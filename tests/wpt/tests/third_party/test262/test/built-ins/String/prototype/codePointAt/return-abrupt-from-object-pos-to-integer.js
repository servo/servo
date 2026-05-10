// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.3
description: >
  Returns abrupt from ToInteger(pos)
info: |
  21.1.3.3 String.prototype.codePointAt ( pos )

  1. Let O be RequireObjectCoercible(this value).
  2. Let S be ToString(O).
  3. ReturnIfAbrupt(S).
  4. Let position be ToInteger(pos).
  5. ReturnIfAbrupt(position).
---*/

var o = {
  valueOf: function() {
    throw new Test262Error();
  }
}

assert.throws(Test262Error, function() {
  'abc'.codePointAt(o);
});
