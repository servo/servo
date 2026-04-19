// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf does not implement [[Construct]], is not new-able
includes: [isConstructor.js]
features: [Reflect.construct, set-methods]
---*/

assert.sameValue(
  isConstructor(Set.prototype.isSubsetOf),
  false,
  "isConstructor(Set.prototype.isSubsetOf) must return false"
);

assert.throws(
  TypeError,
  () => {
    new Set.prototype.isSubsetOf();
  });
