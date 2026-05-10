// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padstart
description: String#padStart should have name property with value 'padStart'
author: Jordan Harband
includes: [propertyHelper.js]
---*/

verifyProperty(String.prototype.padStart, "name", {
  value: "padStart",
  writable: false,
  enumerable: false,
  configurable: true
});
