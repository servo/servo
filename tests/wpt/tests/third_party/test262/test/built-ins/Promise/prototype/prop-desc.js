// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.prototype
description: Property descriptor of 'prototype' property
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

verifyProperty(Promise, 'prototype', {
  writable: false,
  enumerable: false,
  configurable: false,
});
