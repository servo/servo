// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.plaintimeiso
description: The "plainTimeISO" property of Temporal.Now
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(typeof Temporal.Now.plainTimeISO, "function", "typeof is function");

verifyProperty(Temporal.Now, "plainTimeISO", {
  enumerable: false,
  writable: true,
  configurable: true
});
