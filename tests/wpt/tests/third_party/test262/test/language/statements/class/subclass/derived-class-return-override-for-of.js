// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: >
  TypeError from `return 0` is thrown after the function body has been left, so
  an error thrown from an iterator has precedence.
---*/

var error = new Test262Error();

var iter = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {done: false};
  },
  return() {
    throw error;
  },
};

class C extends class {} {
  constructor() {
    super();

    for (var k of iter) {
      return 0;
    }
  }
}

assert.throws(Test262Error, function() {
  new C();
});
