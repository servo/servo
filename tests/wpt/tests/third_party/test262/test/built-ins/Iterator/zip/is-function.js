// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Iterator.zip is a built-in function
features: [joint-iteration]
---*/

assert.sameValue(
  typeof Iterator.zip,
  "function",
  "The value of `typeof Iterator.zip` is 'function'"
);
