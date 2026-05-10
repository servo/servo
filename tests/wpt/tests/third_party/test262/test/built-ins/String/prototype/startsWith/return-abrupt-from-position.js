// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  Returns abrupt from ToInteger(position).
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  ...
  9. Let pos be ToInteger(position). (If position is undefined, this step
  produces the value 0).
  10. ReturnIfAbrupt(pos).
  ...
---*/

var position = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  ''.startsWith('', position);
});
