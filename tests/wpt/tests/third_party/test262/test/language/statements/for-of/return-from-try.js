// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13
description: >
    Control flow during body evaluation should honor `return` statements within
    `try` blocks.
features: [generators]
---*/

function* values() {
  yield 1;
  throw new Test262Error('This code is unreachable (following `yield` statement).');
}
var iterator = values();
var i = 0;

var result = (function() {
  for (var x of iterator) {
    try {
      i++;
      return 34;

      throw new Test262Error('This code is unreachable (following `return` statement).');
    } catch(err) {
      throw new Test262Error('This code is unreachable (within `catch` block).');
    }

    throw new Test262Error('This code is unreachable (following `try` statement).');
  }

  throw new Test262Error('This code is unreachable (following `for..in` statement).');
})();

assert.sameValue(result, 34);
assert.sameValue(i, 1);
