// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.fromEntries.name is "fromEntries".
esid: sec-object.fromentries
includes: [propertyHelper.js]
features: [Object.fromEntries]
---*/

verifyProperty(Object.fromEntries, "name", {
  value: "fromEntries",
  enumerable: false,
  writable: false,
  configurable: true
});
