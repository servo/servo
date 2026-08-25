// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Property type and descriptor. 
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Intl.DateTimeFormat.prototype.formatToParts,
  'function',
  '`typeof Intl.DateTimeFormat.prototype.formatToParts` is `function`'
);

verifyProperty(Intl.DateTimeFormat.prototype, "formatToParts", {
  writable: true,
  enumerable: false,
  configurable: true,
});
