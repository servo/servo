// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-symbol-constructor
description: >
  Symbol ( [ description ] )

includes: [propertyHelper.js]
features: [Symbol]
---*/

verifyProperty(Symbol, "name", {
  value: "Symbol",
  writable: false,
  enumerable: false,
  configurable: true
});
