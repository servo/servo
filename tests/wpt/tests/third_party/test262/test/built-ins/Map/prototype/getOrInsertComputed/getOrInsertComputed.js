// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Property type and descriptor.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [upsert]
---*/
verifyPrimordialCallableProperty(Map.prototype, "getOrInsertComputed", "getOrInsertComputed", 2, {
  value: Map.prototype.getOrInsertComputed,
  writable: true,
  enumerable: false,
  configurable: true
});

