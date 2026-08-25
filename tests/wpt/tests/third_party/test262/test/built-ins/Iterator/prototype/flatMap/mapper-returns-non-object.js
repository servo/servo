// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap mapper return value must be an object
info: |
  %Iterator.prototype%.flatMap ( mapper )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
  yield 0;
  yield 0;
  yield 1;
}

let iter = g();

let mapperCalls = 0;
iter = iter.flatMap(v => {
  ++mapperCalls;
  return null;
});

assert.sameValue(mapperCalls, 0);

assert.throws(TypeError, function () {
  iter.next();
});

assert.sameValue(mapperCalls, 1);
