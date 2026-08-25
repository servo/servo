// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Object.is, "name", {
  value: "is",
  writable: false,
  enumerable: false,
  configurable: true,
});
