// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13 S5.n
description: >
    Control flow during body evaluation should honor labeled `break`
    statements.
features: [generators]
---*/

function* values() {
  yield 1;
  throw new Test262Error('This code is unreachable (following `yield` statement).');
}
var iterator = values();
var i = 0;

outer:
while (true) {
  for (var x of iterator) {
    i++;
    break outer;

    throw new Test262Error('This code is unreachable (following `break` statement).');
  }
  throw new Test262Error('This code is unreachable (following `for..of` statement).');
}

assert.sameValue(i, 1);
