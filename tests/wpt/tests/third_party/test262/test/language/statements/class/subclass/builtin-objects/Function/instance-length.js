// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.4.1
description: Subclassed Function instances has length and name properties
info: |
  19.2.4.1 length

  The value of the length property is an integer that indicates the typical
  number of arguments expected by the function. However, the language permits
  the function to be invoked with some other number of arguments. The behaviour
  of a function when invoked on a number of arguments other than the number
  specified by its length property depends on the function. This property has
  the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

class Fn extends Function {}

var fn = new Fn('a', 'b', 'return a + b');

verifyProperty(fn, 'length', {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true,
});
