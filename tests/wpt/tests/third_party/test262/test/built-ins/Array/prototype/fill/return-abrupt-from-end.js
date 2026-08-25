// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.fill
description: >
  Return abrupt from ToInteger(end).
info: |
  22.1.3.6 Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  8. If end is undefined, let relativeEnd be len; else let relativeEnd be
  ToInteger(end).
  9. ReturnIfAbrupt(relativeEnd).
  ...
---*/

var end = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  [].fill(1, 0, end);
});
