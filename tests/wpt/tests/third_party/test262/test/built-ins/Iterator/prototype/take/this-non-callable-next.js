// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Iterator.prototype.take throws TypeError when its this value is an object with a non-callable next
info: |
  %Iterator.prototype%.take ( limit )

  7. Let iterated be ? GetIteratorDirect(this value).

features: [iterator-helpers]
flags: []
---*/
let iter = Iterator.prototype.take.call({ next: 0 }, 1);

assert.throws(TypeError, function () {
  iter.next();
});
