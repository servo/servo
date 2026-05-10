// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Intl.NumberFormat.prototype.formatToParts.name value and descriptor. 
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.NumberFormat.prototype.formatToParts, "name", {
  value: "formatToParts",
  writable: false,
  enumerable: false,
  configurable: true,
});
