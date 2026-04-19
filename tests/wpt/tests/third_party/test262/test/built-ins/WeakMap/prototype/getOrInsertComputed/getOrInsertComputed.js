// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2025 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getorinsertcomputed
description: |
  WeakMap.prototype.getOrInsertComputed property descriptor
info: |
  WeakMap.prototype.getOrInsertComputed ( key, callbackfn )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [WeakMap, upsert]
---*/
verifyPrimordialCallableProperty(WeakMap.prototype, "getOrInsertComputed", "getOrInsertComputed", 2, {
  value: WeakMap.prototype.getOrInsertComputed,
  writable: true,
  enumerable: false,
  configurable: true
});

