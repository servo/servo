// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.12
description: >
  String.prototype.normalize.name value and descriptor.
info: |
  21.1.3.12 String.prototype.normalize ( [ form ] )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(String.prototype.normalize, "name", {
  value: "normalize",
  writable: false,
  enumerable: false,
  configurable: true
});
