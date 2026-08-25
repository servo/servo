// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2025 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getOrInsert
description: |
  WeakMap.prototype.getOrInsert.length descriptor
info: |
  WeakMap.prototype.getOrInsert ( key, value )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [upsert]
---*/
verifyProperty(WeakMap.prototype.getOrInsert, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});

