// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Iterator.zipKeyed is a built-in function
features: [joint-iteration]
---*/

assert.sameValue(
  typeof Iterator.zipKeyed,
  "function",
  "The value of `typeof Iterator.zipKeyed` is 'function'"
);
