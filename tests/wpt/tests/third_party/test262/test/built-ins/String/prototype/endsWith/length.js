// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  String.prototype.endsWith.length value and descriptor.
info: |
  21.1.3.6 String.prototype.endsWith ( searchString [ , endPosition] )

  The length property of the endsWith method is 1.

includes: [propertyHelper.js]
features: [String.prototype.endsWith]
---*/

verifyProperty(String.prototype.endsWith, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
