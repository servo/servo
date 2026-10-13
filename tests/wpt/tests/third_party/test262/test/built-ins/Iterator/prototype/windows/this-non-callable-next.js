// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows throws TypeError when its this value is an object
  with a non-callable next
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  7. Set iterated to ? GetIteratorDirect(O).

features: [iterator-chunking]
---*/
let iter = Iterator.prototype.windows.call({ next: 0 }, 1);

assert.throws(TypeError, function () {
  iter.next();
});
