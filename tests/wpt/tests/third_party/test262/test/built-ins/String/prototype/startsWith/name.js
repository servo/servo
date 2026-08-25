// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  String.prototype.startsWith.name value and descriptor.
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(String.prototype.startsWith, "name", {
  value: "startsWith",
  writable: false,
  enumerable: false,
  configurable: true
});
