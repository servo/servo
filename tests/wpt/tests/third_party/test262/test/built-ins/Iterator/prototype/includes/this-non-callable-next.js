// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes throws TypeError when its this value is an object
  with a non-callable next
features: [iterator-includes]
---*/

assert.throws(TypeError, function() {
  Iterator.prototype.includes.call({ next: 0 }, 0);
});
