// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.4
description: >
  String.raw.name value and property descriptor
info: |
  String.raw ( template , ...substitutions )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(String.raw, "name", {
  value: "raw",
  writable: false,
  enumerable: false,
  configurable: true
});
