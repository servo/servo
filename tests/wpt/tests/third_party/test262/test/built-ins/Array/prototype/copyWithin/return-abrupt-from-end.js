// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Return abrupt from ToInteger(end).
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  11. If end is undefined, let relativeEnd be len; else let relativeEnd be
  ToInteger(end).
  12. ReturnIfAbrupt(relativeEnd).
  ...
---*/

var o1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};
assert.throws(Test262Error, function() {
  [].copyWithin(0, 0, o1);
});
