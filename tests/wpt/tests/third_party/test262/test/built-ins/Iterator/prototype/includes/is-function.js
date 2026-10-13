// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes is a function
features: [iterator-includes]
---*/

assert.sameValue(typeof Iterator.prototype.includes, 'function');
