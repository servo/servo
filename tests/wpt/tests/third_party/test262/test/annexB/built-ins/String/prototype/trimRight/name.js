// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimRight
description: >
  String.prototype.trimRight.name is "trimEnd".
info: >
  String.prototype.trimRight ( )#

  The function object that is the initial value of  String.prototype.trimRight is the same function object that is the initial value of  String.prototype.trimEnd.
includes: [propertyHelper.js]
features: [string-trimming, String.prototype.trimEnd]
---*/

verifyProperty(String.prototype.trimRight, "name", {
  value: "trimEnd",
  enumerable: false,
  writable: false,
  configurable: true,
});
