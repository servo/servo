// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype-@@tostringtag
description: The @@toStringTag property of Temporal.Duration
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Temporal.Duration.prototype, Symbol.toStringTag, {
  value: "Temporal.Duration",
  writable: false,
  enumerable: false,
  configurable: true,
});
