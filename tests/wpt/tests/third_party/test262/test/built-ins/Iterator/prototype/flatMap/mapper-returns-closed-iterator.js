// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap handles closed return values from mapper and does not try to close them again
info: |
  %Iterator.prototype%.flatMap ( mapper )

features: [iterator-helpers]
flags: []
---*/

function* g() {
  yield 0;
  yield 1;
  yield 2;
}

let closed = g();
closed.return();
closed.return = function () {
  throw new Test262Error();
};

let iter = g().flatMap(v => closed);
let { value, done } = iter.next();

assert.sameValue(value, undefined);
assert.sameValue(done, true);
