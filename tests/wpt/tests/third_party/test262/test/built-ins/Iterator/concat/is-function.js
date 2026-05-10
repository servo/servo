// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Iterator.concat is a built-in function
features: [iterator-sequencing]
---*/

assert.sameValue(
  typeof Iterator.concat,
  "function",
  "The value of `typeof Iterator.concat` is 'function'"
);
