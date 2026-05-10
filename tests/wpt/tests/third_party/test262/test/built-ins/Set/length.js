// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set-constructor
description: >
    Properties of the Set Constructor

    Besides the length property (whose value is 0)

includes: [propertyHelper.js]
---*/

verifyProperty(Set, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
