// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.foreach
description: >
    Set.prototype.forEach ( callbackfn [ , thisArg ] )

    17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Set.prototype.forEach,
  "function",
  "`typeof Set.prototype.forEach` is `'function'`"
);

verifyProperty(Set.prototype, "forEach", {
  writable: true,
  enumerable: false,
  configurable: true,
});
