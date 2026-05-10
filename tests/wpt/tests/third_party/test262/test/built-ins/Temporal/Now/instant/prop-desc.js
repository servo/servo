// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.instant
description: The "instant" property of Temporal.Now
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(typeof Temporal.Now.instant, "function", "typeof is function");

verifyProperty(Temporal.Now, 'instant', {
  enumerable: false,
  writable: true,
  configurable: true
});
