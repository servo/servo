// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.13
description: >
  Throws a RangeError if count < 0
info: |
  21.1.3.13 String.prototype.repeat ( count )

  7. If n is +∞, throw a RangeError exception.
---*/

assert.throws(RangeError, function() {
  ''.repeat(Infinity);
});
