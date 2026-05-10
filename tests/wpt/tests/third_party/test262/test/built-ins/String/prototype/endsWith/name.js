// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  String.prototype.endsWith.name value and descriptor.
info: |
  21.1.3.6 String.prototype.endsWith ( searchString [ , endPosition] )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [String.prototype.endsWith]
---*/

verifyProperty(String.prototype.endsWith, "name", {
  value: "endsWith",
  writable: false,
  enumerable: false,
  configurable: true
});
