// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13 S5.n
description: >
    Control flow during body evaluation should honor labeled `continue`
    statements.
features: [generators]
---*/

function* values() {
  yield 1;
}
var iterator = values();
var i = 0;
var loop = true;

outer:
while (loop) {
  loop = false;

  for (var x of iterator) {
    i++;
    continue outer;

    throw new Test262Error('This code is unreachable (inside for-of).');
  }
  throw new Test262Error('This code is unreachable (inside while).');
}

assert.sameValue(i, 1);
