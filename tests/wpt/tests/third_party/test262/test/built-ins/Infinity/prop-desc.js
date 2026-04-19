// Copyright (C) 2019 Bocoup. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-value-properties-of-the-global-object-infinity
description: Property descriptor of Infinity
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

verifyProperty(this, "Infinity", {
  enumerable: false,
  writable: false,
  configurable: false
});
