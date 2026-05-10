// Copyright (C) 2019 Bocoup. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-value-properties-of-the-global-object-nan
description: Property descriptor of NaN
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

verifyProperty(this, "NaN", {
  enumerable: false,
  writable: false,
  configurable: false
});
