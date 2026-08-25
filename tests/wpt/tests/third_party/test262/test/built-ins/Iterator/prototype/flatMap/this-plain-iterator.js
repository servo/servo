// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap supports a this value that does not inherit from Iterator.prototype but implements the iterator protocol
info: |
  %Iterator.prototype%.flatMap ( mapper )

features: [iterator-helpers]
flags: []
---*/
let iter = {
  get next() {
    let count = 3;
    return function () {
      --count;
      return count >= 0 ? { done: false, value: count } : { done: true, value: undefined };
    };
  },
};

let mapperCalls = 0;
iter = Iterator.prototype.flatMap.call(iter, function (v) {
  ++mapperCalls;
  return [v];
});

for (let e of iter);

assert.sameValue(mapperCalls, 3);
