// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-weakset-constructor
description: >
  The length property of the WeakSet constructor is 0.
includes: [propertyHelper.js]
---*/

verifyProperty(WeakSet, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
