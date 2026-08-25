// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf does not implement [[Construct]], is not new-able
includes: [isConstructor.js]
features: [Reflect.construct, set-methods]
---*/

assert.sameValue(
  isConstructor(Set.prototype.isSupersetOf),
  false,
  "isConstructor(Set.prototype.isSupersetOf) must return false"
);

assert.throws(
  TypeError,
  () => {
    new Set.prototype.isSupersetOf();
  });
