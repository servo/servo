// Copyright 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.pause
description: Atomics.pause.length is 0.
includes: [propertyHelper.js]
features: [Atomics.pause]
---*/

verifyProperty(Atomics.pause, 'length', {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true,
});
