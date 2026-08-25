// Copyright (C) 2023 Anthony Frehner. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.union
description: Tests that Set.prototype.union meets the requirements for built-in objects
features: [set-methods]
---*/

assert.sameValue(
  Object.isExtensible(Set.prototype.union),
  true,
  "Built-in objects must be extensible."
);

assert.sameValue(
  Object.prototype.toString.call(Set.prototype.union),
  "[object Function]",
  "Object.prototype.toString"
);

assert.sameValue(
  Object.getPrototypeOf(Set.prototype.union),
  Function.prototype,
  "prototype"
);
