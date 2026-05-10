// Copyright 2017 Lyza Danger Gardner. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Testing descriptor property of Array.isArray
includes: [propertyHelper.js]
esid: sec-array.isarray
---*/

verifyProperty(Array, "isArray", {
  writable: true,
  enumerable: false,
  configurable: true
});
