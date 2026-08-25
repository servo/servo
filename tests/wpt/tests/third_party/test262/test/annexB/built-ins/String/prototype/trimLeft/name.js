// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimLeft
description: >
  String.prototype.trimLeft.name is "trimStart".
info: >
  String.prototype.trimLeft ( )

  The function object that is the initial value of  String.prototype.trimLeft is the same function object that is the initial value of  String.prototype.trimStart.

includes: [propertyHelper.js]
features: [string-trimming, String.prototype.trimStart]
---*/

verifyProperty(String.prototype.trimLeft, "name", {
  value: "trimStart",
  enumerable: false,
  writable: false,
  configurable: true,
});
