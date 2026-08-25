// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Mapper returned iterator return is called when result iterator is closed
info: |
  %Iterator.prototype%.flatMap ( mapper )

features: [iterator-helpers]
flags: []
---*/
let returnCount = 0;

function* g() {
  yield 0;
}

let iter = g().flatMap(v => ({
  next() {
    return {
      done: false,
      value: 1,
    };
  },
  return() {
    ++returnCount;
    return {};
  },
}));

assert.sameValue(returnCount, 0);

let { done, value } = iter.next();

assert.sameValue(done, false);
assert.sameValue(value, 1);

assert.sameValue(returnCount, 0);

iter.return();
assert.sameValue(returnCount, 1);

iter.return();
assert.sameValue(returnCount, 1);
