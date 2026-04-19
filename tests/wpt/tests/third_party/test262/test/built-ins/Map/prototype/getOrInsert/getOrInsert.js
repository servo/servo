// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Property type and descriptor.
info: |
  Map.prototype.getOrInsert ( key , value )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [upsert]
---*/
assert.sameValue(
  typeof Map.prototype.getOrInsert,
  'function',
  '`typeof Map.prototype.getOrInsert` is `function`'
);


verifyProperty(Map.prototype, "getOrInsert", {
  value: Map.prototype.getOrInsert,
  writable: true,
  enumerable: false,
  configurable: true
});


