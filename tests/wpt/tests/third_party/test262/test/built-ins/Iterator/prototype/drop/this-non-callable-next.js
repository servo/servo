// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Iterator.prototype.drop throws TypeError when its this value is an object with a non-callable next
info: |
  %Iterator.prototype%.drop ( limit )

  1. Let iterated be ? GetIteratorDirect(this value).

features: [iterator-helpers]
flags: []
---*/
let iter = Iterator.prototype.drop.call({ next: 0 }, 1);

assert.throws(TypeError, function () {
  iter.next();
});
