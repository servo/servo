// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13 S5.n
description: >
    Control flow during body evaluation should honor `continue` statements
    within the `finally` block of `try` statements.
features: [generators]
---*/

function* values() {
  yield 1;
  yield 1;
}
var iterator = values();
var loop = true;
var i = 0;

outer:
while (loop) {
  loop = false;

  for (var x of iterator) {
    try {
      throw new Error();
    } catch (err) {
    } finally {
      i++;
      continue outer;

      throw new Test262Error('This code is unreachable (following `continue` statement).');
    }

    throw new Test262Error('This code is unreachable (following `try` statement).');
  }

  throw new Test262Error('This code is unreachable (following `for..of` statement).');
}

assert.sameValue(i, 1);
