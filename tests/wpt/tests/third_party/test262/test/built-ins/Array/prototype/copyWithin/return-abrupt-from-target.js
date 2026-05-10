// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Return abrupt from ToInteger(target).
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  5. Let relativeTarget be ToInteger(target).
  6. ReturnIfAbrupt(relativeTarget).
  ...
---*/

var o1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};
assert.throws(Test262Error, function() {
  [].copyWithin(o1);
});
