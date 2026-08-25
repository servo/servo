// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.add
description: >
    Set.prototype.add ( value )

    17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Set.prototype.add, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
