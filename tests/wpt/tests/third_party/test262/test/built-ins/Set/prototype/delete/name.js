// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.delete
description: >
    Set.prototype.delete ( value )

    17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Set.prototype.delete, "name", {
  value: "delete",
  writable: false,
  enumerable: false,
  configurable: true
});
