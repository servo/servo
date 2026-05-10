// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  The iterator is closed when iterable `next` throws an error.
info: |
  Map ( [ iterable ] )

  ...
  9. Repeat
    a. Let next be IteratorStep(iter).
    b. ReturnIfAbrupt(next).
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
  new Map(iterable);
});
