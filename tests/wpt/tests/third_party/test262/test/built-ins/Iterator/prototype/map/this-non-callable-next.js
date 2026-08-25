// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.map
description: >
  Iterator.prototype.map throws TypeError when its this value is an object with a non-callable next
info: |
  %Iterator.prototype%.map ( mapper )

features: [iterator-helpers]
flags: []
---*/
let iter = Iterator.prototype.map.call({ next: 0 }, () => 0);

assert.throws(TypeError, function () {
  iter.next();
});
