// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy property length descriptor
info: |
  Object.groupBy ( items, callbackfn )

  ...

    17 ECMAScript Standard Built-in Objects

  ...

includes: [propertyHelper.js]
features: [array-grouping]
---*/

verifyProperty(Object.groupBy, "length", {
  value: 2,
  enumerable: false,
  writable: false,
  configurable: true
});
