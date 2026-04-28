// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  If the iterable argument is empty, return new Weakset object.
info: |
  23.4.1.1 WeakSet ( [ iterable ] )

  ...
  9. Repeat
    ...
    d. Let nextValue be IteratorValue(next).
    e. ReturnIfAbrupt(nextValue).
features: [Symbol.iterator]
---*/

var count = 0;
var iterable = {};
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        get value() {
          throw new Test262Error();
        },
        done: false
      };
    }
  };
};

assert.throws(Test262Error, function() {
  new WeakSet(iterable);
});
