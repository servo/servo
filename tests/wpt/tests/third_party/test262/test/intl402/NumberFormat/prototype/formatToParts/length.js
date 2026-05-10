// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Intl.NumberFormat.prototype.formatToParts.length. 
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.NumberFormat.prototype.formatToParts, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
