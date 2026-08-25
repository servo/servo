// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: Map.name value and descriptor.
info: |
  Map ( [ iterable ] )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(Map, "name", {
  value: "Map",
  writable: false,
  enumerable: false,
  configurable: true
});
