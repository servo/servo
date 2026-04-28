// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padend
description: String#padEnd should have length 1
author: Jordan Harband
includes: [propertyHelper.js]
---*/

verifyProperty(String.prototype.padEnd, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
