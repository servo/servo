// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.intersection
description: Tests that Set.prototype.intersection meets the requirements for built-in objects
features: [set-methods]
---*/

assert.sameValue(
  Object.isExtensible(Set.prototype.intersection),
  true,
  "Built-in objects must be extensible."
);

assert.sameValue(
  Object.prototype.toString.call(Set.prototype.intersection),
  "[object Function]",
  "Object.prototype.toString"
);

assert.sameValue(
  Object.getPrototypeOf(Set.prototype.intersection),
  Function.prototype,
  "prototype"
);
