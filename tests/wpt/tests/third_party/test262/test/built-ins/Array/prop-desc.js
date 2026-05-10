// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array-constructor
description: >
  Property descriptor of Array
info: |
  22.1.1 The Array Constructor

  * is the initial value of the Array property of the global object.

  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

verifyProperty(this, 'Array', {
  value: Array,
  writable: true,
  enumerable: false,
  configurable: true,
});
