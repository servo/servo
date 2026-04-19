// Copyright (C) 2017 The Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimstart
description: >
  "trimStart" property of String.prototype
info: >
  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [string-trimming, String.prototype.trimStart]
---*/

verifyProperty(String.prototype, "trimStart", {
  enumerable: false,
  writable: true,
  configurable: true,
});
