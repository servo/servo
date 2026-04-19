// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.find
description: >
  Iterator.prototype.find is a built-in function
features: [iterator-helpers]
---*/

assert.sameValue(typeof Iterator.prototype.find, 'function');
