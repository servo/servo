// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.13
description: >
  String.prototype.repeat.length value and descriptor.
info: |
  21.1.3.13 String.prototype.repeat ( count )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(String.prototype.repeat, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
