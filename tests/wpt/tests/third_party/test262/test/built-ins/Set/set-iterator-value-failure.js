// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set-constructor
description: >
    Set ( [ iterable ] )

    When the Set function is called with optional argument iterable the following steps are taken:

    ...
    9. Repeat
      ...
      d. Let nextValue be IteratorValue(next).
      e. ReturnIfAbrupt(nextValue).
features: [Symbol.iterator]
---*/

var count = 0;
var iterable = {};

function MyError() {}
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        get value() {
          throw new MyError();
        },
        done: false
      };
    }
  };
};

assert.throws(MyError, function() {
  new Set(iterable);
});
