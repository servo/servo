// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.values
description: >
    Set.prototype.values ( )

    17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Set.prototype.values,
  "function",
  "`typeof Set.prototype.values` is `'function'`"
);

verifyProperty(Set.prototype, "values", {
  writable: true,
  enumerable: false,
  configurable: true,
});
