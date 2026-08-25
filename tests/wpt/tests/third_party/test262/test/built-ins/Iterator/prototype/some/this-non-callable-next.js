// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator.prototype.some throws TypeError when its this value is an object with a non-callable next
info: |
  %Iterator.prototype%.some ( predicate )

features: [iterator-helpers]
flags: []
---*/
assert.throws(TypeError, function () {
  Iterator.prototype.some.call({ next: 0 }, () => true);
});
