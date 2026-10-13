// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: Checks the "transferToImmutable" property of ArrayBuffer.prototype
info: |
  ECMAScript Standard Built-in Objects
  ...
  Every built-in function object, including constructors, has a "length"
  property whose value is a non-negative integral Number. Unless otherwise
  specified, this value is the number of required parameters shown in the
  subclause heading for the function description. Optional parameters and rest
  parameters are not included in the parameter count.
  ...
  For functions that are specified as properties of objects, the name value is
  the property name string used to access the function.
features: [immutable-arraybuffer]
includes: [propertyHelper.js]
---*/

verifyPrimordialCallableProperty(
  ArrayBuffer.prototype,
  "transferToImmutable",
  "transferToImmutable",
  0
);
