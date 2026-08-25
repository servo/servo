// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.forEach
description: >
  Iterator.prototype.forEach throws TypeError when its this value is an object with a non-callable next
info: |
  %Iterator.prototype%.forEach ( fn )

features: [iterator-helpers]
flags: []
---*/
assert.throws(TypeError, function () {
  Iterator.prototype.forEach.call({ next: 0 }, () => true);
});
