// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-constructor
description: >
  The DataView Constructor
includes: [propertyHelper.js]
---*/

verifyProperty(this, "DataView", {
  writable: true,
  enumerable: false,
  configurable: true
});
