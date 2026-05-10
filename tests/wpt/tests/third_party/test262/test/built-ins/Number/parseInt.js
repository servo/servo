// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.parseint
description: >
  "parseInt" property descriptor and value of Number
info: |
  Number.parseInt

  The value of the Number.parseInt data property is the same built-in function
  object that is the value of the parseInt property of the global object
  defined in 18.2.4.

  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

assert.sameValue(Number.parseInt, parseInt);

verifyProperty(Number, "parseInt", {
  writable: true,
  enumerable: false,
  configurable: true
});
