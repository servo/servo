// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  String.prototype.startsWith.length value and descriptor.
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  The length property of the startsWith method is 1.

includes: [propertyHelper.js]
---*/

verifyProperty(String.prototype.startsWith, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
