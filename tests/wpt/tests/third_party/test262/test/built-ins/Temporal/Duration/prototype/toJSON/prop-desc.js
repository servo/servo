// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tojson
description: The "toJSON" property of Temporal.Duration.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.prototype.toJSON,
  "function",
  "`typeof Duration.prototype.toJSON` is `function`"
);

verifyProperty(Temporal.Duration.prototype, "toJSON", {
  writable: true,
  enumerable: false,
  configurable: true,
});
