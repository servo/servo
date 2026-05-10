// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tojson
description: The "toJSON" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.toJSON,
  "function",
  "`typeof ZonedDateTime.prototype.toJSON` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "toJSON", {
  writable: true,
  enumerable: false,
  configurable: true,
});
