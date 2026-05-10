// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Iterator.prototype.reduce expects to be called with a callable argument.
info: |
  %Iterator.prototype%.reduce ( reducer )

features: [iterator-helpers]
flags: []
---*/
let nonCallable = {};
function* gen() {
  yield 1;
}

assert.throws(TypeError, function () {
  gen().reduce(nonCallable);
});

gen().reduce(() => {});
