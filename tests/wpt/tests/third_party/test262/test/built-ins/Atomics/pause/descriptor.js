// Copyright 2024 the V8 project authors. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-atomics.pause
description: Testing descriptor property of Atomics.pause
includes: [propertyHelper.js]
features: [Atomics.pause]
---*/

verifyProperty(Atomics, 'pause', {
  enumerable: false,
  writable: true,
  configurable: true,
});
