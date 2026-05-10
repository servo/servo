// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13
description: >
    Control flow during body evaluation should honor `throw` statements within
    the `catch` block of `try` statements.
features: [generators]
---*/

function* values() {
  yield 1;
  throw new Test262Error('This code is unreachable (following `yield` statement).');
}
var CustomError = function() {};
var iterator = values();
var i = 0;
var error = new CustomError();

assert.throws(CustomError, function() {
  for (var x of iterator) {
    try {
      throw new Error();
    } catch (err) {
      i++;
      throw error;

      throw new Test262Error('This code is unreachable (following `throw` statement).');
    }

    throw new Test262Error('This code is unreachable (following `try` statement).');
  }

  throw new Test262Error('This code is unreachable (following `for..in` statement).');
});

assert.sameValue(i, 1);
