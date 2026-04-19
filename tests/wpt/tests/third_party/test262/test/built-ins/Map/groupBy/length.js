// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy property length descriptor
info: |
  Map.groupBy ( items, callbackfn )

  ...

    17 ECMAScript Standard Built-in Objects

  ...

includes: [propertyHelper.js]
features: [array-grouping, Map]
---*/

verifyProperty(Map.groupBy, "length", {
  value: 2,
  enumerable: false,
  writable: false,
  configurable: true
});
