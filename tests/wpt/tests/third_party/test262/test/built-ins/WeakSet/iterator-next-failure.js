// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  Return abrupt from next iterator step.
info: |
  23.4.1.1 WeakSet ( [ iterable ] )

  ...
  9. Repeat
    a. Let next be IteratorStep(iter).
    b. ReturnIfAbrupt(next).
    ...
features: [Symbol.iterator]
---*/

var iterable = {};
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      throw new Test262Error();
    }
  };
};

assert.throws(Test262Error, function() {
  new WeakSet(iterable);
});
