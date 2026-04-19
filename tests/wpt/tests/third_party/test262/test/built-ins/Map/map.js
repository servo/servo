// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Map descriptor as a standard built-in object.
info: |
  Map ( [ iterable ] )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

verifyProperty(this, 'Map', {
  writable: true,
  enumerable: false,
  configurable: true
});
