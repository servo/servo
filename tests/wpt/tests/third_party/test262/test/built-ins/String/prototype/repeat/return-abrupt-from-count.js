// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.13
description: >
  Returns abrupt from ToInteger(count)
info: |
  21.1.3.13 String.prototype.repeat ( count )

  4. Let n be ToInteger(count).
  5. ReturnIfAbrupt(n).
---*/

var o = {
  toString: function() {
    throw new Test262Error();
  }
}

assert.throws(Test262Error, function() {
  ''.repeat(o);
});
